use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

/// Un artículo activo con el precio de la tarifa vigente de su proveedor.
#[derive(Debug, Clone)]
pub struct ArticuloParaPrecio {
    pub id: i64,
    pub referencia: String,
    pub descripcion: String,
    pub proveedor: String,
    pub unidad: String,
    pub longitud_barra_dmm: Option<i64>,
    pub unidades_por_caja: Option<i64>,
    pub con_acabado: bool,
    pub precio_pu: Option<i64>,
    pub descuento_compra_bp: i64,
    pub tarifa_id: Option<i64>,
}

const SELECCION: &str = "SELECT a.id, a.referencia, a.descripcion, p.nombre, a.unidad, \
     a.longitud_barra_dmm, a.unidades_por_caja, a.con_acabado, tp.precio_pu, \
     coalesce(t.descuento_compra_bp, 0), t.id \
     FROM articulo a JOIN proveedor p ON p.id = a.proveedor_id \
     LEFT JOIN tarifa t ON t.proveedor_id = a.proveedor_id AND t.estado = 'vigente' \
     LEFT JOIN tarifa_precio tp ON tp.tarifa_id = t.id AND tp.articulo_id = a.id";

fn leer(f: &Row) -> rusqlite::Result<ArticuloParaPrecio> {
    Ok(ArticuloParaPrecio {
        id: f.get(0)?,
        referencia: f.get(1)?,
        descripcion: f.get(2)?,
        proveedor: f.get(3)?,
        unidad: f.get(4)?,
        longitud_barra_dmm: f.get(5)?,
        unidades_por_caja: f.get(6)?,
        con_acabado: f.get(7)?,
        precio_pu: f.get(8)?,
        descuento_compra_bp: f.get(9)?,
        tarifa_id: f.get(10)?,
    })
}

pub fn por_referencia(
    con: &Connection,
    referencia: &str,
) -> Result<Vec<ArticuloParaPrecio>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE a.referencia = ?1 AND a.archivado = 0 AND p.archivado = 0 ORDER BY p.nombre"
    ))?;
    let filas = consulta.query_map(params![referencia], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn por_id(con: &Connection, id: i64) -> Result<Option<ArticuloParaPrecio>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE a.id = ?1"), [id], leer)
        .optional()?)
}

/// Vidrios elegibles al presupuestar: los que se venden por m².
pub fn vidrios(con: &Connection) -> Result<Vec<ArticuloParaPrecio>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE a.unidad = 'm2' AND a.archivado = 0 AND p.archivado = 0 \
         ORDER BY p.nombre, a.referencia"
    ))?;
    let filas = consulta.query_map([], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}
