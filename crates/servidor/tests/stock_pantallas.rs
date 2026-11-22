#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use comun::{Contexto, destino, formulario, get, post, texto};
use dominio::unidades::Porcentaje;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;

/// Trabajo 1 aceptado; devuelve el token y el CSRF del propietario.
async fn preparado(contexto: &Contexto) -> (String, String) {
    let (token, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    let linea = (
        ventana(150, 120),
        Porcentaje::desde_puntos_basicos(2_100),
        false,
    );
    guardar_ventana(&estado, &actor, (id, 0), linea)
        .await
        .unwrap();
    aceptar_presupuesto(&estado, &actor, id).await.unwrap();
    (token, actor.token_csrf)
}

async fn id_de(contexto: &Contexto, referencia: &'static str) -> String {
    contexto
        .estado
        .bd
        .leer(move |con| {
            Ok(con.query_row(
                "SELECT id FROM articulo WHERE referencia = ?1",
                [referencia],
                |f| f.get::<_, i64>(0),
            )?)
        })
        .await
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn entradas_retales_y_minimo_se_ven_en_el_stock_y_en_el_inicio() {
    let contexto = Contexto::nuevo().await;
    let (token, csrf) = preparado(&contexto).await;
    let rueda = id_de(&contexto, "DEMO_RUEDA").await;
    let hoja = id_de(&contexto, "DEMO_HOJA_HORIZONTAL").await;
    let entrada = formulario(&[("_csrf", &csrf), ("articulo", &rueda), ("cantidad", "3")]);
    let respuesta = contexto
        .pedir(post("/stock/entrada", Some(&token), &entrada))
        .await;
    assert_eq!(destino(&respuesta), Some("/stock?aviso=entrada"));
    let retal = formulario(&[
        ("_csrf", &csrf),
        ("articulo", &hoja),
        ("cantidad", "1"),
        ("largo", "300"),
    ]);
    contexto
        .pedir(post("/stock/entrada", Some(&token), &retal))
        .await;
    let minimo = formulario(&[("_csrf", &csrf), ("minimo", "10")]);
    contexto
        .pedir(post(
            &format!("/stock/{rueda}/minimo"),
            Some(&token),
            &minimo,
        ))
        .await;
    let stock = texto(contexto.pedir(get("/stock", Some(&token))).await).await;
    assert!(stock.contains("1 × 300 cm") && stock.contains("Bajo mínimo"));
    let inicio = texto(contexto.pedir(get("/", Some(&token))).await).await;
    assert!(inicio.contains("DEMO_RUEDA") && inicio.contains("quedan 3"));
    let compra = texto(contexto.pedir(get("/stock/compra", Some(&token))).await).await;
    assert!(
        compra.contains("DEMO_RUEDA"),
        "bajo mínimo y lo que necesita el trabajo"
    );
    let orden = texto(contexto.pedir(get("/trabajos/1/orden", Some(&token))).await).await;
    assert!(orden.contains("Coger el retal de 3000 del stock"));
}

#[tokio::test]
async fn terminar_la_orden_desde_la_pantalla_confirma_los_retales() {
    let contexto = Contexto::nuevo().await;
    let (token, csrf) = preparado(&contexto).await;
    let vista = texto(
        contexto
            .pedir(get("/trabajos/1/terminar", Some(&token)))
            .await,
    )
    .await;
    assert!(vista.contains("Sale del stock") && vista.contains("Retales que entran"));
    let confirmar = formulario(&[
        ("_csrf", &csrf),
        ("extra_grupo_0", "0"),
        ("extra_largo_0", "abc"),
    ]);
    let mal = contexto
        .pedir(post("/trabajos/1/terminar", Some(&token), &confirmar))
        .await;
    assert_eq!(
        mal.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        "un largo mal escrito no termina nada"
    );
    let bien = formulario(&[("_csrf", &csrf)]);
    let respuesta = contexto
        .pedir(post("/trabajos/1/terminar", Some(&token), &bien))
        .await;
    assert_eq!(destino(&respuesta), Some("/trabajos/1?aviso=terminada"));
    let ficha = texto(contexto.pedir(get("/trabajos/1", Some(&token))).await).await;
    assert!(ficha.contains("Orden terminada"));
    let otra = contexto
        .pedir(get("/trabajos/1/terminar", Some(&token)))
        .await;
    assert_eq!(otra.status(), axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}
