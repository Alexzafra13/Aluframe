use rusqlite::{Connection, params};

use crate::errores::ErrorApp;

pub struct Apunte<'a> {
    pub cuando_en: &'a str,
    pub usuario_id: Option<i64>,
    pub accion: &'a str,
    pub entidad: &'a str,
    pub entidad_id: Option<i64>,
}

pub fn registrar(con: &Connection, apunte: &Apunte) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO auditoria (cuando_en, usuario_id, accion, entidad, entidad_id) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            apunte.cuando_en,
            apunte.usuario_id,
            apunte.accion,
            apunte.entidad,
            apunte.entidad_id
        ],
    )?;
    Ok(())
}
