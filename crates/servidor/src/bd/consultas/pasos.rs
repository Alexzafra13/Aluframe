//! Pasos de cada ventana del taller y su historial de solo añadir.

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaPaso {
    pub id: i64,
    pub ventana_id: i64,
    pub orden: i64,
    pub nombre: String,
    pub estado: String,
    pub usuario_nombre: Option<String>,
    pub cambiado_en: Option<String>,
    pub version: i64,
}

fn leer_paso(f: &Row<'_>) -> rusqlite::Result<FilaPaso> {
    Ok(FilaPaso {
        id: f.get(0)?,
        ventana_id: f.get(1)?,
        orden: f.get(2)?,
        nombre: f.get(3)?,
        estado: f.get(4)?,
        usuario_nombre: f.get(5)?,
        cambiado_en: f.get(6)?,
        version: f.get(7)?,
    })
}

const SELECCION: &str = "SELECT p.id, p.ventana_id, p.orden, p.nombre, p.estado, u.nombre, \
     p.cambiado_en, p.version FROM paso_ventana p LEFT JOIN usuario u ON u.id = p.usuario_id";

pub fn insertar(con: &Connection, ventana: i64, orden: i64, nombre: &str) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO paso_ventana (ventana_id, orden, nombre) VALUES (?1, ?2, ?3)",
        params![ventana, orden, nombre],
    )?;
    Ok(())
}

pub fn de_ventana(con: &Connection, ventana: i64) -> Result<Vec<FilaPaso>, ErrorApp> {
    let sql = format!("{SELECCION} WHERE p.ventana_id = ?1 ORDER BY p.orden");
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([ventana], leer_paso)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Todos los pasos de un trabajo, por ventana y orden: para el progreso y el estado.
pub fn de_trabajo(con: &Connection, trabajo: i64) -> Result<Vec<FilaPaso>, ErrorApp> {
    let sql = format!(
        "{SELECCION} JOIN ventana_trabajo v ON v.id = p.ventana_id WHERE v.trabajo_id = ?1 \
         ORDER BY v.orden, p.orden"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([trabajo], leer_paso)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaPaso>, ErrorApp> {
    let sql = format!("{SELECCION} WHERE p.id = ?1");
    Ok(con.query_row(&sql, [id], leer_paso).optional()?)
}

/// Si otra persona tocó el paso antes (otra versión), no se cambia: la segunda ve el aviso.
pub fn cambiar(
    con: &Connection,
    (id, version): (i64, i64),
    (estado, usuario, cuando): (&str, i64, &str),
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE paso_ventana SET estado = ?3, usuario_id = ?4, cambiado_en = ?5, \
         version = version + 1 WHERE id = ?1 AND version = ?2",
        params![id, version, estado, usuario, cuando],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Conflicto);
    }
    Ok(())
}

pub fn reiniciar_de_ventana(con: &Connection, ventana: i64) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE paso_ventana SET estado = 'pendiente', version = version + 1 \
         WHERE ventana_id = ?1 AND estado <> 'pendiente'",
        [ventana],
    )?;
    Ok(())
}

pub struct ApunteDePaso<'a> {
    pub paso: i64,
    pub anterior: &'a str,
    pub nuevo: &'a str,
    pub nota: Option<&'a str>,
    pub usuario: i64,
    pub cuando: &'a str,
}

pub fn anotar(con: &Connection, a: &ApunteDePaso<'_>) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO paso_historial (paso_id, estado_anterior, estado_nuevo, nota, usuario_id, \
         cuando_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![a.paso, a.anterior, a.nuevo, a.nota, a.usuario, a.cuando],
    )?;
    Ok(())
}
