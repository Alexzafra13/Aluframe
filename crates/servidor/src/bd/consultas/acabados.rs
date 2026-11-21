use dominio::catalogo::DatosDeAcabado;
use rusqlite::{Connection, OptionalExtension, Row, params};

use super::proveedores::comprobar;
use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaAcabado {
    pub id: i64,
    pub nombre: String,
    pub es_base: bool,
    pub recargo_bp: i64,
    pub archivado: bool,
    pub version: i64,
}

const SELECCION: &str = "SELECT id, nombre, es_base, recargo_bp, archivado, version FROM acabado";

fn leer_fila(f: &Row) -> rusqlite::Result<FilaAcabado> {
    Ok(FilaAcabado {
        id: f.get(0)?,
        nombre: f.get(1)?,
        es_base: f.get(2)?,
        recargo_bp: f.get(3)?,
        archivado: f.get(4)?,
        version: f.get(5)?,
    })
}

pub fn listar(con: &Connection) -> Result<Vec<FilaAcabado>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} ORDER BY archivado, es_base DESC, nombre"
    ))?;
    let filas = consulta.query_map([], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaAcabado>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE id = ?1"), [id], leer_fila)
        .optional()?)
}

/// Comprueba nombre libre y que solo haya un acabado base.
pub fn comprobar_unicos(
    con: &Connection,
    d: &DatosDeAcabado,
    excepto: Option<i64>,
) -> Result<(), ErrorApp> {
    let nombre_ocupado: bool = con.query_row(
        "SELECT EXISTS (SELECT 1 FROM acabado WHERE nombre = ?1 AND id IS NOT ?2)",
        params![d.nombre, excepto],
        |f| f.get(0),
    )?;
    if nombre_ocupado {
        return Err(ErrorApp::Validacion(format!(
            "Ya existe el acabado «{}».",
            d.nombre
        )));
    }
    let otro_base: Option<String> = con
        .query_row(
            "SELECT nombre FROM acabado WHERE es_base = 1 AND id IS NOT ?1",
            [excepto],
            |f| f.get(0),
        )
        .optional()?;
    match otro_base {
        Some(nombre) if d.es_base => Err(ErrorApp::Validacion(format!(
            "El acabado base ya es «{nombre}». Solo puede haber uno (el precio de tarifa)."
        ))),
        _ => Ok(()),
    }
}

pub fn insertar(con: &Connection, d: &DatosDeAcabado) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO acabado (nombre, es_base, recargo_bp) VALUES (?1, ?2, ?3)",
        params![d.nombre, d.es_base, d.recargo.puntos_basicos()],
    )?;
    Ok(con.last_insert_rowid())
}

pub fn actualizar(
    con: &Connection,
    (id, version): (i64, i64),
    d: &DatosDeAcabado,
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE acabado SET nombre = ?1, es_base = ?2, recargo_bp = ?3, version = version + 1 \
         WHERE id = ?4 AND version = ?5",
        params![d.nombre, d.es_base, d.recargo.puntos_basicos(), id, version],
    )?;
    comprobar(con, "acabado", cambiadas, id)
}
