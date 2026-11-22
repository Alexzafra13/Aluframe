//! Terminar la orden de taller (SPEC, H5): en una transacción se congela el plan de corte y se
//! descuenta del stock lo que gasta, con los retales que el propietario confirma. No se repite:
//! una orden terminada ya no cambia (y la clave única de los movimientos lo garantiza).

use dominio::acceso::Accion;
use dominio::stock::{ClaveDeStock, agrupar_movimientos};
use serde_json::json;

use super::prevision::prever;
use crate::aplicacion::auditar::{Cambio, anotar};
use crate::aplicacion::trabajos::orden::{OrdenDeTaller, cargar_en};
use crate::aplicacion::{Actor, autorizar};
use crate::bd::consultas::stock::{self, NuevoMovimiento};
use crate::bd::consultas::trabajos;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

/// Un retal que entra en el stock: el grupo (perfil y color) y su largo, ya revisado.
#[derive(Debug, Clone)]
pub struct RetalConfirmado {
    pub grupo: usize,
    pub largo_dmm: i64,
}

/// El plan tal como se cortó: por perfil y color, cada barra (nueva o retal) con sus piezas.
fn plan_en_json(orden: &OrdenDeTaller) -> String {
    let grupos: Vec<_> = orden
        .plan
        .iter()
        .map(|g| {
            let barras: Vec<_> = g.plan.as_ref().map_or(Vec::new(), |p| {
                p.barras
                    .iter()
                    .map(|b| {
                        let piezas: Vec<_> = b
                            .piezas
                            .iter()
                            .map(|p| (g.origen.get(&p.id), p.longitud.decimas_de_mm()))
                            .collect();
                        let retal = b.retal.and_then(|id| orden.enlace.largo_de_retal.get(&id));
                        json!({ "retal_dmm": retal, "sobrante_dmm": b.sobrante.decimas_de_mm(), "piezas": piezas })
                    })
                    .collect()
            });
            json!({ "perfil": g.perfil, "color": g.color, "barra_dmm": g.longitud_barra.decimas_de_mm(), "barras": barras })
        })
        .collect();
    json!(grupos).to_string()
}

/// Las salidas previstas y las entradas de los retales confirmados, ya agrupadas.
fn movimientos_de(
    orden: &OrdenDeTaller,
    retales: &[RetalConfirmado],
) -> Result<Vec<(ClaveDeStock, i64)>, ErrorApp> {
    let mut movimientos: Vec<_> = prever(orden)
        .salidas
        .iter()
        .map(|s| (s.clave, s.cantidad_mil))
        .collect();
    for r in retales {
        let (articulo, color) = orden
            .enlace
            .perfiles
            .values()
            .nth(r.grupo)
            .copied()
            .ok_or_else(|| {
                ErrorApp::Validacion("Ese retal no es de ningún perfil del trabajo.".into())
            })?;
        movimientos.push(((articulo, color, Some(r.largo_dmm)), 1_000));
    }
    Ok(agrupar_movimientos(&movimientos))
}

fn guardar(
    tx: &rusqlite::Transaction,
    trabajo: i64,
    retales: &[RetalConfirmado],
    (cuando, autor): (&str, i64),
) -> Result<(), ErrorApp> {
    let orden = cargar_en(tx, trabajo)?;
    if orden.ficha.trabajo.estado == "anulado" {
        return Err(ErrorApp::Validacion("El trabajo está anulado.".into()));
    }
    let movimientos = movimientos_de(&orden, retales)?;
    let id_orden = trabajos::terminar_orden(tx, trabajo, (&plan_en_json(&orden), cuando, autor))?;
    let motivo = format!("Orden de taller {}", orden.ficha.trabajo.codigo());
    for ((articulo, acabado, longitud), cantidad) in movimientos {
        stock::insertar(
            tx,
            &NuevoMovimiento {
                articulo,
                acabado,
                longitud_dmm: longitud,
                tipo: if cantidad > 0 { "entrada" } else { "salida" },
                cantidad_mil: cantidad,
                motivo: &motivo,
                orden_taller: Some(id_orden),
                usuario: autor,
                cuando,
            },
        )?;
    }
    Ok(())
}

pub async fn terminar_orden(
    estado: &EstadoApp,
    actor: &Actor,
    trabajo: i64,
    retales: Vec<RetalConfirmado>,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            guardar(tx, trabajo, &retales, (&formatear_utc(ahora), autor))?;
            let cambio = Cambio {
                ahora,
                autor,
                accion: "terminar_orden",
                entidad: "trabajo",
                entidad_id: trabajo,
            };
            anotar(tx, &cambio)
        })
        .await
}

/// Lo que se verá antes de confirmar: la orden (con su plan) y lo que descontará.
pub async fn preparar_terminar(
    estado: &EstadoApp,
    actor: &Actor,
    trabajo: i64,
) -> Result<(OrdenDeTaller, super::prevision::Prevision), ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado
        .bd
        .leer(move |con| {
            let orden = cargar_en(con, trabajo)?;
            if orden.ficha.trabajo.orden_terminada {
                return Err(ErrorApp::Validacion("La orden ya está terminada.".into()));
            }
            let prevision = prever(&orden);
            Ok((orden, prevision))
        })
        .await
}
