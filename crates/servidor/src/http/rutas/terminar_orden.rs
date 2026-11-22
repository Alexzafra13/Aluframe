//! «Terminar orden y descontar stock» desde la ficha del trabajo (PANTALLAS 8, SPEC H5).

use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::unidades::{Longitud, Unidad, leer_decimal_es};

use crate::aplicacion::Actor;
use crate::aplicacion::stock::terminar::{RetalConfirmado, preparar_terminar, terminar_orden};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{PaginaTerminar, renderizar};

async fn pagina(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    aviso: Option<String>,
) -> Result<Response, ErrorApp> {
    let (orden, prevision) = preparar_terminar(estado, actor, id).await?;
    let codigo = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaTerminar::nueva(actor.into(), (&orden, &prevision), aviso);
    Ok((codigo, renderizar(&pagina)?).into_response())
}

pub async fn mostrar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    pagina(&estado, &actor, id, None).await
}

fn largo(texto: &str) -> Result<i64, ErrorApp> {
    leer_decimal_es(texto.trim())
        .ok()
        .and_then(|d| Longitud::desde_decimal(d, Unidad::Cm).ok())
        .map(Longitud::decimas_de_mm)
        .filter(|l| *l > 0)
        .ok_or_else(|| {
            ErrorApp::Validacion(format!("El largo «{texto}» no es válido: escríbelo en cm."))
        })
}

/// Los retales marcados (con su largo, quizá corregido) y los añadidos a mano.
fn retales_confirmados(campos: &Campos) -> Result<Vec<RetalConfirmado>, ErrorApp> {
    let mut retales = Vec::new();
    for i in 0..200 {
        let Some(grupo) = campos.entero(&format!("grupo_{i}")) else {
            break;
        };
        if campos.texto(&format!("guardar_{i}")) == "si" {
            let largo_dmm = largo(&campos.texto(&format!("largo_{i}")))?;
            retales.push(RetalConfirmado {
                grupo: usize::try_from(grupo).unwrap_or(usize::MAX),
                largo_dmm,
            });
        }
    }
    for n in 0..2 {
        let (grupo, texto) = (
            campos.entero(&format!("extra_grupo_{n}")),
            campos.texto(&format!("extra_largo_{n}")),
        );
        if let (Some(grupo), false) = (grupo, texto.trim().is_empty()) {
            retales.push(RetalConfirmado {
                grupo: usize::try_from(grupo).unwrap_or(usize::MAX),
                largo_dmm: largo(&texto)?,
            });
        }
    }
    Ok(retales)
}

pub async fn confirmar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let resultado = match retales_confirmados(&Campos::de(pares)) {
        Ok(retales) => terminar_orden(&estado, &actor, id, retales).await,
        Err(error) => Err(error),
    };
    match resultado {
        Ok(()) => Ok(Redirect::to(&format!("/trabajos/{id}?aviso=terminada")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => pagina(&estado, &actor, id, Some(aviso)).await,
        Err(otro) => Err(otro),
    }
}
