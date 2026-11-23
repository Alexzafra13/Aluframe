//! Entrar en la demostración, salir de ella y empezarla de nuevo (ADR-0028).

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Redirect, Response};

use crate::aplicacion::demostracion::{asegurar_rellena, empezar_de_nuevo};
use crate::autenticacion::{
    cookie_de_cierre, cookie_de_sesion, leer_cookie, leer_cookie_de_sesion,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::demostracion::{
    COOKIE_VOLVER, cookie_de_entrada, cookie_de_salida, cookie_para_volver,
    cookie_para_volver_borrada,
};
use crate::http::extractores::ActorActual;

/// Se guarda la sesión de verdad para volver a ella y se va a la entrada de la demostración.
pub async fn entrar(
    State(estado): State<EstadoApp>,
    cabeceras: HeaderMap,
) -> Result<Response, ErrorApp> {
    if estado.es_demostracion() {
        return Ok(Redirect::to("/entrar").into_response());
    }
    let demo = estado.demostracion().ok_or(ErrorApp::NoEncontrado)?;
    asegurar_rellena(demo).await?;
    let mut cookies = vec![cookie_de_entrada(), cookie_de_cierre()];
    if let Some(token) = leer_cookie_de_sesion(&cabeceras) {
        cookies.push(cookie_para_volver(&token));
    }
    Ok(con_cookies(&cookies, Redirect::to("/entrar")))
}

/// Vuelve a la sesión de verdad que había (si ya caducó, a la entrada).
pub async fn salir(cabeceras: HeaderMap) -> Response {
    let sesion = match leer_cookie(&cabeceras, COOKIE_VOLVER) {
        Some(token) => cookie_de_sesion(&token),
        None => cookie_de_cierre(),
    };
    let cookies = [cookie_de_salida(), cookie_para_volver_borrada(), sesion];
    con_cookies(&cookies, Redirect::to("/"))
}

pub async fn empezar_otra_vez(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    empezar_de_nuevo(&estado, &actor).await?;
    Ok(con_cookies(&[cookie_de_cierre()], Redirect::to("/entrar")))
}

fn con_cookies(cookies: &[String], destino: Redirect) -> Response {
    let mut respuesta = destino.into_response();
    for cookie in cookies {
        if let Ok(valor) = HeaderValue::from_str(cookie) {
            respuesta.headers_mut().append(SET_COOKIE, valor);
        }
    }
    respuesta
}
