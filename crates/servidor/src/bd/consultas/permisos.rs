//! Permisos extra de cada operario (ADR-0024).

use dominio::acceso::{Permiso, Permisos};
use rusqlite::{Connection, params};

use crate::errores::ErrorApp;

/// Un permiso guardado que este programa no conoce (de una versión más nueva) se ignora: no da
/// acceso a nada.
pub fn leer(con: &Connection, usuario_id: i64) -> Result<Permisos, ErrorApp> {
    let mut consulta = con.prepare("SELECT permiso FROM usuario_permiso WHERE usuario_id = ?1")?;
    let textos = consulta.query_map([usuario_id], |f| f.get::<_, String>(0))?;
    let mut permisos = Vec::new();
    for texto in textos {
        if let Ok(permiso) = Permiso::desde_texto(&texto?) {
            permisos.push(permiso);
        }
    }
    Ok(Permisos::con(permisos))
}

pub fn guardar(con: &Connection, usuario_id: i64, permisos: &Permisos) -> Result<(), ErrorApp> {
    con.execute(
        "DELETE FROM usuario_permiso WHERE usuario_id = ?1",
        [usuario_id],
    )?;
    let mut insercion =
        con.prepare("INSERT INTO usuario_permiso (usuario_id, permiso) VALUES (?1, ?2)")?;
    for permiso in permisos.iter() {
        insercion.execute(params![usuario_id, permiso.como_texto()])?;
    }
    Ok(())
}
