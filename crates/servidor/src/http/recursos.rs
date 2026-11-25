use axum::extract::{Path, RawQuery};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "recursos/"]
struct Recursos;

pub async fn servir_recurso(Path(ruta): Path<String>, RawQuery(consulta): RawQuery) -> Response {
    let Some(archivo) = Recursos::get(&ruta) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cabeceras = [
        (CONTENT_TYPE, tipo_de_contenido(&ruta)),
        (CACHE_CONTROL, duracion_en_cache(&ruta, consulta.as_deref())),
    ];
    (cabeceras, archivo.data.into_owned()).into_response()
}

/// Logotipos de los fabricantes: como las siluetas, un día en el navegador.
pub async fn servir_logo(Path(archivo): Path<String>) -> Response {
    let Some(datos) = crate::series::marcas::leer_logo(&archivo) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cabeceras = [
        (CONTENT_TYPE, "image/png"),
        (CACHE_CONTROL, "public, max-age=86400"),
    ];
    (cabeceras, datos).into_response()
}

/// Siluetas de perfiles: no cambian dentro de una versión, se guardan un día en el navegador.
pub async fn servir_silueta(Path(ruta): Path<String>) -> Response {
    let Some(datos) = crate::series::siluetas::leer_silueta(&ruta) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cabeceras = [
        (CONTENT_TYPE, "image/svg+xml"),
        (CACHE_CONTROL, "public, max-age=86400"),
    ];
    (cabeceras, datos).into_response()
}

/// Con `?v=<versión>` la dirección cambia en cada versión: el navegador lo guarda un año sin
/// volver a preguntar (la tablet carga antes). Las letras se piden desde el CSS sin versión, pero
/// no cambian nunca: también un año (para cambiar una, cambia el nombre de su archivo).
/// El resto, una hora.
fn duracion_en_cache(ruta: &str, consulta: Option<&str>) -> &'static str {
    let con_version = consulta.is_some_and(|c| c.split('&').any(|par| par.starts_with("v=")));
    if con_version || ruta.starts_with("fuentes/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}

fn tipo_de_contenido(ruta: &str) -> &'static str {
    match ruta.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn solo_lo_que_lleva_la_version_se_guarda_un_ano() {
        assert!(duracion_en_cache("estilos.css", Some("v=0.13.1")).contains("immutable"));
        assert!(duracion_en_cache("estilos.css", Some("x=1&v=0.13.1")).contains("immutable"));
        assert_eq!(
            duracion_en_cache("estilos.css", None),
            "public, max-age=3600"
        );
        assert_eq!(
            duracion_en_cache("estilos.css", Some("vista=1")),
            "public, max-age=3600"
        );
    }

    #[test]
    fn las_letras_se_guardan_un_ano_aunque_el_css_las_pida_sin_version() {
        let ruta = "fuentes/IBMPlexSans-Regular-Latin1.woff2";
        assert!(Recursos::get(ruta).is_some());
        assert!(duracion_en_cache(ruta, None).contains("immutable"));
    }

    /// El logotipo del programa y sus iconos son PNG (`recursos/marca/`).
    #[test]
    fn el_logotipo_se_sirve_como_png() {
        assert_eq!(tipo_de_contenido("marca/logo.png"), "image/png");
        assert!(Recursos::get("marca/logo.png").is_some());
        assert!(Recursos::get("marca/simbolo.png").is_some());
    }
}
