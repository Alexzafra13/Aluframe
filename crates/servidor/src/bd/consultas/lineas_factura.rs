//! Líneas de una factura en borrador; al emitirla, los disparadores del esquema las protegen.

use rusqlite::{Connection, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaLineaFactura {
    pub id: i64,
    pub orden: i64,
    pub descripcion: String,
    pub cantidad_mil: i64,
    pub precio_unitario_pu: i64,
    pub descuento_bp: i64,
    pub tipo_iva_bp: i64,
    pub base_cent: i64,
}

pub fn lineas(con: &Connection, factura: i64) -> Result<Vec<FilaLineaFactura>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT id, orden, descripcion, cantidad_mil, precio_unitario_pu, descuento_bp, tipo_iva_bp, \
         base_cent FROM linea_factura WHERE factura_id = ?1 ORDER BY orden",
    )?;
    let filas = consulta.query_map([factura], |f| {
        Ok(FilaLineaFactura {
            id: f.get(0)?,
            orden: f.get(1)?,
            descripcion: f.get(2)?,
            cantidad_mil: f.get(3)?,
            precio_unitario_pu: f.get(4)?,
            descuento_bp: f.get(5)?,
            tipo_iva_bp: f.get(6)?,
            base_cent: f.get(7)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn insertar_linea(
    con: &Connection,
    factura: i64,
    l: &FilaLineaFactura,
) -> Result<(), ErrorApp> {
    let orden: i64 = con.query_row(
        "SELECT ifnull(max(orden), 0) + 1 FROM linea_factura WHERE factura_id = ?1",
        [factura],
        |f| f.get(0),
    )?;
    con.execute(
        "INSERT INTO linea_factura (factura_id, orden, descripcion, cantidad_mil, precio_unitario_pu, \
         descuento_bp, tipo_iva_bp, base_cent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            factura,
            orden,
            l.descripcion,
            l.cantidad_mil,
            l.precio_unitario_pu,
            l.descuento_bp,
            l.tipo_iva_bp,
            l.base_cent
        ],
    )?;
    Ok(())
}

pub fn quitar_linea(con: &Connection, factura: i64, linea: i64) -> Result<(), ErrorApp> {
    con.execute(
        "DELETE FROM linea_factura WHERE id = ?2 AND factura_id = ?1",
        params![factura, linea],
    )?;
    Ok(())
}
