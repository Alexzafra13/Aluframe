#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use comun::presupuestos::{SERIE_DEMO, elegir_reglas, preparar_demo};
use comun::{Contexto, destino, formulario, get, post, texto};

const PNG: [u8; 12] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

fn subir_logo(token: &str, csrf: &str, cuerpo: &[u8]) -> Request<Body> {
    Request::post("/ajustes/presupuestos/logo")
        .header(header::COOKIE, format!("alumframe_sesion={token}"))
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header("x-csrf-token", csrf)
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(cuerpo.to_vec()))
        .unwrap()
}

/// Presupuesto 1 con una línea libre, para ver su hoja impresa.
async fn con_presupuesto() -> (Contexto, String, String) {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_DEMO).await;
    elegir_reglas(&estado, &actor).await;
    let csrf = actor.token_csrf;
    let nuevo = formulario(&[("_csrf", &csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &nuevo))
        .await;
    let libre = formulario(&[
        ("_csrf", &csrf),
        ("descripcion", "Instalación"),
        ("cantidad", "1"),
        ("precio", "80"),
        ("iva", "2100"),
    ]);
    contexto
        .pedir(post("/presupuestos/1/lineas/libre", Some(&token), &libre))
        .await;
    (contexto, token, csrf)
}

async fn hoja(contexto: &Contexto, token: &str) -> String {
    texto(
        contexto
            .pedir(get("/presupuestos/1/imprimir", Some(token)))
            .await,
    )
    .await
}

#[tokio::test]
async fn sin_logotipo_la_hoja_lleva_el_nombre_de_la_empresa() {
    let (contexto, token, _) = con_presupuesto().await;
    let impreso = hoja(&contexto, &token).await;
    assert!(impreso.contains("<p class=\"a4-marca\">Taller de prueba</p>"));
    assert!(!impreso.contains("a4-logo"));
    assert!(impreso.contains("Conforme, el cliente"));
}

#[tokio::test]
async fn con_logotipo_la_hoja_lo_lleva_y_se_puede_quitar() {
    let (contexto, token, csrf) = con_presupuesto().await;
    let respuesta = contexto.pedir(subir_logo(&token, &csrf, &PNG)).await;
    assert_eq!(
        destino(&respuesta),
        Some("/ajustes/presupuestos?guardado=logo")
    );
    let impreso = hoja(&contexto, &token).await;
    assert!(impreso.contains("class=\"a4-logo\" src=\"data:image/png;base64,iVBORw0KGgo"));
    let ajustes = texto(
        contexto
            .pedir(get("/ajustes/presupuestos?guardado=logo", Some(&token)))
            .await,
    )
    .await;
    assert!(ajustes.contains("Logotipo guardado.") && ajustes.contains("Quitar logotipo"));
    let quitar = formulario(&[("_csrf", &csrf)]);
    contexto
        .pedir(post(
            "/ajustes/presupuestos/logo/quitar",
            Some(&token),
            &quitar,
        ))
        .await;
    assert!(hoja(&contexto, &token).await.contains("a4-marca"));
}

#[tokio::test]
async fn rechaza_un_logotipo_que_no_es_png_ni_jpg() {
    let (contexto, token, csrf) = con_presupuesto().await;
    let svg = b"<svg xmlns='http://www.w3.org/2000/svg'><script/></svg>";
    let respuesta = contexto.pedir(subir_logo(&token, &csrf, svg)).await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        texto(respuesta)
            .await
            .contains("tiene que ser una imagen PNG o JPG")
    );
    assert!(!hoja(&contexto, &token).await.contains("a4-logo"));
}

#[tokio::test]
async fn los_datos_de_la_empresa_se_guardan_normalizados_o_se_avisa() {
    let (contexto, token, csrf) = con_presupuesto().await;
    let mal = formulario(&[
        ("_csrf", &csrf),
        ("iva.general", "21"),
        ("empresa.iban", "ES00 1234"),
    ]);
    let respuesta = contexto
        .pedir(post("/ajustes/presupuestos", Some(&token), &mal))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(texto(respuesta).await.contains("IBAN"));
    let bien = formulario(&[
        ("_csrf", &csrf),
        ("iva.general", "21"),
        ("empresa.nombre", "Taller de prueba"),
        ("empresa.movil", "612345678"),
        ("empresa.iban", "es9121000418450200051332"),
    ]);
    contexto
        .pedir(post("/ajustes/presupuestos", Some(&token), &bien))
        .await;
    let impreso = hoja(&contexto, &token).await;
    assert!(impreso.contains("Móvil 612 345 678"));
    assert!(impreso.contains("IBAN ES91 2100 0418 4502 0005 1332"));
}
