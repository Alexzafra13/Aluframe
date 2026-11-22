//! Rutas del propietario para los trabajos: tablero, lista, ficha y sus acciones.

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::{Accion, Rol};
use dominio::taller::EstadoDeTrabajo;
use serde::Deserialize;

use crate::aplicacion::trabajos::acciones::{
    anotar_falta, asignar_trabajo, asignar_ventana, cerrar_trabajo, marcar_cristal,
};
use crate::aplicacion::trabajos::consultar::{cargar_trabajo, listar_trabajos};
use crate::aplicacion::trabajos::medidas::{
    NuevasMedidas, cm_texto, modificar_medidas, preparar_revision, previsualizar,
};
use crate::aplicacion::trabajos::orden::cargar_orden;
use crate::aplicacion::usuarios::listar_usuarios;
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    Comparacion, MENU, Opcion, OpcionEnPantalla, PaginaInicio, PaginaMedidas, PaginaOrden,
    PaginaTrabajo, PaginaTrabajos, TarjetaTrabajo, renderizar, tablero,
};

pub async fn mostrar_inicio(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    if actor.rol == Rol::Operario {
        return Ok(Redirect::to("/mi-dia").into_response());
    }
    autorizar(&actor, Accion::VerPanelDelPropietario)?;
    let fichas = listar_trabajos(&estado, &actor, false).await?;
    let (columnas, avisos) = tablero(&fichas);
    let pagina = PaginaInicio {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "inicio",
        columnas,
        avisos,
    };
    Ok(renderizar(&pagina)?.into_response())
}

#[derive(Deserialize)]
pub struct FiltroTrabajos {
    cerrados: Option<String>,
}

pub async fn mostrar_trabajos(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(filtro): Query<FiltroTrabajos>,
) -> Result<Response, ErrorApp> {
    let cerrados = filtro.cerrados.is_some();
    let fichas = listar_trabajos(&estado, &actor, cerrados).await?;
    let pagina = PaginaTrabajos {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "trabajos",
        tarjetas: fichas.iter().map(TarjetaTrabajo::from).collect(),
        cerrados,
    };
    Ok(renderizar(&pagina)?.into_response())
}

#[derive(Deserialize)]
pub struct Confirmacion {
    aviso: Option<String>,
}

/// Claves fijas en la dirección, nunca texto libre (como en los presupuestos).
fn texto_de_confirmacion(clave: &str) -> Option<String> {
    let texto = match clave {
        "guardado" => "Trabajo guardado.",
        "asignada" => "Ventana asignada.",
        "cristal" => "Cristal anotado.",
        "aviso" => "Aviso quitado.",
        "entregado" => "Trabajo entregado.",
        "anulado" => "Trabajo anulado.",
        "medidas" => "Medidas cambiadas: la orden de taller lleva la revisión nueva.",
        _ => return None,
    };
    Some(texto.to_owned())
}

async fn operarios(estado: &EstadoApp, actor: &Actor) -> Result<Vec<Opcion>, ErrorApp> {
    Ok(listar_usuarios(estado, actor)
        .await?
        .into_iter()
        .filter(|u| u.activo)
        .map(|u| Opcion {
            valor: u.id.to_string(),
            texto: u.nombre,
        })
        .collect())
}

async fn pagina_del_trabajo(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    (aviso, confirmacion): (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let ficha = cargar_trabajo(estado, actor, id).await?;
    let lista = operarios(estado, actor).await?;
    let codigo = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaTrabajo::nueva(actor.into(), (&ficha, lista), (aviso, confirmacion));
    Ok((codigo, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_trabajo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Query(c): Query<Confirmacion>,
) -> Result<Response, ErrorApp> {
    let confirmacion = c.aviso.as_deref().and_then(texto_de_confirmacion);
    pagina_del_trabajo(&estado, &actor, id, (None, confirmacion)).await
}

/// Tras una acción: vuelve a la ficha con su confirmación, o la muestra con el aviso.
async fn tras(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    (resultado, clave): (Result<(), ErrorApp>, &str),
) -> Result<Response, ErrorApp> {
    match resultado {
        Ok(()) => Ok(Redirect::to(&format!("/trabajos/{id}?aviso={clave}")).into_response()),
        Err(error @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
            pagina_del_trabajo(estado, actor, id, (Some(error.to_string()), None)).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn guardar_datos(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let version = campos.entero("version").unwrap_or(0);
    let fecha = Some(campos.texto("fecha")).filter(|f| !f.trim().is_empty());
    let datos = (campos.entero("operario"), fecha);
    let resultado = asignar_trabajo(&estado, &actor, (id, version), datos).await;
    tras(&estado, &actor, id, (resultado, "guardado")).await
}

pub async fn asignar_la_ventana(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, ventana)): Path<(i64, i64)>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let operario = Campos::de(pares).entero("operario");
    let resultado = asignar_ventana(&estado, &actor, ventana, operario).await;
    tras(&estado, &actor, id, (resultado, "asignada")).await
}

pub async fn anotar_cristal(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, ventana)): Path<(i64, i64)>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let recibido = Campos::de(pares).texto("recibido") == "si";
    let resultado = marcar_cristal(&estado, &actor, ventana, recibido).await;
    tras(&estado, &actor, id, (resultado, "cristal")).await
}

pub async fn quitar_falta(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, ventana)): Path<(i64, i64)>,
) -> Result<Response, ErrorApp> {
    let resultado = anotar_falta(&estado, &actor, ventana, None).await;
    tras(&estado, &actor, id, (resultado, "aviso")).await
}

pub async fn entregar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let resultado = cerrar_trabajo(&estado, &actor, id, EstadoDeTrabajo::Entregado).await;
    tras(&estado, &actor, id, (resultado, "entregado")).await
}

pub async fn anular(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let resultado = if Campos::de(pares).texto("confirmar") == "si" {
        cerrar_trabajo(&estado, &actor, id, EstadoDeTrabajo::Anulado).await
    } else {
        Err(ErrorApp::Validacion(
            "Marca la casilla para confirmar que se anula.".into(),
        ))
    };
    tras(&estado, &actor, id, (resultado, "anulado")).await
}

pub async fn mostrar_orden(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let orden = cargar_orden(&estado, &actor, id).await?;
    Ok(renderizar(&PaginaOrden::de(&orden))?.into_response())
}

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
