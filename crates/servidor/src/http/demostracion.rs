//! Desvío a la demostración (ADR-0028): con su cookie, cada petición de ese navegador va al
//! enrutador de la base de la demostración; sin ella, al de la base de verdad. Así cada equipo
//! entra y sale por su cuenta y los demás siguen trabajando con los datos reales.

use std::net::SocketAddr;

use axum::Router;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{Extensions, HeaderMap};
use axum::middleware::Next;
use axum::response::Response;
use tower::ServiceExt;

use crate::autenticacion::leer_cookie;

const COOKIE_DEMOSTRACION: &str = "alumframe_demostracion";
/// La sesión de verdad mientras se está en la demostración, para volver a ella al salir.
pub const COOKIE_VOLVER: &str = "alumframe_volver";

pub async fn desviar(State(demo): State<Router>, peticion: Request, siguiente: Next) -> Response {
    // La pantalla del escritorio para elegir dónde trabajar es siempre la de este equipo.
    let del_escritorio = peticion.uri().path().starts_with("/escritorio/");
    if del_escritorio || !esta_en_la_demostracion(peticion.headers()) {
        return siguiente.run(peticion).await;
    }
    match demo.oneshot(limpia(peticion)).await {
        Ok(respuesta) => respuesta,
        Err(nunca) => match nunca {},
    }
}

/// La petición tal como llegó: el enrutador de la base de verdad ya le apuntó los parámetros de
/// su ruta y, si se reenvía así, el de la demostración los duplica. Solo se conserva quién se
/// conecta (para el bloqueo por IP).
fn limpia(peticion: Request) -> Request {
    let (mut partes, cuerpo) = peticion.into_parts();
    let conexion = partes.extensions.get::<ConnectInfo<SocketAddr>>().cloned();
    partes.extensions = Extensions::new();
    if let Some(conexion) = conexion {
        partes.extensions.insert(conexion);
    }
    Request::from_parts(partes, cuerpo)
}

pub fn esta_en_la_demostracion(cabeceras: &HeaderMap) -> bool {
    leer_cookie(cabeceras, COOKIE_DEMOSTRACION).as_deref() == Some("si")
}

pub fn cookie_de_entrada() -> String {
    format!("{COOKIE_DEMOSTRACION}=si; HttpOnly; SameSite=Lax; Path=/")
}

pub fn cookie_de_salida() -> String {
    format!("{COOKIE_DEMOSTRACION}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
}

pub fn cookie_para_volver(token: &str) -> String {
    format!("{COOKIE_VOLVER}={token}; HttpOnly; SameSite=Lax; Path=/")
}

pub fn cookie_para_volver_borrada() -> String {
    format!("{COOKIE_VOLVER}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
}
