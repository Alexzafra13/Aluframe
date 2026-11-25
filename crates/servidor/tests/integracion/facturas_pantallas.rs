use crate::comun;

use axum::http::StatusCode;
use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use comun::{Contexto, destino, formulario, get, post, texto};
use dominio::unidades::Porcentaje;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;

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

fn cobro<'a>(csrf: &'a str, importe: &'a str, medio: &'a str) -> String {
    formulario(&[
        ("_csrf", csrf),
        ("importe", importe),
        ("fecha", "2026-11-22"),
        ("medio", medio),
    ])
}

/// Señal en el trabajo y factura creada desde él; emitida tras confirmar que falta el NIF.
async fn emitida_con_senal(contexto: &Contexto, token: &str, csrf: &str) {
    let senal = contexto
        .pedir(post(
            "/trabajos/1/senal",
            Some(token),
            &cobro(csrf, "100", "bizum"),
        ))
        .await;
    assert_eq!(destino(&senal), Some("/trabajos/1?aviso=senal"));
    let nueva = contexto
        .pedir(post(
            "/trabajos/1/factura",
            Some(token),
            &formulario(&[("_csrf", csrf)]),
        ))
        .await;
    assert_eq!(destino(&nueva), Some("/facturas/1"));
    let sin_confirmar = contexto
        .pedir(post(
            "/facturas/1/emitir",
            Some(token),
            &formulario(&[("_csrf", csrf)]),
        ))
        .await;
    assert_eq!(sin_confirmar.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(texto(sin_confirmar).await.contains("no tiene NIF"));
    let emitir = formulario(&[("_csrf", csrf), ("confirmar", "si")]);
    let emitida = contexto
        .pedir(post("/facturas/1/emitir", Some(token), &emitir))
        .await;
    assert_eq!(destino(&emitida), Some("/facturas/1?aviso=emitida"));
}

#[tokio::test]
async fn senal_factura_emitida_y_cobro_desde_las_pantallas() {
    let contexto = Contexto::nuevo().await;
    let (token, csrf) = preparado(&contexto).await;
    emitida_con_senal(&contexto, &token, &csrf).await;
    let hoja = texto(
        contexto
            .pedir(get("/facturas/1/imprimir", Some(&token)))
            .await,
    )
    .await;
    assert!(hoja.contains("F-2026-0001") && hoja.contains("Base imponible"));
    let pagina = texto(contexto.pedir(get("/facturas/1", Some(&token))).await).await;
    assert!(pagina.contains("Parcial") && pagina.contains("Registrar cobro"));
    let mas = contexto
        .pedir(post(
            "/facturas/1/cobros",
            Some(&token),
            &cobro(&csrf, "10", "efectivo"),
        ))
        .await;
    assert_eq!(destino(&mas), Some("/facturas/1?aviso=cobro"));
    let lista = texto(
        contexto
            .pedir(get("/facturas?pendientes=si", Some(&token)))
            .await,
    )
    .await;
    assert!(lista.contains("F-2026-0001"));
}

#[tokio::test]
async fn el_albaran_se_imprime_y_entrega_el_trabajo() {
    let contexto = Contexto::nuevo().await;
    let (token, csrf) = preparado(&contexto).await;
    let albaran = contexto
        .pedir(post(
            "/trabajos/1/albaran",
            Some(&token),
            &formulario(&[("_csrf", &csrf)]),
        ))
        .await;
    assert_eq!(destino(&albaran), Some("/albaranes/1/imprimir"));
    let hoja = texto(
        contexto
            .pedir(get("/albaranes/1/imprimir", Some(&token)))
            .await,
    )
    .await;
    assert!(hoja.contains("A-2026-0001") && hoja.contains("Recibí conforme"));
    let ficha = texto(contexto.pedir(get("/trabajos/1", Some(&token))).await).await;
    assert!(ficha.contains("Entregado"));
}
