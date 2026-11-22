//! «Modificar medidas» de una ventana del trabajo (PANTALLAS 8): ver lo que cambia y confirmar.

use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

use crate::aplicacion::Actor;
use crate::aplicacion::trabajos::medidas::{
    NuevasMedidas, cm_texto, modificar_medidas, preparar_revision, previsualizar,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{Comparacion, MENU, OpcionEnPantalla, PaginaMedidas, renderizar};

struct FormularioDeMedidas {
    version: i64,
    medidas: NuevasMedidas,
    confirmar: bool,
}

fn leer_formulario_de_medidas(pares: Vec<(String, String)>) -> FormularioDeMedidas {
    let opciones = pares
        .iter()
        .filter_map(|(c, v)| c.strip_prefix("opcion.").map(|c| (c.to_owned(), v.clone())))
        .collect();
    let campos = Campos::de(pares);
    FormularioDeMedidas {
        version: campos.entero("version").unwrap_or(0),
        confirmar: campos.texto("accion") == "confirmar",
        medidas: NuevasMedidas {
            ancho: campos.texto("ancho"),
            alto: campos.texto("alto"),
            opciones,
            motivo: campos.texto("motivo"),
            reiniciar_pasos: campos.texto("reiniciar") == "si",
        },
    }
}

async fn pagina_de_medidas(
    estado: &EstadoApp,
    actor: &Actor,
    (trabajo, ventana): (i64, i64),
    (formulario, comparacion, aviso): (Option<&NuevasMedidas>, Option<Comparacion>, Option<String>),
) -> Result<Response, ErrorApp> {
    let r = preparar_revision(estado, actor, ventana).await?;
    let c = &r.ventana.congelado;
    let elegidas = formulario.map_or(&c.opciones, |m| &m.opciones);
    let codigo = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaMedidas {
        usuario: actor.into(),
        menu: MENU,
        seccion: "trabajos",
        trabajo,
        ventana,
        version: r.ventana.fila.version,
        titulo: r.ventana.fila.descripcion.clone(),
        revision: r.ventana.fila.revision,
        ancho: formulario.map_or_else(|| cm_texto(c.ancho_dmm), |m| m.ancho.clone()),
        alto: formulario.map_or_else(|| cm_texto(c.alto_dmm), |m| m.alto.clone()),
        opciones: r
            .opciones
            .iter()
            .map(|o| OpcionEnPantalla::de(o, elegidas.get(&o.codigo)))
            .collect(),
        motivo: formulario.map(|m| m.motivo.clone()).unwrap_or_default(),
        reiniciar: formulario.is_some_and(|m| m.reiniciar_pasos),
        pasos_hechos: r.pasos_hechos,
        comparacion,
        aviso,
    };
    Ok((codigo, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_medidas(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, ventana)): Path<(i64, i64)>,
) -> Result<Response, ErrorApp> {
    pagina_de_medidas(&estado, &actor, (id, ventana), (None, None, None)).await
}

pub async fn guardar_medidas(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, ventana)): Path<(i64, i64)>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let f = leer_formulario_de_medidas(pares);
    let resultado = if f.confirmar {
        modificar_medidas(&estado, &actor, (ventana, f.version), f.medidas.clone())
            .await
            .map(|_| None)
    } else {
        previsualizar(&estado, &actor, ventana, f.medidas.clone())
            .await
            .map(Some)
    };
    match resultado {
        Ok(None) => Ok(Redirect::to(&format!("/trabajos/{id}?aviso=medidas")).into_response()),
        Ok(Some((antes, despues))) => {
            let comparacion = Some(Comparacion::de(&antes, &despues));
            pagina_de_medidas(
                &estado,
                &actor,
                (id, ventana),
                (Some(&f.medidas), comparacion, None),
            )
            .await
        }
        Err(error @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
            let aviso = Some(error.to_string());
            pagina_de_medidas(
                &estado,
                &actor,
                (id, ventana),
                (Some(&f.medidas), None, aviso),
            )
            .await
        }
        Err(otro) => Err(otro),
    }
}
