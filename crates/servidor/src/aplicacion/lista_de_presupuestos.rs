//! Listas de presupuestos por su situación de hoy (ADR-0027): caducados y archivados se
//! calculan con la fecha, en `dominio::presupuesto`, y solo se cargan los que se muestran.

use std::collections::BTreeMap;

use dominio::acceso::Accion;
use dominio::presupuesto::{Situacion, situacion};
use rusqlite::Connection;

use super::{Actor, autorizar};
use crate::bd::consultas::presupuestos::{self, FilaPresupuesto};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fechas_de_situacion;

/// Un presupuesto de una lista y cómo se ve hoy.
pub struct PresupuestoListado {
    pub fila: FilaPresupuesto,
    pub situacion: Situacion,
}

/// Las listas no pasan de 500: los demás siguen en la ficha de su cliente.
const MAXIMO_EN_LISTA: usize = 500;

/// Los de una situación («caducado», «archivado»…) o, sin filtro, todos menos los archivados.
pub async fn listar_presupuestos(
    estado: &EstadoApp,
    actor: &Actor,
    filtro: Option<String>,
) -> Result<Vec<PresupuestoListado>, ErrorApp> {
    autorizar(actor, Accion::VerPresupuestos)?;
    let fechas = fechas_de_situacion(estado.ahora());
    let elegir = move |s: Situacion| match filtro.as_deref() {
        None => s != Situacion::Archivado,
        Some(clave) => s.clave() == clave,
    };
    estado
        .bd
        .leer(move |con| listar_en(con, None, elegir, &fechas))
        .await
}

/// Todos los del cliente, también los archivados.
pub async fn presupuestos_del_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    cliente_id: i64,
) -> Result<Vec<PresupuestoListado>, ErrorApp> {
    autorizar(actor, Accion::VerPresupuestos)?;
    let fechas = fechas_de_situacion(estado.ahora());
    estado
        .bd
        .leer(move |con| listar_en(con, Some(cliente_id), |_| true, &fechas))
        .await
}

fn listar_en(
    con: &Connection,
    cliente: Option<i64>,
    elegir: impl Fn(Situacion) -> bool,
    (hoy, archivar): &(String, String),
) -> Result<Vec<PresupuestoListado>, ErrorApp> {
    let mut elegidos = BTreeMap::new();
    for (id, estado, validez) in presupuestos::estados_y_validez(con, cliente)? {
        let s = situacion(&estado, validez.as_deref(), (hoy, archivar));
        if elegir(s) {
            elegidos.insert(id, s);
        }
        if elegidos.len() >= MAXIMO_EN_LISTA {
            break;
        }
    }
    let ids: Vec<i64> = elegidos.keys().copied().collect();
    Ok(presupuestos::buscar_varios(con, &ids)?
        .into_iter()
        .filter_map(|fila| {
            let situacion = *elegidos.get(&fila.id)?;
            Some(PresupuestoListado { fila, situacion })
        })
        .collect())
}
