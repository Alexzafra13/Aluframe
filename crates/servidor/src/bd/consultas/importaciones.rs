use rusqlite::{Connection, OptionalExtension, params};

use crate::errores::ErrorApp;

pub struct NuevaImportacion<'a> {
    pub tipo: &'a str,
    pub sha256: &'a str,
    pub filas: i64,
    pub usuario_id: i64,
    pub importada_en: &'a str,
}

#[derive(Debug, Clone)]
pub struct FilaImportacion {
    pub id: i64,
    pub tipo: String,
    pub filas: i64,
    pub estado: String,
    pub importada_en: String,
}

fn leer_fila(f: &rusqlite::Row) -> rusqlite::Result<FilaImportacion> {
    Ok(FilaImportacion {
        id: f.get(0)?,
        tipo: f.get(1)?,
        filas: f.get(2)?,
        estado: f.get(3)?,
        importada_en: f.get(4)?,
    })
}

pub fn insertar(con: &Connection, nueva: &NuevaImportacion) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO importacion (tipo, sha256, filas, usuario_id, importada_en) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            nueva.tipo,
            nueva.sha256,
            nueva.filas,
            nueva.usuario_id,
            nueva.importada_en
        ],
    )?;
    Ok(con.last_insert_rowid())
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaImportacion>, ErrorApp> {
    Ok(con
        .query_row(
            "SELECT id, tipo, filas, estado, importada_en FROM importacion WHERE id = ?1",
            [id],
            leer_fila,
        )
        .optional()?)
}

pub fn ultimas(
    con: &Connection,
    tipo: &str,
    cuantas: i64,
) -> Result<Vec<FilaImportacion>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT id, tipo, filas, estado, importada_en FROM importacion WHERE tipo = ?1 \
         ORDER BY id DESC LIMIT ?2",
    )?;
    let filas = consulta.query_map(params![tipo, cuantas], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn marcar_deshecha(con: &Connection, id: i64, cuando: &str) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE importacion SET estado = 'deshecha', deshecha_en = ?1 \
         WHERE id = ?2 AND estado = 'confirmada'",
        params![cuando, id],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Validacion(
            "Esta importación ya se deshizo.".into(),
        ));
    }
    Ok(())
}
