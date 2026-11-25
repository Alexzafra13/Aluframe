//! Un formulario enviado dos veces (doble toque en la tablet, doble clic) se atiende una sola
//! vez: un cobro, una factura o un cliente no se duplican. Cada formulario lleva `_envio`,
//! aleatorio en cada página; si llega repetido, se espera a la primera respuesta y se devuelve
//! la misma redirección, sin repetir la acción. Es el respaldo de `envio-unico.js`, que ya lo
//! evita en el navegador (ADR-0036).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::header::LOCATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Un doble toque llega en milisegundos; se recuerda de sobra.
const VIGENCIA: Duration = Duration::from_secs(600);
const ESPERA_MAXIMA: Duration = Duration::from_secs(15);
const PAUSA: Duration = Duration::from_millis(20);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Envio {
    EnCurso,
    /// Terminó redirigiendo: el repetido va al mismo sitio.
    Redirigido(String),
    /// Terminó bien sin redirigir: el repetido no hace nada (204, la página se queda como está).
    Atendido,
}

type Envios = Mutex<HashMap<String, (Instant, Envio)>>;

fn envios() -> &'static Envios {
    static ENVIOS: OnceLock<Envios> = OnceLock::new();
    ENVIOS.get_or_init(Envios::default)
}

/// Atiende la petición ya validada (CSRF); `bytes` es su cuerpo de formulario.
pub async fn atender(partes: Parts, bytes: Bytes, siguiente: Next) -> Response {
    let clave = leer_campo(&bytes, "_envio")
        .filter(|clave| !clave.is_empty() && es_envio_de_pagina(&partes.headers))
        .map(str::to_owned);
    let peticion = Request::from_parts(partes, Body::from(bytes));
    let Some(clave) = clave else {
        return siguiente.run(peticion).await;
    };
    if !reservar(&clave) {
        return repetir(&clave).await;
    }
    let respuesta = siguiente.run(peticion).await;
    terminar(&clave, &respuesta);
    respuesta
}

/// Las peticiones HTMX de un trozo de página (el editor recalcula con el mismo formulario en
/// cada tecla) no son envíos; las de la navegación sin recarga (`HX-Boosted`) sí.
fn es_envio_de_pagina(cabeceras: &HeaderMap) -> bool {
    !cabeceras.contains_key("hx-request") || cabeceras.contains_key("hx-boosted")
}

/// `true` si es la primera vez que llega este envío.
fn reservar(clave: &str) -> bool {
    let Ok(mut mapa) = envios().lock() else {
        return true;
    };
    let ahora = Instant::now();
    mapa.retain(|_, (cuando, _)| ahora.duration_since(*cuando) < VIGENCIA);
    if mapa.contains_key(clave) {
        return false;
    }
    mapa.insert(clave.to_owned(), (ahora, Envio::EnCurso));
    true
}

/// Un error (validación, permisos) no cuenta: se puede corregir y volver a enviar.
fn terminar(clave: &str, respuesta: &Response) {
    let Ok(mut mapa) = envios().lock() else {
        return;
    };
    let estado = respuesta.status();
    let destino = respuesta
        .headers()
        .get(LOCATION)
        .and_then(|valor| valor.to_str().ok());
    let resultado = match (estado.is_redirection(), destino) {
        (true, Some(destino)) => Envio::Redirigido(destino.to_owned()),
        _ if estado.is_success() => Envio::Atendido,
        _ => {
            mapa.remove(clave);
            return;
        }
    };
    mapa.insert(clave.to_owned(), (Instant::now(), resultado));
}

async fn repetir(clave: &str) -> Response {
    let limite = Instant::now() + ESPERA_MAXIMA;
    while Instant::now() < limite {
        match consultar(clave) {
            Some(Envio::EnCurso) => tokio::time::sleep(PAUSA).await,
            Some(Envio::Redirigido(destino)) => {
                return (StatusCode::SEE_OTHER, [(LOCATION, destino)]).into_response();
            }
            Some(Envio::Atendido) => return StatusCode::NO_CONTENT.into_response(),
            // La primera falló: este envío tampoco se atiende; quien lo envió ve su formulario.
            None => return StatusCode::NO_CONTENT.into_response(),
        }
    }
    (
        StatusCode::CONFLICT,
        "Ya se está guardando. Espera un momento.",
    )
        .into_response()
}

fn consultar(clave: &str) -> Option<Envio> {
    let mapa = envios().lock().ok()?;
    mapa.get(clave).map(|(_, envio)| envio.clone())
}

fn leer_campo<'a>(cuerpo: &'a [u8], nombre: &str) -> Option<&'a str> {
    std::str::from_utf8(cuerpo)
        .ok()?
        .split('&')
        .filter_map(|par| par.split_once('='))
        .find(|(clave, _)| *clave == nombre)
        .map(|(_, valor)| valor)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn el_trozo_de_htmx_no_es_un_envio_y_la_navegacion_sin_recarga_si() {
        let mut cabeceras = HeaderMap::new();
        assert!(es_envio_de_pagina(&cabeceras));
        cabeceras.insert("hx-request", HeaderValue::from_static("true"));
        assert!(!es_envio_de_pagina(&cabeceras));
        cabeceras.insert("hx-boosted", HeaderValue::from_static("true"));
        assert!(es_envio_de_pagina(&cabeceras));
    }

    #[test]
    fn lee_el_campo_del_formulario() {
        assert_eq!(leer_campo(b"_csrf=a&_envio=b1&x=2", "_envio"), Some("b1"));
        assert_eq!(leer_campo(b"_csrf=a", "_envio"), None);
    }
}
