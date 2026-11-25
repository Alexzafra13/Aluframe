use crate::comun;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use comun::{Contexto, destino, formulario, get, post, texto};
use servidor::aplicacion::datos::exportar_datos;

async fn preparado() -> (Contexto, String, servidor::aplicacion::Actor) {
    let mut contexto = Contexto::nuevo().await;
    let carpeta = std::env::temp_dir().join(format!(
        "alumframe-pantallas-{}",
        servidor::autenticacion::generar_token().unwrap()
    ));
    contexto.estado = contexto.estado.clone().con_carpeta_copias(carpeta);
    let (token, actor) = contexto.con_propietario().await;
    (contexto, token, actor)
}

fn subir(token: &str, csrf: &str, cuerpo: Vec<u8>) -> Request<Body> {
    Request::post("/ajustes/datos/importar")
        .header(header::COOKIE, format!("alumframe_sesion={token}"))
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header("x-csrf-token", csrf)
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(cuerpo))
        .unwrap()
}

#[tokio::test]
async fn la_pantalla_de_datos_ofrece_exportar_e_importar() {
    let (contexto, token, _) = preparado().await;
    let pagina = texto(contexto.pedir(get("/ajustes/datos", Some(&token))).await).await;
    assert!(pagina.contains("href=\"/ajustes/datos/exportar\""));
    assert!(pagina.contains("Importar sustituye todos los datos de aquí"));
    let descarga = contexto
        .pedir(get("/ajustes/datos/exportar", Some(&token)))
        .await;
    assert_eq!(descarga.status(), StatusCode::OK);
    let adjunto = descarga.headers()[header::CONTENT_DISPOSITION]
        .to_str()
        .unwrap();
    assert!(adjunto.contains("alumframe-2026-11-21.alumframe"));
}

#[tokio::test]
async fn subir_revisar_y_confirmar_una_copia() {
    let (contexto, token, actor) = preparado().await;
    let copia = exportar_datos(&contexto.estado, &actor).await.unwrap();
    let subida = contexto
        .pedir(subir(&token, &actor.token_csrf, copia.bytes))
        .await;
    assert_eq!(subida.status(), StatusCode::SEE_OTHER);
    let revision = destino(&subida).unwrap().to_owned();
    assert!(revision.starts_with("/ajustes/datos/importar/"));

    let pagina = texto(contexto.pedir(get(&revision, Some(&token))).await).await;
    assert!(pagina.contains("Sustituir los datos por la copia"));
    let cuerpo = formulario(&[("_csrf", &actor.token_csrf)]);
    let confirmada = contexto.pedir(post(&revision, Some(&token), &cuerpo)).await;
    assert_eq!(destino(&confirmada), Some("/ajustes/datos?importado=si"));
}

#[tokio::test]
async fn un_archivo_que_no_es_copia_devuelve_el_motivo_como_texto() {
    let (contexto, token, actor) = preparado().await;
    let respuesta = contexto
        .pedir(subir(&token, &actor.token_csrf, b"hola".to_vec()))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        texto(respuesta)
            .await
            .starts_with("El archivo no es una copia")
    );
}

#[tokio::test]
async fn sin_token_csrf_no_se_acepta_la_subida() {
    let (contexto, token, _) = preparado().await;
    let respuesta = contexto
        .pedir(subir(&token, "token-falso", b"hola".to_vec()))
        .await;
    assert_eq!(respuesta.status(), StatusCode::FORBIDDEN);
}
