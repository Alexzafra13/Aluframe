use rusqlite::{Connection, OptionalExtension, params};

use crate::errores::ErrorApp;

pub fn leer(con: &Connection, clave: &str) -> Result<Option<String>, ErrorApp> {
    Ok(con
        .query_row(
            "SELECT valor FROM configuracion WHERE clave = ?1",
            [clave],
            |f| f.get(0),
        )
        .optional()?)
}

pub struct Valor<'a> {
    pub clave: &'a str,
    pub valor: &'a str,
    pub ahora: &'a str,
    pub usuario_id: i64,
}

pub fn guardar(con: &Connection, nuevo: &Valor) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO configuracion (clave, valor, actualizado_en, usuario_id) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT (clave) DO UPDATE SET valor = ?2, actualizado_en = ?3, usuario_id = ?4",
        params![nuevo.clave, nuevo.valor, nuevo.ahora, nuevo.usuario_id],
    )?;
    Ok(())
}
