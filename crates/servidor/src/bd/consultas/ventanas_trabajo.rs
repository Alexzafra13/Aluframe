//! Ventanas de un trabajo, sus pasos y su historial (DATOS 7.7).

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaVentana {
    pub id: i64,
    pub trabajo_id: i64,
    pub orden: i64,
    pub linea_orden: i64,
    pub descripcion: String,
    pub asignado_a: Option<i64>,
    pub cristal_recibido: bool,
    pub revision: i64,
    pub revisada_en: Option<String>,
    pub medida_cambiada: bool,
    pub falta_algo: Option<String>,
    pub congelado_json: String,
    pub version: i64,
}

const COLUMNAS: &str = "id, trabajo_id, orden, linea_orden, descripcion, asignado_a, \
     cristal_recibido, revision, revisada_en, medida_cambiada, falta_algo, congelado_json, version";

fn leer_ventana(f: &Row<'_>) -> rusqlite::Result<FilaVentana> {
    Ok(FilaVentana {
        id: f.get(0)?,
        trabajo_id: f.get(1)?,
        orden: f.get(2)?,
        linea_orden: f.get(3)?,
        descripcion: f.get(4)?,
        asignado_a: f.get(5)?,
        cristal_recibido: f.get(6)?,
        revision: f.get(7)?,
        revisada_en: f.get(8)?,
        medida_cambiada: f.get(9)?,
        falta_algo: f.get(10)?,
        congelado_json: f.get(11)?,
        version: f.get(12)?,
    })
}

pub fn insertar(
    con: &Connection,
    trabajo: i64,
    (orden, linea_orden): (i64, i64),
    (descripcion, congelado): (&str, &str),
) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO ventana_trabajo (trabajo_id, orden, linea_orden, descripcion, congelado_json) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![trabajo, orden, linea_orden, descripcion, congelado],
    )?;
    Ok(con.last_insert_rowid())
}

pub fn de_trabajo(con: &Connection, trabajo: i64) -> Result<Vec<FilaVentana>, ErrorApp> {
    let sql =
        format!("SELECT {COLUMNAS} FROM ventana_trabajo WHERE trabajo_id = ?1 ORDER BY orden");
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([trabajo], leer_ventana)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaVentana>, ErrorApp> {
    let sql = format!("SELECT {COLUMNAS} FROM ventana_trabajo WHERE id = ?1");
    Ok(con.query_row(&sql, [id], leer_ventana).optional()?)
}

/// Las del operario: asignadas a él, o sin asignar dentro de un trabajo asignado a él. Solo
/// las de trabajos que siguen en el taller.
pub fn de_operario(con: &Connection, usuario: i64) -> Result<Vec<FilaVentana>, ErrorApp> {
    let columnas = COLUMNAS
        .split(", ")
        .map(|c| format!("v.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {columnas} FROM ventana_trabajo v JOIN trabajo t ON t.id = v.trabajo_id \
         WHERE t.estado NOT IN ('entregado', 'anulado') \
         AND (v.asignado_a = ?1 OR (v.asignado_a IS NULL AND t.asignado_a = ?1)) \
         ORDER BY t.fecha_prevista IS NULL, t.fecha_prevista, t.id, v.orden"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([usuario], leer_ventana)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Las ventanas abiertas del taller que no son de `usuario` (de otros o sin asignar).
pub fn abiertas_de_otros(con: &Connection, usuario: i64) -> Result<Vec<FilaVentana>, ErrorApp> {
    let columnas = COLUMNAS
        .split(", ")
        .map(|c| format!("v.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {columnas} FROM ventana_trabajo v JOIN trabajo t ON t.id = v.trabajo_id \
         WHERE t.estado NOT IN ('entregado', 'anulado') \
         AND coalesce(v.asignado_a, t.asignado_a, 0) <> ?1 \
         ORDER BY t.fecha_prevista IS NULL, t.fecha_prevista, t.id, v.orden"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([usuario], leer_ventana)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn asignar(con: &Connection, id: i64, usuario: Option<i64>) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE ventana_trabajo SET asignado_a = ?2, version = version + 1 WHERE id = ?1",
        params![id, usuario],
    )?;
    Ok(())
}

pub fn marcar_cristal(con: &Connection, id: i64, recibido: bool) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE ventana_trabajo SET cristal_recibido = ?2, version = version + 1 WHERE id = ?1",
        params![id, recibido],
    )?;
    Ok(())
}

pub fn anotar_falta(con: &Connection, id: i64, nota: Option<&str>) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE ventana_trabajo SET falta_algo = ?2, version = version + 1 WHERE id = ?1",
        params![id, nota],
    )?;
    Ok(())
}

pub fn marcar_vista(con: &Connection, id: i64) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE ventana_trabajo SET medida_cambiada = 0 WHERE id = ?1 AND medida_cambiada = 1",
        [id],
    )?;
    Ok(())
}

/// Nueva revisión de las medidas (solo el propietario): la anterior queda en `cambio_ventana_trabajo`.
pub fn revisar(
    con: &Connection,
    (id, version): (i64, i64),
    (descripcion, congelado, cuando): (&str, &str, &str),
) -> Result<i64, ErrorApp> {
    let revision: Option<i64> = con
        .query_row(
            "UPDATE ventana_trabajo SET descripcion = ?3, congelado_json = ?4, revisada_en = ?5, \
             revision = revision + 1, medida_cambiada = 1, version = version + 1 \
             WHERE id = ?1 AND version = ?2 RETURNING revision",
            params![id, version, descripcion, congelado, cuando],
            |f| f.get(0),
        )
        .optional()?;
    revision.ok_or(ErrorApp::Conflicto)
}

pub struct Revision<'a> {
    pub ventana: i64,
    pub revision: i64,
    pub anterior: &'a str,
    pub nuevo: &'a str,
    pub motivo: &'a str,
    pub pasos_reiniciados: bool,
    pub usuario: i64,
    pub cuando: &'a str,
}

pub fn anotar_revision(con: &Connection, r: &Revision<'_>) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO cambio_ventana_trabajo (ventana_id, revision, congelado_anterior_json, \
         congelado_nuevo_json, motivo, pasos_reiniciados, usuario_id, cuando_en) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            r.ventana,
            r.revision,
            r.anterior,
            r.nuevo,
            r.motivo,
            r.pasos_reiniciados,
            r.usuario,
            r.cuando
        ],
    )?;
    Ok(())
}
