//! La orden de taller: las ventanas del trabajo y su plan de corte conjunto (decisión 2 de H4).

use std::collections::BTreeMap;

use dominio::acceso::Accion;
use dominio::taller::{BarrasDePerfil, PiezaDeTrabajo, RetalesPorPerfil, planificar_trabajo};
use dominio::unidades::Longitud;

use super::consultar::{FichaDeTrabajo, cargar};
use crate::aplicacion::taller::{self, AjustesDeTaller};
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub struct OrdenDeTaller {
    pub ficha: FichaDeTrabajo,
    pub plan: Vec<BarrasDePerfil>,
    pub ajustes: AjustesDeTaller,
}

pub fn planificar(
    ficha: &FichaDeTrabajo,
    ajustes: &AjustesDeTaller,
    retales: &RetalesPorPerfil,
) -> Vec<BarrasDePerfil> {
    let mut piezas = Vec::new();
    let mut barras = BTreeMap::new();
    for ventana in &ficha.ventanas {
        let c = &ventana.congelado;
        let orden = u32::try_from(ventana.fila.orden).unwrap_or_default();
        for p in &c.resumen.piezas {
            if let Some(dmm) = c.longitud_barra_dmm {
                barras.insert(p.perfil.clone(), Longitud::desde_decimas_de_mm(dmm));
            }
            piezas.push(PiezaDeTrabajo {
                perfil: p.perfil.clone(),
                color: c.acabado.clone(),
                longitud: Longitud::desde_decimas_de_mm(p.longitud_dmm),
                cantidad: p.cantidad,
                ventana: orden,
                numero: p.numero,
            });
        }
    }
    planificar_trabajo(
        &piezas,
        (&barras, retales),
        ajustes.parametros_de_corte(Longitud::CERO),
    )
}

pub async fn cargar_orden(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<OrdenDeTaller, ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado
        .bd
        .leer(move |con| {
            let ficha = cargar(con, id)?;
            let ajustes = taller::leer_de_la_base(con)?;
            Ok(OrdenDeTaller {
                plan: planificar(&ficha, &ajustes, &RetalesPorPerfil::new()),
                ficha,
                ajustes,
            })
        })
        .await
}
