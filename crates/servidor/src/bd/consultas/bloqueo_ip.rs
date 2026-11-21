use rusqlite::{Connection, OptionalExtension, params};

use crate::errores::ErrorApp;

pub struct FilaBloqueoIp {
    pub intentos_fallidos: u32,
    pub bloqueado_hasta: Option<String>,
}

pub fn buscar(con: &Connection, ip: &str) -> Result<Option<FilaBloqueoIp>, ErrorApp> {
    Ok(con
        .query_row(
            "SELECT intentos_fallidos, bloqueado_hasta FROM bloqueo_ip WHERE ip = ?1",
            [ip],
            |fila| {
                Ok(FilaBloqueoIp {
                    intentos_fallidos: fila.get(0)?,
                    bloqueado_hasta: fila.get(1)?,
                })
            },
        )
        .optional()?)
}

pub fn anotar_fallo(
    con: &Connection,
    ip: &str,
    bloqueado_hasta: Option<&str>,
) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO bloqueo_ip (ip, intentos_fallidos, bloqueado_hasta) VALUES (?1, 1, ?2) \
         ON CONFLICT (ip) DO UPDATE SET intentos_fallidos = intentos_fallidos + 1, \
         bloqueado_hasta = ?2",
        params![ip, bloqueado_hasta],
    )?;
    Ok(())
}

pub fn olvidar(con: &Connection, ip: &str) -> Result<(), ErrorApp> {
    con.execute("DELETE FROM bloqueo_ip WHERE ip = ?1", [ip])?;
    Ok(())
}
