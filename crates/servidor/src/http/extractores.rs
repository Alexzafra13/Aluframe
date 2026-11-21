use std::net::SocketAddr;

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;

use crate::aplicacion::Actor;
use crate::errores::ErrorApp;

pub struct ActorActual(pub Actor);

impl<S: Send + Sync> FromRequestParts<S> for ActorActual {
    type Rejection = ErrorApp;

    async fn from_request_parts(partes: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        partes
            .extensions
            .get::<Actor>()
            .cloned()
            .map(ActorActual)
            .ok_or(ErrorApp::SinSesion)
    }
}

pub struct IpCliente(pub String);

impl<S: Send + Sync> FromRequestParts<S> for IpCliente {
    type Rejection = ErrorApp;

    async fn from_request_parts(partes: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let ip = partes
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map_or_else(|| "desconocida".to_owned(), |info| info.0.ip().to_string());
        Ok(IpCliente(ip))
    }
}
