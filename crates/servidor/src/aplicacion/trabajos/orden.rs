//! La orden de taller: las ventanas del trabajo y su plan de corte conjunto (decisión 2 de H4).

use std::collections::BTreeMap;

use dominio::acceso::Accion;
use dominio::taller::{BarrasDePerfil, PiezaDeTrabajo, RetalesPorPerfil, planificar_trabajo};
use dominio::unidades::Longitud;

use super::consultar::{FichaDeTrabajo, cargar};
use crate::aplicacion::stock::orden::{EnlaceDeStock, enlazar_stock};
use crate::aplicacion::taller::{self, AjustesDeTaller};
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub struct OrdenDeTaller {
    pub ficha: FichaDeTrabajo,
    pub plan: Vec<BarrasDePerfil>,
    pub ajustes: AjustesDeTaller,
    /// Con qué artículo y retales del stock se enlaza cada perfil (H5).
    pub enlace: EnlaceDeStock,
}

/// Las piezas de todas las ventanas, más las que se han vuelto a cortar (H5).
fn piezas_del_trabajo(ficha: &FichaDeTrabajo) -> (Vec<PiezaDeTrabajo>, BTreeMap<String, Longitud>) {
    let mut piezas = Vec::new();
    let mut barras = BTreeMap::new();
    for ventana in &ficha.ventanas {
        let c = &ventana.congelado;
        let orden = u32::try_from(ventana.fila.orden).unwrap_or_default();
        let pieza = |perfil: &str, longitud_dmm: i64, cantidad: u32, numero: u32| PiezaDeTrabajo {
            perfil: perfil.to_owned(),
            color: c.acabado.clone(),
            longitud: Longitud::desde_decimas_de_mm(longitud_dmm),
            cantidad,
            ventana: orden,
            numero,
        };
        for p in &c.resumen.piezas {
            if let Some(dmm) = c.longitud_barra_dmm {
                barras.insert(p.perfil.clone(), Longitud::desde_decimas_de_mm(dmm));
            }
            piezas.push(pieza(&p.perfil, p.longitud_dmm, p.cantidad, p.numero));
        }
        for r in &ventana.repetidas {
            let cantidad = u32::try_from(r.cantidad).unwrap_or_default();
            let numero = u32::try_from(r.numero).unwrap_or_default();
            piezas.push(pieza(&r.perfil, r.longitud_dmm, cantidad, numero));
        }
    }
    (piezas, barras)
}

pub fn planificar(
    ficha: &FichaDeTrabajo,
    ajustes: &AjustesDeTaller,
    retales: &RetalesPorPerfil,
) -> Vec<BarrasDePerfil> {
    let (piezas, barras) = piezas_del_trabajo(ficha);
    planificar_trabajo(
        &piezas,
        (&barras, retales),
        ajustes.parametros_de_corte(Longitud::CERO),
    )
}

pub fn cargar_en(con: &rusqlite::Connection, id: i64) -> Result<OrdenDeTaller, ErrorApp> {
    let ficha = cargar(con, id)?;
    let ajustes = taller::leer_de_la_base(con)?;
    let enlace = enlazar_stock(con, &ficha)?;
    Ok(OrdenDeTaller {
        plan: planificar(&ficha, &ajustes, &enlace.retales),
        ficha,
        ajustes,
        enlace,
    })
}

pub async fn cargar_orden(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<OrdenDeTaller, ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado.bd.leer(move |con| cargar_en(con, id)).await
}
