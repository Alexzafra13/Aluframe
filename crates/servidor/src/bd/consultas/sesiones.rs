use rusqlite::{Connection, OptionalExtension, params};

use crate::errores::ErrorApp;

pub struct NuevaSesion<'a> {
    pub huella: &'a str,
    pub usuario_id: i64,
    pub creada_en: &'a str,
    pub caduca_en: &'a str,
    pub token_csrf: &'a str,
    pub hasta_cerrar_navegador: bool,
}

pub struct FilaSesion {
    pub usuario_id: i64,
    pub ultima_actividad_en: String,
    pub caduca_en: String,
    pub token_csrf: String,
    pub hasta_cerrar_navegador: bool,
}

pub fn insertar(con: &Connection, nueva: &NuevaSesion) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO sesion (id, usuario_id, creada_en, ultima_actividad_en, caduca_en, \
         token_csrf, hasta_cerrar_navegador) VALUES (?1, ?2, ?3, ?3, ?4, ?5, ?6)",
        params![
            nueva.huella,
            nueva.usuario_id,
            nueva.creada_en,
            nueva.caduca_en,
            nueva.token_csrf,
            nueva.hasta_cerrar_navegador
        ],
    )?;
    Ok(())
}

pub fn buscar(con: &Connection, huella: &str) -> Result<Option<FilaSesion>, ErrorApp> {
    Ok(con
        .query_row(
            "SELECT usuario_id, ultima_actividad_en, caduca_en, token_csrf, \
             hasta_cerrar_navegador FROM sesion WHERE id = ?1",
            [huella],
            |fila| {
                Ok(FilaSesion {
                    usuario_id: fila.get(0)?,
                    ultima_actividad_en: fila.get(1)?,
                    caduca_en: fila.get(2)?,
                    token_csrf: fila.get(3)?,
                    hasta_cerrar_navegador: fila.get(4)?,
                })
            },
        )
        .optional()?)
}

pub fn anotar_actividad(con: &Connection, huella: &str, ahora: &str) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE sesion SET ultima_actividad_en = ?2 WHERE id = ?1",
        params![huella, ahora],
    )?;
    Ok(())
}

pub fn borrar(con: &Connection, huella: &str) -> Result<(), ErrorApp> {
    con.execute("DELETE FROM sesion WHERE id = ?1", [huella])?;
    Ok(())
}

pub fn borrar_de_usuario(con: &Connection, usuario_id: i64) -> Result<(), ErrorApp> {
    con.execute("DELETE FROM sesion WHERE usuario_id = ?1", [usuario_id])?;
    Ok(())
}

pub fn borrar_caducadas(con: &Connection, ahora: &str) -> Result<(), ErrorApp> {
    con.execute("DELETE FROM sesion WHERE caduca_en <= ?1", [ahora])?;
    Ok(())
}
