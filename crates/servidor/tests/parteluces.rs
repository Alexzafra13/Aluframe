#![allow(clippy::unwrap_used, clippy::panic)]

//! Parteluces en Aluprom 21 (ADR-0030), con la serie real: lo que el propietario ve al elegirlo.

mod comun;

use comun::{Contexto, formulario, get, post, texto};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::series_incluidas::cargar_series_incluidas;

/// Abre un presupuesto y calcula en vivo una corredera de 2 hojas de 150 × 120 con las
/// opciones dadas; devuelve (fragmento calculado, página del editor).
async fn editor_con(opciones: &[(&str, &str)]) -> (String, String) {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let cliente = validar_cliente(&EntradaDeCliente::default()).unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    let csrf = actor.token_csrf.as_str();
    let cuerpo = formulario(&[("_csrf", csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &cuerpo))
        .await;
    let mut ventana = vec![
        ("_csrf", csrf),
        ("seleccion", "1|corredera_2_hojas"),
        ("ancho", "150"),
        ("alto", "120"),
        ("iva", "2100"),
    ];
    ventana.extend_from_slice(opciones);
    let ruta = "/presupuestos/1/lineas/calcular";
    let fragmento = texto(
        contexto
            .pedir(post(ruta, Some(&token), &formulario(&ventana)))
            .await,
    );
    let editor = get("/presupuestos/1/lineas/nueva", Some(&token));
    let pagina = texto(contexto.pedir(editor).await);
    (fragmento.await, pagina.await)
}

#[tokio::test]
async fn las_opciones_del_parteluces_se_ocultan_hasta_elegirlo() {
    let (_, pagina) = editor_con(&[]).await;
    assert!(pagina.contains(r#"name="opcion.parteluces""#));
    assert!(
        pagina.contains(r#"data-solo-si="parteluces=si" hidden"#),
        "{pagina}"
    );
    assert!(pagina.contains(r#"name="opcion.medida_abajo" value="50""#));
}

#[tokio::test]
async fn con_parteluces_sale_la_chapa_con_su_dibujo() {
    let (html, _) = editor_con(&[("opcion.parteluces", "si"), ("opcion.medida_abajo", "45")]).await;
    assert!(html.contains("Chapa de abajo"));
    assert!(
        html.contains("? × 45"),
        "el ancho de la chapa aún no se sabe"
    );
    assert!(html.contains(r#"class="dibujo-chapa""#));
    assert_eq!(
        html.matches(r#"class="parteluces""#).count(),
        2,
        "uno por hoja"
    );
    // El dibujo del presupuesto solo lleva la medida total; la cota de la chapa va en el del
    // taller (propietario, 2026-11-25). Su medida sigue en la lista de chapas.
    assert!(!html.contains(">chapa 45<"));
    assert!(html.contains("Chapa: falta decidir cómo se cobra"));
    // El parteluces sale en los perfiles a cortar con su dibujo aunque falte su medida.
    assert!(html.contains(r#"src="/perfiles/aluval/21564.svg""#));
    assert!(html.contains(">pendiente<"));
}

#[tokio::test]
async fn con_cristal_abajo_no_hay_chapa_y_el_cristal_de_abajo_mide_lo_escrito() {
    let opciones = [
        ("opcion.parteluces", "si"),
        ("opcion.relleno_abajo", "cristal"),
        ("opcion.medida_abajo", "60"),
    ];
    let (html, _) = editor_con(&opciones).await;
    assert!(!html.contains("Chapa de abajo"));
    assert!(html.contains("Cristal abajo"), "{html}");
    assert!(html.contains("65,2 × 60"));
}

#[tokio::test]
async fn sin_parteluces_la_corredera_queda_como_estaba() {
    let (html, _) = editor_con(&[("opcion.medida_abajo", "5")]).await;
    assert!(!html.contains(r#"class="parteluces""#));
    assert!(
        !html.contains("Medida de abajo: escribe"),
        "la medida oculta no cuenta"
    );
    assert!(html.contains("Para pedir") && html.contains("65,2 × 106,9"));
}
