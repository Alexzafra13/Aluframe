//! Entrar en la demostración, salir de ella y empezarla de nuevo (ADR-0028).

use axum::Form;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Rol;
use serde::Deserialize;

use crate::aplicacion::demostracion::{
    QuienEntra, asegurar_rellena, empezar_de_nuevo, entrar_sin_clave,
};
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

/// Se guarda la sesión de verdad para volver a ella y se entra en la demostración como su
/// dueño, sin contraseña.
pub async fn entrar(
    State(estado): State<EstadoApp>,
    cabeceras: HeaderMap,
) -> Result<Response, ErrorApp> {
    if estado.es_demostracion() {
        return Ok(Redirect::to("/").into_response());
    }
    let demo = estado.demostracion().ok_or(ErrorApp::NoEncontrado)?;
    asegurar_rellena(demo).await?;
    let (token_demo, _) = entrar_sin_clave(demo, QuienEntra::Dueno).await?;
    let mut cookies = vec![cookie_de_entrada(), cookie_de_sesion(&token_demo)];
    if let Some(token) = leer_cookie_de_sesion(&cabeceras) {
        cookies.push(cookie_para_volver(&token));
    }
    Ok(con_cookies(&cookies, Redirect::to("/")))
}

#[derive(Deserialize)]
pub struct UsuarioElegido {
    usuario_id: i64,
}

/// Dentro de la demostración, se entra con el usuario elegido sin contraseña ni PIN. En la
/// base de verdad no existe (`entrar_sin_clave` responde que no se encuentra).
pub async fn entrar_como(
    State(estado): State<EstadoApp>,
    Form(elegido): Form<UsuarioElegido>,
) -> Result<Response, ErrorApp> {
    let quien = QuienEntra::Usuario(elegido.usuario_id);
    let (token, rol) = entrar_sin_clave(&estado, quien).await?;
    let destino = if rol == Rol::Operario { "/mi-dia" } else { "/" };
    Ok(con_cookies(
        &[cookie_de_sesion(&token)],
        Redirect::to(destino),
    ))
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
    let (token, _) = entrar_sin_clave(&estado, QuienEntra::Dueno).await?;
    Ok(con_cookies(&[cookie_de_sesion(&token)], Redirect::to("/")))
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
