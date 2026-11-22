use axum::Form;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Accion;
use serde::Deserialize;

use crate::aplicacion::ajustes_presupuesto::{
    CLAVES, TextosDeAjustes, guardar_ajustes_de_presupuesto, leer_ajustes_de_presupuesto,
};
use crate::aplicacion::logo_empresa::{cambiar_logo, leer_logo_de_empresa};
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{MENU, PaginaAjustesPresupuestos, renderizar};

#[derive(Deserialize)]
pub struct Guardado {
    guardado: Option<String>,
}

fn pagina(
    actor: &Actor,
    (textos, logo): (TextosDeAjustes, Option<String>),
    (aviso, confirmacion): (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let estado_http = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaAjustesPresupuestos {
        usuario: actor.into(),
        menu: MENU,
        seccion: "ajustes",
        textos,
        logo,
        aviso,
        confirmacion,
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

pub async fn mostrar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(g): Query<Guardado>,
) -> Result<Response, ErrorApp> {
    autorizar(&actor, Accion::GestionarUsuarios)?;
    let ajustes = leer_ajustes_de_presupuesto(&estado).await?;
    let logo = leer_logo_de_empresa(&estado).await?;
    let confirmacion = g.guardado.map(|g| {
        match g.as_str() {
            "logo" => "Logotipo guardado.",
            "sin-logo" => "Logotipo quitado: la hoja lleva el nombre de la empresa.",
            _ => "Ajustes guardados.",
        }
        .to_owned()
    });
    pagina(&actor, (ajustes.textos, logo), (None, confirmacion))
}

pub async fn guardar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let textos = TextosDeAjustes(
        CLAVES
            .iter()
            .map(|clave| {
                let valor = pares
                    .iter()
                    .find(|(c, _)| c == clave)
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                ((*clave).to_owned(), valor)
            })
            .collect(),
    );
    match guardar_ajustes_de_presupuesto(&estado, &actor, textos.clone()).await {
        Ok(()) => Ok(Redirect::to("/ajustes/presupuestos?guardado=si").into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            let logo = leer_logo_de_empresa(&estado).await?;
            pagina(&actor, (textos, logo), (Some(aviso), None))
        }
        Err(otro) => Err(otro),
    }
}

/// El navegador envía la imagen tal cual con el token CSRF en la cabecera (como la copia de
/// datos); si no vale, responde con el motivo en texto para mostrarlo junto al botón.
pub async fn subir_logo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    cuerpo: Bytes,
) -> Response {
    match cambiar_logo(&estado, &actor, Some(cuerpo.to_vec())).await {
        Ok(()) => Redirect::to("/ajustes/presupuestos?guardado=logo").into_response(),
        Err(ErrorApp::Validacion(mensaje)) => {
            (StatusCode::UNPROCESSABLE_ENTITY, mensaje).into_response()
        }
        Err(otro) => otro.into_response(),
    }
}

pub async fn quitar_logo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    cambiar_logo(&estado, &actor, None).await?;
    Ok(Redirect::to("/ajustes/presupuestos?guardado=sin-logo").into_response())
}
