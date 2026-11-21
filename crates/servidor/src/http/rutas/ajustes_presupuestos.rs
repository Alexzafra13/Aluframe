use axum::Form;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Accion;
use serde::Deserialize;

use crate::aplicacion::ajustes_presupuesto::{
    CLAVES, TextosDeAjustes, guardar_ajustes_de_presupuesto, leer_ajustes_de_presupuesto,
};
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
    textos: TextosDeAjustes,
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
    let confirmacion = g.guardado.map(|_| "Ajustes guardados.".to_owned());
    pagina(&actor, ajustes.textos, (None, confirmacion))
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
        Err(ErrorApp::Validacion(aviso)) => pagina(&actor, textos, (Some(aviso), None)),
        Err(otro) => Err(otro),
    }
}
