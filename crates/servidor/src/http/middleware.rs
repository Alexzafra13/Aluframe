use axum::body::{Body, to_bytes};
use axum::extract::{Request, State};
use axum::http::header::{CONTENT_TYPE, HOST, ORIGIN};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};

use crate::aplicacion::{Actor, sesion};
use crate::autenticacion::{leer_cookie_de_sesion, tokens_iguales};
use crate::estado::EstadoApp;

/// Una tarifa CSV de 2 MB codificada en un formulario puede triplicar su tamaño.
const LIMITE_DE_FORMULARIO: usize = 8 * 1024 * 1024;
const RUTAS_SIN_CONFIGURAR: [&str; 3] = ["/primer-arranque", "/health", "/recursos/"];

pub async fn exigir_configuracion(
    State(estado): State<EstadoApp>,
    peticion: Request,
    siguiente: Next,
) -> Response {
    let ruta = peticion.uri().path();
    let permitida = RUTAS_SIN_CONFIGURAR.iter().any(|r| ruta.starts_with(r));
    if !estado.hay_usuarios() && !permitida {
        return Redirect::to("/primer-arranque").into_response();
    }
    siguiente.run(peticion).await
}

pub async fn cargar_sesion(
    State(estado): State<EstadoApp>,
    mut peticion: Request,
    siguiente: Next,
) -> Response {
    if let Some(token) = leer_cookie_de_sesion(peticion.headers()) {
        match sesion::identificar(&estado, &token).await {
            Ok(Some(actor)) => {
                peticion.extensions_mut().insert(actor);
            }
            Ok(None) => {}
            Err(error) => return error.into_response(),
        }
    }
    siguiente.run(peticion).await
}

/// Doble defensa: el origen debe ser el propio servidor y, con sesión, el token CSRF debe coincidir.
pub async fn proteger_csrf(peticion: Request, siguiente: Next) -> Response {
    if es_metodo_seguro(peticion.method()) {
        return siguiente.run(peticion).await;
    }
    if !es_mismo_origen(peticion.headers()) {
        return rechazar();
    }
    let Some(esperado) = peticion
        .extensions()
        .get::<Actor>()
        .map(|a| a.token_csrf.clone())
    else {
        return siguiente.run(peticion).await;
    };
    if let Some(enviado) = peticion.headers().get("x-csrf-token") {
        let valido = enviado.to_str().is_ok_and(|t| tokens_iguales(t, &esperado));
        return if valido {
            siguiente.run(peticion).await
        } else {
            rechazar()
        };
    }
    let (partes, cuerpo) = peticion.into_parts();
    let Ok(bytes) = to_bytes(cuerpo, LIMITE_DE_FORMULARIO).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    let es_formulario = partes.headers.get(CONTENT_TYPE).is_some_and(|tipo| {
        tipo.as_bytes()
            .starts_with(b"application/x-www-form-urlencoded")
    });
    let valido = es_formulario
        && leer_campo_csrf(&bytes).is_some_and(|token| tokens_iguales(token, &esperado));
    if !valido {
        return rechazar();
    }
    siguiente
        .run(Request::from_parts(partes, Body::from(bytes)))
        .await
}

pub async fn anadir_cabeceras_de_seguridad(peticion: Request, siguiente: Next) -> Response {
    let mut respuesta = siguiente.run(peticion).await;
    let cabeceras = respuesta.headers_mut();
    for (nombre, valor) in [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "same-origin"),
        (
            "content-security-policy",
            "default-src 'self'; img-src 'self' data:; frame-ancestors 'none'; form-action 'self'",
        ),
    ] {
        cabeceras.insert(nombre, HeaderValue::from_static(valor));
    }
    respuesta
}

fn es_metodo_seguro(metodo: &Method) -> bool {
    matches!(*metodo, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn es_mismo_origen(cabeceras: &HeaderMap) -> bool {
    if let Some(sitio) = cabeceras.get("sec-fetch-site") {
        return matches!(sitio.as_bytes(), b"same-origin" | b"none");
    }
    let Some(origen) = cabeceras.get(ORIGIN).and_then(|o| o.to_str().ok()) else {
        return true;
    };
    let anfitrion = cabeceras
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let sin_esquema = origen.split_once("://").map_or(origen, |(_, resto)| resto);
    !anfitrion.is_empty() && sin_esquema == anfitrion
}

fn leer_campo_csrf(cuerpo: &[u8]) -> Option<&str> {
    std::str::from_utf8(cuerpo)
        .ok()?
        .split('&')
        .filter_map(|par| par.split_once('='))
        .find(|(clave, _)| *clave == "_csrf")
        .map(|(_, valor)| valor)
}

fn rechazar() -> Response {
    (
        StatusCode::FORBIDDEN,
        "Petición rechazada: recarga la página e inténtalo de nuevo.",
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_el_token_del_formulario() {
        assert_eq!(leer_campo_csrf(b"nombre=Ana&_csrf=abc&pin=1"), Some("abc"));
        assert_eq!(leer_campo_csrf(b"nombre=Ana"), None);
    }

    #[test]
    fn rechaza_peticiones_de_otro_origen() {
        let mut cabeceras = HeaderMap::new();
        cabeceras.insert(HOST, HeaderValue::from_static("taller:9090"));
        cabeceras.insert(ORIGIN, HeaderValue::from_static("http://taller:9090"));
        assert!(es_mismo_origen(&cabeceras));
        cabeceras.insert(ORIGIN, HeaderValue::from_static("http://malicioso.example"));
        assert!(!es_mismo_origen(&cabeceras));
        cabeceras.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
        assert!(!es_mismo_origen(&cabeceras));
    }
}
