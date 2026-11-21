use axum::extract::Path;
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "recursos/"]
struct Recursos;

pub async fn servir_recurso(Path(ruta): Path<String>) -> Response {
    let Some(archivo) = Recursos::get(&ruta) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cabeceras = [
        (CONTENT_TYPE, tipo_de_contenido(&ruta)),
        (CACHE_CONTROL, "public, max-age=3600"),
    ];
    (cabeceras, archivo.data.into_owned()).into_response()
}

fn tipo_de_contenido(ruta: &str) -> &'static str {
    match ruta.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
