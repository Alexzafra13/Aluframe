//! Rutas de facturas: lista, ficha, hoja impresa y albarán impreso (PANTALLAS 11).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::presupuestos::opciones_de_iva;
use crate::aplicacion::Actor;
use crate::aplicacion::ajustes_presupuesto::leer_ajustes_de_presupuesto;
use crate::aplicacion::clientes::{ConsultaDeClientes, listar_clientes};
use crate::aplicacion::datos_de_empresa::CabeceraDeEmpresa;
use crate::aplicacion::facturas::albaranes::buscar_albaran;
use crate::aplicacion::facturas::{cargar_factura, hoja_de_factura, listar_facturas};
use crate::aplicacion::logo_empresa::leer_logo_de_empresa;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fecha_de_madrid;
use crate::http::extractores::ActorActual;
use crate::vistas::{
    FilaFacturaEnLista, MENU, Opcion, PaginaAlbaranImpreso, PaginaFactura, PaginaFacturas,
    hoja_desde_fotografia, renderizar,
};

#[derive(Deserialize)]
pub struct FiltroFacturas {
    pendientes: Option<String>,
    aviso: Option<String>,
}

async fn clientes_para_elegir(estado: &EstadoApp, actor: &Actor) -> Result<Vec<Opcion>, ErrorApp> {
    let consulta = ConsultaDeClientes {
        texto: String::new(),
        archivados: false,
        pagina: 1,
    };
    let lista = listar_clientes(estado, actor, consulta).await?;
    Ok(lista
        .clientes
        .iter()
        .map(|c| Opcion {
            valor: c.id.to_string(),
            texto: c.nombre_visible(),
        })
        .collect())
}

pub async fn mostrar_facturas(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(filtro): Query<FiltroFacturas>,
) -> Result<Response, ErrorApp> {
    let hoy = fecha_de_madrid(estado.ahora(), 0);
    let pendientes = filtro.pendientes.is_some();
    let mut filas: Vec<FilaFacturaEnLista> = listar_facturas(&estado, &actor)
        .await?
        .iter()
        .map(|f| FilaFacturaEnLista::de(f, &hoy))
        .collect();
    if pendientes {
        filas.retain(|f| matches!(f.estado, "Pendiente" | "Parcial" | "Vencida"));
    }
    let pagina = PaginaFacturas {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "facturas",
        filas,
        pendientes,
        clientes: clientes_para_elegir(&estado, &actor).await?,
    };
    Ok(renderizar(&pagina)?.into_response())
}

fn texto_de_confirmacion(clave: &str) -> Option<String> {
    let texto = match clave {
        "creada" => "Borrador creado.",
        "guardada" => "Guardado.",
        "emitida" => "Factura emitida: ya tiene su número y no se puede modificar.",
        "cobro" => "Cobro registrado.",
        "anulado" => "Cobro anulado.",
        _ => return None,
    };
    Some(texto.to_owned())
}

pub async fn pagina_de_factura(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    (aviso, confirmacion): (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let ficha = cargar_factura(estado, actor, id).await?;
    let ajustes = leer_ajustes_de_presupuesto(estado).await?;
    let ivas = opciones_de_iva(ajustes.iva_general, ajustes.iva_reducido);
    let hoy = fecha_de_madrid(estado.ahora(), 0);
    let codigo = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaFactura::nueva(actor.into(), (&ficha, &hoy, ivas), (aviso, confirmacion));
    Ok((codigo, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_factura(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Query(filtro): Query<FiltroFacturas>,
) -> Result<Response, ErrorApp> {
    let confirmacion = filtro.aviso.as_deref().and_then(texto_de_confirmacion);
    pagina_de_factura(&estado, &actor, id, (None, confirmacion)).await
}

pub async fn imprimir_factura(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let h = hoja_de_factura(&estado, &actor, id).await?;
    let pagina = hoja_desde_fotografia(id, &h.foto, (h.borrador, h.logo), (h.vencimiento, h.notas));
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn imprimir_albaran(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let albaran = buscar_albaran(&estado, &actor, id).await?;
    let textos = leer_ajustes_de_presupuesto(&estado).await?.textos;
    let empresa = CabeceraDeEmpresa::desde(&textos, leer_logo_de_empresa(&estado).await?);
    let foto = serde_json::from_str(&albaran.snapshot_json).unwrap_or_default();
    let pagina = PaginaAlbaranImpreso::desde(albaran.trabajo_id, &foto, empresa);
    Ok(renderizar(&pagina)?.into_response())
}
