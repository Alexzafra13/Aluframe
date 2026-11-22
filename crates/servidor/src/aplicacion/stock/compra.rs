//! Lista de la compra (PANTALLAS 9): lo que necesitan los trabajos con la orden abierta más lo
//! que falta para el mínimo, menos lo que hay. Solo para leer o imprimir: no genera pedidos.

use std::collections::BTreeMap;

use dominio::acceso::Accion;
use dominio::stock::ClaveDeStock;

use super::prevision::prever;
use crate::aplicacion::trabajos::orden::cargar_en;
use crate::aplicacion::{Actor, autorizar};
use crate::bd::consultas::stock::{self, FilaSaldo};
use crate::bd::consultas::{acabados, articulos, trabajos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub struct LineaDeCompra {
    pub referencia: String,
    pub descripcion: String,
    pub unidad: String,
    pub color: Option<String>,
    pub hay_mil: i64,
    pub necesitan_mil: i64,
    pub minimo_mil: i64,
    pub comprar_mil: i64,
}

/// Lo que gastarán los trabajos que aún no han terminado su orden, por artículo y color.
fn necesidades(con: &rusqlite::Connection) -> Result<BTreeMap<ClaveDeStock, i64>, ErrorApp> {
    let mut suma = BTreeMap::new();
    for t in trabajos::listar(con, false)?
        .iter()
        .filter(|t| !t.orden_terminada)
    {
        for salida in prever(&cargar_en(con, t.id)?).salidas {
            if salida.clave.2.is_none() {
                *suma.entry(salida.clave).or_default() -= salida.cantidad_mil;
            }
        }
    }
    Ok(suma)
}

fn linea(
    con: &rusqlite::Connection,
    (articulo, color): (i64, Option<i64>),
    (hay, necesitan, minimo): (i64, i64, i64),
) -> Result<Option<LineaDeCompra>, ErrorApp> {
    let comprar = necesitan + minimo - hay;
    if comprar <= 0 {
        return Ok(None);
    }
    let a = articulos::buscar(con, articulo)?.ok_or(ErrorApp::NoEncontrado)?;
    let color = match color {
        Some(id) => acabados::buscar(con, id)?.map(|c| c.nombre),
        None => None,
    };
    Ok(Some(LineaDeCompra {
        referencia: a.referencia,
        descripcion: a.descripcion,
        unidad: a.unidad,
        color,
        hay_mil: hay,
        necesitan_mil: necesitan,
        minimo_mil: minimo,
        comprar_mil: comprar,
    }))
}

pub async fn lista_de_compra(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<LineaDeCompra>, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado
        .bd
        .leer(|con| {
            let enteras: Vec<FilaSaldo> = stock::saldos(con)?
                .into_iter()
                .filter(|s| s.longitud_dmm.is_none())
                .collect();
            let mut claves: BTreeMap<(i64, Option<i64>), (i64, i64, i64)> = BTreeMap::new();
            for s in &enteras {
                let minimo = if s.acabado_id.is_none() {
                    s.minimo_mil
                } else {
                    0
                };
                claves.insert((s.articulo_id, s.acabado_id), (s.cantidad_mil, 0, minimo));
            }
            for ((articulo, color, _), necesita) in necesidades(con)? {
                claves.entry((articulo, color)).or_default().1 += necesita;
            }
            let mut lineas = Vec::new();
            for (clave, cantidades) in claves {
                lineas.extend(linea(con, clave, cantidades)?);
            }
            Ok(lineas)
        })
        .await
}
