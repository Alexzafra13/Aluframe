use axum::Form;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Accion;
use serde::Deserialize;

use crate::aplicacion::taller::{
    TextosDeTaller, guardar_ajustes_de_taller, leer_ajustes_de_taller,
};
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{MENU, PaginaTaller, renderizar};

#[derive(Deserialize)]
pub struct Guardado {
    guardado: Option<String>,
}

#[derive(Deserialize)]
pub struct FormularioTaller {
    grosor_sierra_mm: String,
    recorte_extremos_mm: String,
    retal_minimo_cm: String,
}

pub async fn mostrar_taller(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(guardado): Query<Guardado>,
) -> Result<Response, ErrorApp> {
    autorizar(&actor, Accion::GestionarCatalogo)?;
    let ajustes = leer_ajustes_de_taller(&estado).await?;
    let textos = TextosDeTaller {
        grosor_sierra_mm: ajustes.grosor_sierra.formatear_mm(),
        recorte_extremos_mm: ajustes.recorte_extremos.formatear_mm(),
        retal_minimo_cm: ajustes.retal_minimo.formatear_cm(),
    };
    let confirmacion = guardado
        .guardado
        .map(|_| "Ajustes del taller guardados.".to_owned());
    pagina(&actor, textos, (!ajustes.confirmados, None, confirmacion))
}

pub async fn guardar_taller(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(formulario): Form<FormularioTaller>,
) -> Result<Response, ErrorApp> {
    let textos = TextosDeTaller {
        grosor_sierra_mm: formulario.grosor_sierra_mm,
        recorte_extremos_mm: formulario.recorte_extremos_mm,
        retal_minimo_cm: formulario.retal_minimo_cm,
    };
    let copia = clonar(&textos);
    match guardar_ajustes_de_taller(&estado, &actor, textos).await {
        Ok(()) => Ok(Redirect::to("/ajustes/taller?guardado=si").into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            let respuesta = pagina(&actor, copia, (true, Some(aviso), None))?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, respuesta).into_response())
        }
        Err(otro) => Err(otro),
    }
}

fn clonar(textos: &TextosDeTaller) -> TextosDeTaller {
    TextosDeTaller {
        grosor_sierra_mm: textos.grosor_sierra_mm.clone(),
        recorte_extremos_mm: textos.recorte_extremos_mm.clone(),
        retal_minimo_cm: textos.retal_minimo_cm.clone(),
    }
}

type Mensajes = (bool, Option<String>, Option<String>);

fn pagina(
    actor: &Actor,
    textos: TextosDeTaller,
    (provisional, aviso, confirmacion): Mensajes,
) -> Result<Response, ErrorApp> {
    let pagina = PaginaTaller {
        usuario: actor.into(),
        menu: MENU,
        seccion: "ajustes",
        pestana: "taller",
        grosor_sierra_mm: textos.grosor_sierra_mm,
        recorte_extremos_mm: textos.recorte_extremos_mm,
        retal_minimo_cm: textos.retal_minimo_cm,
        provisional,
        aviso,
        confirmacion,
    };
    Ok(renderizar(&pagina)?.into_response())
}
