//! Pantalla de la aplicación de escritorio para elegir dónde trabajar (ADR-0015). Solo existe
//! cuando el programa se abre como aplicación; en el servidor del taller da 404.

use std::path::Path;

use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::errores::ErrorApp;
use crate::escritorio::preferencias::{Conexion, Preferencias, validar_direccion};
use crate::estado::EstadoApp;
use crate::vistas::{PaginaConectando, PaginaConexion, renderizar};

#[derive(Deserialize)]
pub struct Eleccion {
    conexion: String,
    #[serde(default)]
    servidor: String,
}

pub async fn mostrar_conexion(State(estado): State<EstadoApp>) -> Result<Response, ErrorApp> {
    let archivo = archivo_de_preferencias(&estado)?;
    let preferencias = Preferencias::cargar(archivo);
    let pagina = PaginaConexion {
        servidor: preferencias.servidor.unwrap_or_default(),
        en_servidor: preferencias.conexion == Some(Conexion::Servidor),
        aviso: None,
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn elegir_conexion(
    State(estado): State<EstadoApp>,
    Form(eleccion): Form<Eleccion>,
) -> Result<Response, ErrorApp> {
    let archivo = archivo_de_preferencias(&estado)?;
    let mut preferencias = Preferencias::cargar(archivo);
    if eleccion.conexion != "servidor" {
        preferencias.conexion = Some(Conexion::EsteEquipo);
        guardar(&preferencias, archivo)?;
        return Ok(Redirect::to("/").into_response());
    }
    match validar_direccion(&eleccion.servidor) {
        Ok(direccion) => {
            preferencias.conexion = Some(Conexion::Servidor);
            preferencias.servidor = Some(direccion.clone());
            guardar(&preferencias, archivo)?;
            Ok(renderizar(&PaginaConectando { direccion })?.into_response())
        }
        Err(aviso) => {
            let pagina = PaginaConexion {
                servidor: eleccion.servidor,
                en_servidor: true,
                aviso: Some(aviso),
            };
            Ok((StatusCode::UNPROCESSABLE_ENTITY, renderizar(&pagina)?).into_response())
        }
    }
}

fn archivo_de_preferencias(estado: &EstadoApp) -> Result<&Path, ErrorApp> {
    estado
        .preferencias_escritorio()
        .ok_or(ErrorApp::NoEncontrado)
}

fn guardar(preferencias: &Preferencias, archivo: &Path) -> Result<(), ErrorApp> {
    preferencias
        .guardar(archivo)
        .map_err(|error| ErrorApp::Interno(format!("preferencias del escritorio: {error}")))
}
