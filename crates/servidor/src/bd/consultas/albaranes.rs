//! Albaranes de entrega (DATOS 3.8): se crean y se leen; nunca se modifican.

use rusqlite::{Connection, OptionalExtension, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaAlbaran {
    pub id: i64,
    pub anio: i64,
    pub numero: i64,
    pub trabajo_id: i64,
    pub fecha: String,
    pub snapshot_json: String,
}

impl FilaAlbaran {
    pub fn codigo(&self) -> String {
        format!("A-{}-{:04}", self.anio, self.numero)
    }
}

const SELECCION: &str = "SELECT id, anio, numero, trabajo_id, fecha, snapshot_json FROM albaran";

fn leer(f: &rusqlite::Row<'_>) -> rusqlite::Result<FilaAlbaran> {
    Ok(FilaAlbaran {
        id: f.get(0)?,
        anio: f.get(1)?,
        numero: f.get(2)?,
        trabajo_id: f.get(3)?,
        fecha: f.get(4)?,
        snapshot_json: f.get(5)?,
    })
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaAlbaran>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE id = ?1"), [id], leer)
        .optional()?)
}

pub fn de_trabajo(con: &Connection, trabajo: i64) -> Result<Vec<FilaAlbaran>, ErrorApp> {
    let mut consulta = con.prepare(&format!("{SELECCION} WHERE trabajo_id = ?1 ORDER BY id"))?;
    let filas = consulta.query_map([trabajo], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// La fotografía y su huella se guardan tal cual: el albarán no cambia después.
pub fn insertar(
    tx: &Connection,
    (anio, numero, trabajo): (i64, i64, i64),
    fecha: &str,
    (foto, huella): (&str, &str),
) -> Result<i64, ErrorApp> {
    tx.execute(
        "INSERT INTO albaran (anio, numero, trabajo_id, fecha, snapshot_json, snapshot_sha256) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![anio, numero, trabajo, fecha, foto, huella],
    )?;
    Ok(tx.last_insert_rowid())
}
