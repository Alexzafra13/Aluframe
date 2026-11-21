use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::autenticacion::calcular_sha256;

const ESQUEMA: &str = include_str!("esquema.sql");

#[derive(Debug, Error)]
pub enum ErrorEsquema {
    #[error(
        "La base de datos es de otra versión de desarrollo de AlumFrame. Mientras no esté instalado en el taller, borra la carpeta de datos y vuelve a arrancar."
    )]
    EsquemaDistinto,
    #[error("Error de SQLite al preparar la base de datos: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub fn huella_del_esquema() -> String {
    calcular_sha256(ESQUEMA)
}

/// Crea el esquema en una base vacía o comprueba que la existente es exactamente este esquema.
pub fn preparar_esquema(conexion: &mut Connection, ahora_utc: &str) -> Result<(), ErrorEsquema> {
    match leer_huella_guardada(conexion)? {
        Some(huella) if huella == huella_del_esquema() => Ok(()),
        Some(_) => Err(ErrorEsquema::EsquemaDistinto),
        None if contar_tablas(conexion)? > 0 => Err(ErrorEsquema::EsquemaDistinto),
        None => crear(conexion, ahora_utc),
    }
}

fn crear(conexion: &mut Connection, ahora_utc: &str) -> Result<(), ErrorEsquema> {
    let transaccion = conexion.transaction()?;
    transaccion.execute_batch(ESQUEMA)?;
    transaccion.execute(
        "INSERT INTO esquema (huella, creado_en) VALUES (?1, ?2)",
        params![huella_del_esquema(), ahora_utc],
    )?;
    transaccion.commit()?;
    Ok(())
}

fn contar_tablas(conexion: &Connection) -> Result<i64, rusqlite::Error> {
    conexion.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |fila| fila.get(0),
    )
}

pub fn leer_huella_guardada(conexion: &Connection) -> Result<Option<String>, rusqlite::Error> {
    let existe: bool = conexion.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'esquema')",
        [],
        |fila| fila.get(0),
    )?;
    if !existe {
        return Ok(None);
    }
    conexion
        .query_row("SELECT huella FROM esquema", [], |fila| fila.get(0))
        .optional()
}

#[cfg(test)]
mod tests {
    use super::*;

    const AHORA: &str = "2026-11-21T10:00:00Z";

    fn preparada() -> Connection {
        let mut conexion = Connection::open_in_memory().unwrap();
        preparar_esquema(&mut conexion, AHORA).unwrap();
        conexion
    }

    #[test]
    fn crea_todas_las_tablas_en_una_base_vacia() {
        let conexion = preparada();
        for tabla in [
            "auditoria",
            "bloqueo_ip",
            "esquema",
            "serie",
            "serie_version",
            "sesion",
            "usuario",
        ] {
            let existe: bool = conexion
                .query_row(
                    "SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE name = ?1)",
                    [tabla],
                    |f| f.get(0),
                )
                .unwrap();
            assert!(existe, "{tabla}");
        }
        let integridad: String = conexion
            .query_row("PRAGMA integrity_check", [], |f| f.get(0))
            .unwrap();
        assert_eq!(integridad, "ok");
    }

    #[test]
    fn preparar_dos_veces_no_cambia_nada() {
        let mut conexion = preparada();
        assert!(preparar_esquema(&mut conexion, AHORA).is_ok());
    }

    #[test]
    fn rechaza_una_base_de_otro_esquema() {
        let mut conexion = preparada();
        conexion
            .execute("UPDATE esquema SET huella = ?1", [&"0".repeat(64)])
            .unwrap();
        assert!(matches!(
            preparar_esquema(&mut conexion, AHORA),
            Err(ErrorEsquema::EsquemaDistinto)
        ));
        let mut antigua = Connection::open_in_memory().unwrap();
        antigua
            .execute_batch("CREATE TABLE version_esquema (version INTEGER)")
            .unwrap();
        assert!(matches!(
            preparar_esquema(&mut antigua, AHORA),
            Err(ErrorEsquema::EsquemaDistinto)
        ));
    }

    #[test]
    fn la_auditoria_y_las_versiones_de_serie_no_se_modifican() {
        let conexion = preparada();
        conexion
            .execute(
                "INSERT INTO auditoria (cuando_en, accion, entidad) VALUES (?1, 'prueba', 'x')",
                [AHORA],
            )
            .unwrap();
        assert!(
            conexion
                .execute("UPDATE auditoria SET accion = 'otra'", [])
                .is_err()
        );
        assert!(conexion.execute("DELETE FROM auditoria", []).is_err());
        conexion
            .execute_batch(
                "INSERT INTO serie (codigo, fabricante, creada_en) VALUES ('S', 'F', 'x');
                 INSERT INTO serie_version (serie_id, numero, estado, origen_yaml, sha256, avisos, creada_en)
                 VALUES (1, 1, 'borrador', 'serie: S', printf('%064d', 0), 0, 'x');",
            )
            .unwrap();
        assert!(
            conexion
                .execute("UPDATE serie_version SET avisos = 1", [])
                .is_err()
        );
        assert!(conexion.execute("DELETE FROM serie_version", []).is_err());
    }
}
