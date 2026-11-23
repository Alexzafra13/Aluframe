use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Accion;
use serde::Deserialize;

use crate::aplicacion::medidas_habituales_ajustes::{
    anadir_medida_habitual, quitar_medida_habitual, ver_medidas_habituales,
};
use crate::aplicacion::taller::{
    TextosDeTaller, guardar_ajustes_de_taller, guardar_metodo_de_corte, leer_ajustes_de_taller,
};
use crate::aplicacion::trabajos::crear::{guardar_pasos_del_taller, leer_pasos_del_taller};
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{MENU, PaginaTaller, renderizar};

const GUARDADO: &str = "/ajustes/taller?guardado=si";

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
    let textos = textos_guardados(&estado).await?;
    let confirmacion = guardado
        .guardado
        .map(|_| "Ajustes del taller guardados.".to_owned());
    pagina(&estado, &actor, textos, (None, confirmacion)).await
}

#[derive(Deserialize)]
pub struct FormularioMetodo {
    metodo: String,
}

pub async fn guardar_metodo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(formulario): Form<FormularioMetodo>,
) -> Result<Response, ErrorApp> {
    guardar_metodo_de_corte(&estado, &actor, formulario.metodo).await?;
    Ok(Redirect::to(GUARDADO).into_response())
}

#[derive(Deserialize)]
pub struct FormularioHabitual {
    tipologia: String,
    ancho: String,
    alto: String,
}

pub async fn anadir_habitual(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(f): Form<FormularioHabitual>,
) -> Result<Response, ErrorApp> {
    match anadir_medida_habitual(&estado, &actor, (f.tipologia, f.ancho, f.alto)).await {
        Ok(()) => Ok(Redirect::to(GUARDADO).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            let textos = textos_guardados(&estado).await?;
            let respuesta = pagina(&estado, &actor, textos, (Some(aviso), None)).await?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, respuesta).into_response())
        }
        Err(otro) => Err(otro),
    }
}

pub async fn quitar_habitual(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(indice): Path<usize>,
) -> Result<Response, ErrorApp> {
    quitar_medida_habitual(&estado, &actor, indice).await?;
    Ok(Redirect::to(GUARDADO).into_response())
}

#[derive(Deserialize)]
pub struct FormularioPasos {
    pasos: String,
}

pub async fn guardar_pasos(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(formulario): Form<FormularioPasos>,
) -> Result<Response, ErrorApp> {
    guardar_pasos_del_taller(&estado, &actor, formulario.pasos).await?;
    Ok(Redirect::to(GUARDADO).into_response())
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
        Ok(()) => Ok(Redirect::to(GUARDADO).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            let respuesta = pagina(&estado, &actor, copia, (Some(aviso), None)).await?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, respuesta).into_response())
        }
        Err(otro) => Err(otro),
    }
}

async fn textos_guardados(estado: &EstadoApp) -> Result<TextosDeTaller, ErrorApp> {
    let ajustes = leer_ajustes_de_taller(estado).await?;
    Ok(TextosDeTaller {
        grosor_sierra_mm: ajustes.grosor_sierra.formatear_mm(),
        recorte_extremos_mm: ajustes.recorte_extremos.formatear_mm(),
        retal_minimo_cm: ajustes.retal_minimo.formatear_cm(),
    })
}

fn clonar(textos: &TextosDeTaller) -> TextosDeTaller {
    TextosDeTaller {
        grosor_sierra_mm: textos.grosor_sierra_mm.clone(),
        recorte_extremos_mm: textos.recorte_extremos_mm.clone(),
        retal_minimo_cm: textos.retal_minimo_cm.clone(),
    }
}

/// La página con los textos del formulario de corte (los guardados o los que no se aceptaron)
/// y lo demás leído de la base: método de reparto, medidas habituales y pasos.
async fn pagina(
    estado: &EstadoApp,
    actor: &Actor,
    textos: TextosDeTaller,
    (aviso, confirmacion): (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let ajustes = leer_ajustes_de_taller(estado).await?;
    let habituales = ver_medidas_habituales(estado, actor).await?;
    let pagina = PaginaTaller {
        usuario: actor.into(),
        menu: MENU,
        seccion: "ajustes",
        pestana: "taller",
        grosor_sierra_mm: textos.grosor_sierra_mm,
        recorte_extremos_mm: textos.recorte_extremos_mm,
        retal_minimo_cm: textos.retal_minimo_cm,
        metodo: ajustes.metodo.como_texto(),
        habituales: habituales.medidas,
        tipologias: habituales.tipologias,
        pasos: leer_pasos_del_taller(estado).await?.join("\n"),
        provisional: aviso.is_some() || !ajustes.confirmados,
        aviso,
        confirmacion,
    };
    Ok(renderizar(&pagina)?.into_response())
}
