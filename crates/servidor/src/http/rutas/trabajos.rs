//! Rutas del propietario para los trabajos: tablero, lista, ficha y sus acciones.

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::taller::EstadoDeTrabajo;
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::facturas::cobros::MEDIOS;
use crate::aplicacion::facturas::documentos_de_trabajo;
use crate::aplicacion::trabajos::acciones::{
    anotar_falta, asignar_trabajo, asignar_ventana, cerrar_trabajo, marcar_cristal,
};
use crate::aplicacion::trabajos::consultar::{cargar_trabajo, listar_trabajos};
use crate::aplicacion::trabajos::orden::cargar_orden;
use crate::aplicacion::usuarios::listar_usuarios;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fecha_de_madrid;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    CobroEnPantalla, DocumentosEnPantalla, MENU, Opcion, PaginaOrden, PaginaTrabajo,
    PaginaTrabajos, TarjetaTrabajo, estado_de_factura, euros, renderizar,
};

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
        "terminada" => "Orden terminada: el stock está descontado.",
        "senal" => "Señal registrada: contará como pagado al emitir la factura.",
        "repetida" => "Pieza repetida anotada: entra en el plan y se descontará al terminar.",
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

async fn documentos_en_pantalla(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<DocumentosEnPantalla, ErrorApp> {
    let d = documentos_de_trabajo(estado, actor, id).await?;
    let hoy = fecha_de_madrid(estado.ahora(), 0);
    let cobrado: i64 = d
        .senales
        .iter()
        .filter(|c| !c.anulado)
        .map(|c| c.importe_cent)
        .sum();
    Ok(DocumentosEnPantalla {
        senales: d.senales.iter().map(CobroEnPantalla::from).collect(),
        cobrado: euros(cobrado),
        facturas: d
            .facturas
            .iter()
            .map(|f| (f.id, f.codigo(), estado_de_factura(f, &hoy)))
            .collect(),
        albaranes: d.albaranes.iter().map(|a| (a.id, a.codigo())).collect(),
        facturada: d
            .facturas
            .iter()
            .any(|f| f.estado == "emitida" && f.tipo == "ordinaria"),
        medios: MEDIOS
            .iter()
            .map(|(v, t)| Opcion {
                valor: (*v).to_owned(),
                texto: (*t).to_owned(),
            })
            .collect(),
        hoy,
    })
}

pub(crate) async fn pagina_del_trabajo(
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
    let mut pagina = PaginaTrabajo::nueva(actor.into(), (&ficha, lista), (aviso, confirmacion));
    pagina.documentos = documentos_en_pantalla(estado, actor, id).await?;
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
