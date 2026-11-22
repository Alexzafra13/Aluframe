#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use comun::{Contexto, formulario, post, texto};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use dominio::dibujo::Superficie;
use servidor::aplicacion::acabados_incluidos::{
    cargar_acabados_incluidos, leer_archivo_de_acabados,
};
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::series_incluidas::cargar_series_incluidas;
use servidor::estado::EstadoApp;

async fn fila(estado: &EstadoApp, nombre: &'static str) -> (i64, String, i64, String) {
    estado
        .bd
        .leer(move |con| {
            Ok(con.query_row(
                "SELECT id, color_dibujo, recargo_bp, cierres FROM acabado WHERE nombre = ?1",
                [nombre],
                |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?)),
            )?)
        })
        .await
        .unwrap()
}

#[test]
fn el_archivo_de_acabados_del_programa_es_valido() {
    let acabados = leer_archivo_de_acabados().unwrap();
    let blanco = acabados.iter().find(|a| a.nombre == "Blanco").unwrap();
    assert!(blanco.base && blanco.ral.as_deref() == Some("9016"));
    assert_eq!(
        blanco.color.superficie,
        Superficie::Brillo,
        "el blanco va en brillo"
    );
    let anodizados = acabados
        .iter()
        .filter(|a| a.color.superficie == Superficie::Anodizado);
    assert!(anodizados.count() >= 2);
    assert!(acabados.iter().filter(|a| a.color.madera).count() >= 3);
}

#[tokio::test]
async fn se_cargan_una_vez_y_no_tocan_el_recargo_del_propietario() {
    let contexto = Contexto::nuevo().await;
    let creados = cargar_acabados_incluidos(&contexto.estado).await.unwrap();
    assert_eq!(creados, leer_archivo_de_acabados().unwrap().len());
    contexto
        .estado
        .bd
        .escribir(|tx| {
            tx.execute(
                "UPDATE acabado SET recargo_bp = 1500, color_dibujo = '#123456' WHERE nombre = 'Negro'",
                [],
            )?;
            Ok(())
        })
        .await
        .unwrap();

    assert_eq!(
        cargar_acabados_incluidos(&contexto.estado).await.unwrap(),
        0
    );
    let (_, color, recargo, cierres) = fila(&contexto.estado, "Negro").await;
    assert_eq!(recargo, 1500, "el recargo es del propietario");
    assert_eq!(color, "#0e0e10", "el color lo corrige el archivo");
    assert_eq!(cierres, "negro");
}

/// Un presupuesto vacío con las series y los colores de serie; devuelve (contexto, sesión, csrf).
async fn presupuesto_con_colores() -> (Contexto, String, String) {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    cargar_acabados_incluidos(&contexto.estado).await.unwrap();
    let cliente = validar_cliente(&EntradaDeCliente::default()).unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    let crear = formulario(&[("_csrf", &actor.token_csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &crear))
        .await;
    (contexto, token, actor.token_csrf)
}

async fn dibujar_con(contexto: &Contexto, (token, csrf): (&str, &str), acabado: &str) -> String {
    let ventana = [
        ("_csrf", csrf),
        ("seleccion", "1|corredera_2_hojas"),
        ("ancho", "150"),
        ("alto", "120"),
        ("iva", "2100"),
        ("acabado", acabado),
    ];
    let respuesta = contexto
        .pedir(post(
            "/presupuestos/1/lineas/calcular",
            Some(token),
            &formulario(&ventana),
        ))
        .await;
    texto(respuesta).await
}

/// Lo que pidió el propietario: la ventana en su color; blanca y con cierres blancos por defecto.
#[tokio::test]
async fn la_ventana_se_dibuja_en_el_color_elegido() {
    let (contexto, token, csrf) = presupuesto_con_colores().await;
    let (verde, ..) = fila(&contexto.estado, "Verde").await;
    let verde = verde.to_string();
    for (acabado, relleno, cierres) in [
        ("", "#f2f2ed", "#f4f4f2"),
        (verde.as_str(), "#0f4336", "#15191d"),
    ] {
        let html = dibujar_con(&contexto, (&token, &csrf), acabado).await;
        assert!(
            html.contains(&format!("fill=\"{relleno}\"")),
            "marco {relleno}"
        );
        assert!(html.contains(&format!("class=\"herrajes\" fill=\"{cierres}\"")));
    }
}
