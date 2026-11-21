use dominio::acceso::Rol;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaUsuario {
    pub id: i64,
    pub nombre: String,
    pub rol: Rol,
    pub clave_hash: String,
    pub activo: bool,
    pub intentos_fallidos: u32,
    pub bloqueado_hasta: Option<String>,
}

const COLUMNAS: &str = "id, nombre, rol, clave_hash, activo, intentos_fallidos, bloqueado_hasta";

fn leer_fila(fila: &Row) -> rusqlite::Result<(FilaUsuario, String)> {
    let rol: String = fila.get(2)?;
    let usuario = FilaUsuario {
        id: fila.get(0)?,
        nombre: fila.get(1)?,
        rol: Rol::Operario,
        clave_hash: fila.get(3)?,
        activo: fila.get(4)?,
        intentos_fallidos: fila.get(5)?,
        bloqueado_hasta: fila.get(6)?,
    };
    Ok((usuario, rol))
}

fn con_rol((mut usuario, rol): (FilaUsuario, String)) -> Result<FilaUsuario, ErrorApp> {
    usuario.rol = Rol::desde_texto(&rol)?;
    Ok(usuario)
}

pub fn contar_usuarios(con: &Connection) -> Result<i64, ErrorApp> {
    Ok(con.query_row("SELECT count(*) FROM usuario", [], |fila| fila.get(0))?)
}

pub fn buscar_por_id(con: &Connection, id: i64) -> Result<Option<FilaUsuario>, ErrorApp> {
    let sql = format!("SELECT {COLUMNAS} FROM usuario WHERE id = ?1");
    con.query_row(&sql, [id], leer_fila)
        .optional()?
        .map(con_rol)
        .transpose()
}

pub fn buscar_por_nombre(con: &Connection, nombre: &str) -> Result<Option<FilaUsuario>, ErrorApp> {
    let sql = format!("SELECT {COLUMNAS} FROM usuario WHERE nombre = ?1");
    con.query_row(&sql, [nombre], leer_fila)
        .optional()?
        .map(con_rol)
        .transpose()
}

pub fn listar(con: &Connection, solo_activos: bool) -> Result<Vec<FilaUsuario>, ErrorApp> {
    let sql = format!(
        "SELECT {COLUMNAS} FROM usuario WHERE activo = 1 OR ?1 = 0 \
         ORDER BY rol = 'operario', nombre"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([solo_activos], leer_fila)?;
    filas.map(|fila| con_rol(fila?)).collect()
}

pub fn existe_nombre(con: &Connection, nombre: &str) -> Result<bool, ErrorApp> {
    Ok(con.query_row(
        "SELECT EXISTS (SELECT 1 FROM usuario WHERE nombre = ?1)",
        [nombre],
        |fila| fila.get(0),
    )?)
}

pub struct NuevoUsuario<'a> {
    pub nombre: &'a str,
    pub rol: Rol,
    pub clave_hash: &'a str,
    pub creado_en: &'a str,
}

pub fn insertar(con: &Connection, nuevo: &NuevoUsuario) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO usuario (nombre, rol, clave_hash, creado_en) VALUES (?1, ?2, ?3, ?4)",
        params![
            nuevo.nombre,
            nuevo.rol.como_texto(),
            nuevo.clave_hash,
            nuevo.creado_en
        ],
    )?;
    Ok(con.last_insert_rowid())
}

pub fn anotar_fallo(
    con: &Connection,
    id: i64,
    bloqueado_hasta: Option<&str>,
) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE usuario SET intentos_fallidos = intentos_fallidos + 1, \
         bloqueado_hasta = ?2 WHERE id = ?1",
        params![id, bloqueado_hasta],
    )?;
    Ok(())
}

pub fn reiniciar_fallos(con: &Connection, id: i64) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE usuario SET intentos_fallidos = 0, bloqueado_hasta = NULL WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

pub fn cambiar_clave(con: &Connection, id: i64, clave_hash: &str) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE usuario SET clave_hash = ?2, intentos_fallidos = 0, bloqueado_hasta = NULL, \
         version = version + 1 WHERE id = ?1",
        params![id, clave_hash],
    )?;
    Ok(())
}

pub fn cambiar_activo(con: &Connection, id: i64, activo: bool) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE usuario SET activo = ?2, version = version + 1 WHERE id = ?1",
        params![id, activo],
    )?;
    Ok(())
}
