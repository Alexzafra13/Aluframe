use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::{IntoResponse, Response};

use crate::autenticacion::generar_token;
use crate::errores::ErrorApp;
use crate::vistas::{PaginaError, renderizar};

impl IntoResponse for ErrorApp {
    fn into_response(self) -> Response {
        let (estado, titulo, codigo) = match &self {
            ErrorApp::SinSesion => {
                return (StatusCode::SEE_OTHER, [(LOCATION, "/entrar")]).into_response();
            }
            ErrorApp::Validacion(_) => (StatusCode::UNPROCESSABLE_ENTITY, "Revisa los datos", None),
            ErrorApp::NoEncontrado => (StatusCode::NOT_FOUND, "No encontrado", None),
            ErrorApp::Conflicto => (StatusCode::CONFLICT, "Cambio de otra persona", None),
            ErrorApp::Bloqueado { .. } => {
                (StatusCode::TOO_MANY_REQUESTS, "Espera un momento", None)
            }
            ErrorApp::Interno(detalle) => {
                let codigo = codigo_de_correlacion();
                tracing::error!(codigo = %codigo, "{detalle}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Algo ha fallado",
                    Some(codigo),
                )
            }
        };
        let mensaje = match self {
            ErrorApp::Interno(_) => "Avisa a quien mantiene el programa con este código.".into(),
            otro => otro.to_string(),
        };
        let pagina = PaginaError {
            titulo,
            mensaje,
            codigo,
        };
        match renderizar(&pagina) {
            Ok(html) => (estado, html).into_response(),
            Err(_) => estado.into_response(),
        }
    }
}

fn codigo_de_correlacion() -> String {
    generar_token()
        .map(|token| token.chars().take(8).collect())
        .unwrap_or_else(|_| "sin-codigo".into())
}
