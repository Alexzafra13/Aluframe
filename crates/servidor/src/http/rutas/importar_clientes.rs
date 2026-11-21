use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::clientes::COLUMNAS_DE_CLIENTES;
use serde::Deserialize;

use super::clientes::SECCION;
use crate::aplicacion::Actor;
use crate::aplicacion::importar_clientes::{
    PrevisualizacionDeClientes, deshacer_importacion_de_clientes, importar_clientes,
    previsualizar_clientes, ultimas_importaciones,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{ImportacionEnLista, MENU, PaginaImportarClientes, renderizar};

#[derive(Deserialize)]
pub struct ArchivoRevisado {
    texto: String,
    sha256: Option<String>,
}

async fn pagina(
    estado: &EstadoApp,
    actor: &Actor,
    texto: String,
    (previsualizacion, aviso): (Option<PrevisualizacionDeClientes>, Option<String>),
) -> Result<Response, ErrorApp> {
    let importaciones = ultimas_importaciones(estado, actor, "clientes").await?;
    let con_problemas = aviso.is_some()
        || previsualizacion
            .as_ref()
            .is_some_and(|p| !p.errores.is_empty());
    let pagina = PaginaImportarClientes {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        texto,
        previsualizacion,
        importaciones: importaciones.iter().map(ImportacionEnLista::from).collect(),
        aviso,
        plantilla: COLUMNAS_DE_CLIENTES.join(";"),
    };
    let estado_http = if con_problemas {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

pub async fn mostrar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    pagina(&estado, &actor, String::new(), (None, None)).await
}

pub async fn revisar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(archivo): Form<ArchivoRevisado>,
) -> Result<Response, ErrorApp> {
    let vista = previsualizar_clientes(&estado, &actor, archivo.texto.clone()).await?;
    pagina(&estado, &actor, archivo.texto, (Some(vista), None)).await
}

pub async fn confirmar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(archivo): Form<ArchivoRevisado>,
) -> Result<Response, ErrorApp> {
    let sha = archivo.sha256.unwrap_or_default();
    match importar_clientes(&estado, &actor, archivo.texto.clone(), sha).await {
        Ok((importacion, n)) => {
            let destino = format!("/clientes?importados={n}&importacion={importacion}");
            Ok(Redirect::to(&destino).into_response())
        }
        Err(ErrorApp::Validacion(aviso)) => {
            let vista = previsualizar_clientes(&estado, &actor, archivo.texto.clone()).await?;
            pagina(&estado, &actor, archivo.texto, (Some(vista), Some(aviso))).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn deshacer(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let (quitados, conservados) = deshacer_importacion_de_clientes(&estado, &actor, id).await?;
    let destino = format!("/clientes?quitados={quitados}&conservados={conservados}");
    Ok(Redirect::to(&destino).into_response())
}
