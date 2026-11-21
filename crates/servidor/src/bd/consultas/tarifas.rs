use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaTarifa {
    pub id: i64,
    pub proveedor_id: i64,
    pub proveedor: String,
    pub nombre: String,
    pub fecha_tarifa: Option<String>,
    pub descuento_compra_bp: i64,
    pub formato_origen: String,
    pub estado: String,
    pub importacion_id: Option<i64>,
    pub importada_en: String,
    pub precios: i64,
}

const SELECCION: &str = "SELECT t.id, t.proveedor_id, p.nombre, t.nombre, t.fecha_tarifa, \
     t.descuento_compra_bp, t.formato_origen, t.estado, t.importacion_id, t.importada_en, \
     (SELECT count(*) FROM tarifa_precio tp WHERE tp.tarifa_id = t.id) \
     FROM tarifa t JOIN proveedor p ON p.id = t.proveedor_id";

fn leer_fila(f: &Row) -> rusqlite::Result<FilaTarifa> {
    Ok(FilaTarifa {
        id: f.get(0)?,
        proveedor_id: f.get(1)?,
        proveedor: f.get(2)?,
        nombre: f.get(3)?,
        fecha_tarifa: f.get(4)?,
        descuento_compra_bp: f.get(5)?,
        formato_origen: f.get(6)?,
        estado: f.get(7)?,
        importacion_id: f.get(8)?,
        importada_en: f.get(9)?,
        precios: f.get(10)?,
    })
}

pub fn listar(con: &Connection) -> Result<Vec<FilaTarifa>, ErrorApp> {
    let mut consulta = con.prepare(&format!("{SELECCION} ORDER BY p.nombre, t.id DESC"))?;
    let filas = consulta.query_map([], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaTarifa>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE t.id = ?1"), [id], leer_fila)
        .optional()?)
}

pub fn vigente(con: &Connection, proveedor_id: i64) -> Result<Option<FilaTarifa>, ErrorApp> {
    Ok(con
        .query_row(
            &format!("{SELECCION} WHERE t.proveedor_id = ?1 AND t.estado = 'vigente'"),
            [proveedor_id],
            leer_fila,
        )
        .optional()?)
}

/// Precios de una versión, por artículo.
pub fn precios(con: &Connection, tarifa_id: i64) -> Result<BTreeMap<i64, i64>, ErrorApp> {
    let mut consulta =
        con.prepare("SELECT articulo_id, precio_pu FROM tarifa_precio WHERE tarifa_id = ?1")?;
    let filas = consulta.query_map([tarifa_id], |f| Ok((f.get(0)?, f.get(1)?)))?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub struct PrecioDeTarifa {
    pub referencia: String,
    pub descripcion: String,
    pub unidad: String,
    pub precio_pu: i64,
}

pub fn precios_con_articulo(
    con: &Connection,
    tarifa_id: i64,
) -> Result<Vec<PrecioDeTarifa>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT a.referencia, a.descripcion, a.unidad, tp.precio_pu FROM tarifa_precio tp \
         JOIN articulo a ON a.id = tp.articulo_id WHERE tp.tarifa_id = ?1 ORDER BY a.referencia",
    )?;
    let filas = consulta.query_map([tarifa_id], |f| {
        Ok(PrecioDeTarifa {
            referencia: f.get(0)?,
            descripcion: f.get(1)?,
            unidad: f.get(2)?,
            precio_pu: f.get(3)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub struct NuevaTarifa<'a> {
    pub proveedor_id: i64,
    pub nombre: &'a str,
    pub fecha_tarifa: Option<&'a str>,
    pub descuento_compra_bp: i64,
    pub formato_origen: &'a str,
    pub archivo_origen: Option<&'a str>,
    pub importacion_id: Option<i64>,
    pub ahora: &'a str,
    pub usuario_id: i64,
}

/// Inserta la versión nueva como vigente y deja la anterior como sustituida.
pub fn insertar(
    con: &Connection,
    t: &NuevaTarifa,
    precios: &BTreeMap<i64, i64>,
) -> Result<i64, ErrorApp> {
    con.execute(
        "UPDATE tarifa SET estado = 'sustituida' WHERE proveedor_id = ?1 AND estado = 'vigente'",
        [t.proveedor_id],
    )?;
    con.execute(
        "INSERT INTO tarifa (proveedor_id, nombre, fecha_tarifa, vigente_desde, descuento_compra_bp, \
         formato_origen, archivo_origen, estado, importacion_id, importada_en, usuario_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'vigente', ?8, ?4, ?9)",
        params![
            t.proveedor_id,
            t.nombre,
            t.fecha_tarifa,
            t.ahora,
            t.descuento_compra_bp,
            t.formato_origen,
            t.archivo_origen,
            t.importacion_id,
            t.usuario_id
        ],
    )?;
    let id = con.last_insert_rowid();
    let mut insercion = con.prepare(
        "INSERT INTO tarifa_precio (tarifa_id, articulo_id, precio_pu) VALUES (?1, ?2, ?3)",
    )?;
    for (articulo, precio) in precios {
        insercion.execute(params![id, articulo, precio])?;
    }
    Ok(id)
}

/// Borra una versión vigente y vuelve a dejar vigente la anterior de ese proveedor.
pub fn borrar_vigente(con: &Connection, tarifa: &FilaTarifa) -> Result<(), ErrorApp> {
    con.execute(
        "DELETE FROM tarifa_precio WHERE tarifa_id = ?1",
        [tarifa.id],
    )?;
    con.execute(
        "DELETE FROM tarifa WHERE id = ?1 AND estado = 'vigente'",
        [tarifa.id],
    )?;
    con.execute(
        "UPDATE tarifa SET estado = 'vigente' WHERE id = \
         (SELECT max(id) FROM tarifa WHERE proveedor_id = ?1)",
        [tarifa.proveedor_id],
    )?;
    Ok(())
}

/// Quita los artículos que creó una importación si ninguna otra tarifa los usa y nadie los editó.
pub fn quitar_articulos_importados(
    con: &Connection,
    importacion_id: i64,
) -> Result<usize, ErrorApp> {
    let quitados = con.execute(
        "DELETE FROM articulo WHERE importacion_id = ?1 AND version = 1 \
         AND NOT EXISTS (SELECT 1 FROM tarifa_precio tp WHERE tp.articulo_id = articulo.id)",
        [importacion_id],
    )?;
    con.execute(
        "UPDATE articulo SET importacion_id = NULL WHERE importacion_id = ?1",
        [importacion_id],
    )?;
    Ok(quitados)
}

pub struct PrecioEnHistorial {
    pub tarifa: String,
    pub vigente_desde: String,
    pub formato_origen: String,
    pub estado: String,
    pub precio_pu: i64,
}

pub fn historial(con: &Connection, articulo_id: i64) -> Result<Vec<PrecioEnHistorial>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT t.nombre, t.vigente_desde, t.formato_origen, t.estado, tp.precio_pu \
         FROM tarifa_precio tp JOIN tarifa t ON t.id = tp.tarifa_id \
         WHERE tp.articulo_id = ?1 ORDER BY t.id DESC",
    )?;
    let filas = consulta.query_map([articulo_id], |f| {
        Ok(PrecioEnHistorial {
            tarifa: f.get(0)?,
            vigente_desde: f.get(1)?,
            formato_origen: f.get(2)?,
            estado: f.get(3)?,
            precio_pu: f.get(4)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}
