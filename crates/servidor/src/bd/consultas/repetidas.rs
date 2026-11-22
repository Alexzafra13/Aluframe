//! Piezas que se vuelven a cortar durante la fabricación (H5).

use rusqlite::{Connection, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaRepetida {
    pub id: i64,
    pub ventana_id: i64,
    pub numero: i64,
    pub perfil: String,
    pub longitud_dmm: i64,
    pub cantidad: i64,
    pub motivo: String,
    pub usuario: String,
    pub cuando_en: String,
}

pub fn de_ventana(con: &Connection, ventana: i64) -> Result<Vec<FilaRepetida>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT r.id, r.ventana_id, r.numero, r.perfil, r.longitud_dmm, r.cantidad, r.motivo, \
         u.nombre, r.cuando_en FROM pieza_repetida r JOIN usuario u ON u.id = r.usuario_id \
         WHERE r.ventana_id = ?1 ORDER BY r.id",
    )?;
    let filas = consulta.query_map([ventana], |f| {
        Ok(FilaRepetida {
            id: f.get(0)?,
            ventana_id: f.get(1)?,
            numero: f.get(2)?,
            perfil: f.get(3)?,
            longitud_dmm: f.get(4)?,
            cantidad: f.get(5)?,
            motivo: f.get(6)?,
            usuario: f.get(7)?,
            cuando_en: f.get(8)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub struct NuevaRepetida<'a> {
    pub ventana: i64,
    pub numero: i64,
    pub perfil: &'a str,
    pub longitud_dmm: i64,
    pub cantidad: i64,
    pub motivo: &'a str,
    pub usuario: i64,
    pub cuando: &'a str,
}

pub fn insertar(con: &Connection, r: &NuevaRepetida<'_>) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO pieza_repetida (ventana_id, numero, perfil, longitud_dmm, cantidad, motivo, \
         usuario_id, cuando_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            r.ventana,
            r.numero,
            r.perfil,
            r.longitud_dmm,
            r.cantidad,
            r.motivo,
            r.usuario,
            r.cuando
        ],
    )?;
    Ok(())
}
