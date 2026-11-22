//! Libro de stock (DATOS 7.8): el saldo es la suma de los movimientos, sin saldo guardado.

use rusqlite::{Connection, Row, params};

use crate::errores::ErrorApp;

/// Saldo de un artículo en un color y, si es un retal, con su longitud.
#[derive(Debug, Clone)]
pub struct FilaSaldo {
    pub articulo_id: i64,
    pub referencia: String,
    pub descripcion: String,
    pub unidad: String,
    pub acabado_id: Option<i64>,
    pub acabado: Option<String>,
    pub longitud_dmm: Option<i64>,
    pub cantidad_mil: i64,
    pub minimo_mil: i64,
}

fn leer_saldo(f: &Row<'_>) -> rusqlite::Result<FilaSaldo> {
    Ok(FilaSaldo {
        articulo_id: f.get(0)?,
        referencia: f.get(1)?,
        descripcion: f.get(2)?,
        unidad: f.get(3)?,
        acabado_id: f.get(4)?,
        acabado: f.get(5)?,
        longitud_dmm: f.get(6)?,
        cantidad_mil: f.get(7)?,
        minimo_mil: f.get(8)?,
    })
}

/// Los artículos con movimientos o con mínimo, con sus saldos por color y longitud. Las barras
/// enteras van sin longitud; los retales, con la suya.
pub fn saldos(con: &Connection) -> Result<Vec<FilaSaldo>, ErrorApp> {
    let sql = "SELECT a.id, a.referencia, a.descripcion, a.unidad, m.acabado_id, c.nombre, \
         m.longitud_dmm, ifnull(sum(m.cantidad_mil), 0), a.stock_minimo_mil \
         FROM articulo a LEFT JOIN movimiento_stock m ON m.articulo_id = a.id \
         LEFT JOIN acabado c ON c.id = m.acabado_id \
         WHERE a.archivado = 0 AND (m.id IS NOT NULL OR a.stock_minimo_mil > 0) \
         GROUP BY a.id, m.acabado_id, m.longitud_dmm \
         HAVING m.longitud_dmm IS NULL OR sum(m.cantidad_mil) <> 0 \
         ORDER BY a.referencia, c.nombre, m.longitud_dmm IS NOT NULL, m.longitud_dmm DESC";
    let mut consulta = con.prepare(sql)?;
    let filas = consulta.query_map([], leer_saldo)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Retales (longitud y cuántos) de un artículo en un color; sin color es `acabado_id` NULL.
pub fn retales(
    con: &Connection,
    articulo: i64,
    acabado: Option<i64>,
) -> Result<Vec<(i64, i64)>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT longitud_dmm, sum(cantidad_mil) FROM movimiento_stock \
         WHERE articulo_id = ?1 AND acabado_id IS ?2 AND longitud_dmm IS NOT NULL \
         GROUP BY longitud_dmm HAVING sum(cantidad_mil) > 0 ORDER BY longitud_dmm",
    )?;
    let filas = consulta.query_map(params![articulo, acabado], |f| Ok((f.get(0)?, f.get(1)?)))?;
    Ok(filas.collect::<Result<_, _>>()?)
}

#[derive(Debug, Clone)]
pub struct FilaMovimiento {
    pub id: i64,
    pub acabado: Option<String>,
    pub longitud_dmm: Option<i64>,
    pub tipo: String,
    pub cantidad_mil: i64,
    pub motivo: String,
    pub orden_taller_id: Option<i64>,
    pub usuario: String,
    pub cuando_en: String,
}

pub fn movimientos(con: &Connection, articulo: i64) -> Result<Vec<FilaMovimiento>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT m.id, c.nombre, m.longitud_dmm, m.tipo, m.cantidad_mil, m.motivo, \
         m.orden_taller_id, u.nombre, m.cuando_en FROM movimiento_stock m \
         JOIN usuario u ON u.id = m.usuario_id LEFT JOIN acabado c ON c.id = m.acabado_id \
         WHERE m.articulo_id = ?1 ORDER BY m.id DESC LIMIT 200",
    )?;
    let filas = consulta.query_map([articulo], |f| {
        Ok(FilaMovimiento {
            id: f.get(0)?,
            acabado: f.get(1)?,
            longitud_dmm: f.get(2)?,
            tipo: f.get(3)?,
            cantidad_mil: f.get(4)?,
            motivo: f.get(5)?,
            orden_taller_id: f.get(6)?,
            usuario: f.get(7)?,
            cuando_en: f.get(8)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub struct NuevoMovimiento<'a> {
    pub articulo: i64,
    pub acabado: Option<i64>,
    pub longitud_dmm: Option<i64>,
    pub tipo: &'a str,
    pub cantidad_mil: i64,
    pub motivo: &'a str,
    pub orden_taller: Option<i64>,
    pub usuario: i64,
    pub cuando: &'a str,
}

pub fn insertar(con: &Connection, m: &NuevoMovimiento<'_>) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO movimiento_stock (articulo_id, acabado_id, longitud_dmm, tipo, cantidad_mil, \
         motivo, orden_taller_id, usuario_id, cuando_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            m.articulo,
            m.acabado,
            m.longitud_dmm,
            m.tipo,
            m.cantidad_mil,
            m.motivo,
            m.orden_taller,
            m.usuario,
            m.cuando
        ],
    )?;
    Ok(())
}

pub fn cambiar_minimo(con: &Connection, articulo: i64, minimo_mil: i64) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE articulo SET stock_minimo_mil = ?2, version = version + 1 WHERE id = ?1",
        params![articulo, minimo_mil],
    )?;
    Ok(())
}

pub fn saldo(
    con: &Connection,
    (articulo, acabado, longitud): (i64, Option<i64>, Option<i64>),
) -> Result<i64, ErrorApp> {
    Ok(con.query_row(
        "SELECT ifnull(sum(cantidad_mil), 0) FROM movimiento_stock \
         WHERE articulo_id = ?1 AND acabado_id IS ?2 AND longitud_dmm IS ?3",
        params![articulo, acabado, longitud],
        |f| f.get(0),
    )?)
}
