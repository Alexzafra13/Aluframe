use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::aplicacion::primer_arranque::{DatosDelPropietario, crear_propietario_inicial};
use crate::autenticacion::cookie_de_sesion;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::vistas::{PaginaPrimerArranque, renderizar};

#[derive(Deserialize)]
pub struct FormularioDelPropietario {
    nombre: String,
    contrasena: String,
    confirmacion: String,
}

pub async fn mostrar_primer_arranque(
    State(estado): State<EstadoApp>,
) -> Result<Response, ErrorApp> {
    if estado.hay_usuarios() {
        return Ok(Redirect::to("/entrar").into_response());
    }
    let pagina = PaginaPrimerArranque {
        nombre: String::new(),
        aviso: None,
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn procesar_primer_arranque(
    State(estado): State<EstadoApp>,
    Form(formulario): Form<FormularioDelPropietario>,
) -> Result<Response, ErrorApp> {
    let nombre = formulario.nombre.clone();
    let datos = DatosDelPropietario {
        nombre: formulario.nombre,
        contrasena: formulario.contrasena,
        confirmacion: formulario.confirmacion,
    };
    match crear_propietario_inicial(&estado, datos).await {
        Ok(token) => {
            let cookie = [(SET_COOKIE, cookie_de_sesion(&token))];
            Ok((cookie, Redirect::to("/")).into_response())
        }
        Err(ErrorApp::Validacion(aviso)) => {
            let pagina = PaginaPrimerArranque {
                nombre,
                aviso: Some(aviso),
            };
            Ok((StatusCode::UNPROCESSABLE_ENTITY, renderizar(&pagina)?).into_response())
        }
        Err(otro) => Err(otro),
    }
}
