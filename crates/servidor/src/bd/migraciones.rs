use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

struct Migracion {
    version: i64,
    nombre: &'static str,
    sql: &'static str,
}

const MIGRACIONES: &[Migracion] = &[Migracion {
    version: 1,
    nombre: "inicial",
    sql: include_str!("migraciones/0001_inicial.sql"),
}];

pub const ULTIMA_VERSION: i64 = 1;

#[derive(Debug, Error)]
pub enum ErrorMigracion {
    #[error(
        "La base de datos es de una versión más nueva ({en_base}) que este programa ({programa}). Instala la versión correcta de AlumFrame."
    )]
    BaseMasNueva { en_base: i64, programa: i64 },
    #[error("No se pudo hacer la copia previa a la migración: {0}")]
    CopiaPrevia(String),
    #[error("Error de SQLite al migrar: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Aplica las migraciones pendientes; si la base ya tenía datos, antes guarda una copia.
pub fn migrar(
    conexion: &mut Connection,
    copia_previa: Option<&Path>,
    ahora_utc: &str,
) -> Result<i64, ErrorMigracion> {
    crear_tabla_de_versiones(conexion)?;
    let actual = leer_version_actual(conexion)?;
    if actual > ULTIMA_VERSION {
        return Err(ErrorMigracion::BaseMasNueva {
            en_base: actual,
            programa: ULTIMA_VERSION,
        });
    }
    let pendientes: Vec<&Migracion> = MIGRACIONES.iter().filter(|m| m.version > actual).collect();
    if actual > 0
        && !pendientes.is_empty()
        && let Some(ruta) = copia_previa
    {
        copiar_antes_de_migrar(conexion, ruta)?;
    }
    for migracion in pendientes {
        aplicar(conexion, migracion, ahora_utc)?;
    }
    Ok(leer_version_actual(conexion)?)
}

fn crear_tabla_de_versiones(conexion: &Connection) -> Result<(), rusqlite::Error> {
    conexion.execute_batch(
        "CREATE TABLE IF NOT EXISTS version_esquema (
            version INTEGER PRIMARY KEY,
            nombre TEXT NOT NULL,
            aplicada_en TEXT NOT NULL
        ) STRICT;",
    )
}

fn leer_version_actual(conexion: &Connection) -> Result<i64, rusqlite::Error> {
    let version: Option<i64> = conexion
        .query_row("SELECT max(version) FROM version_esquema", [], |fila| {
            fila.get(0)
        })
        .optional()?
        .flatten();
    Ok(version.unwrap_or(0))
}

fn aplicar(
    conexion: &mut Connection,
    migracion: &Migracion,
    ahora_utc: &str,
) -> Result<(), rusqlite::Error> {
    let transaccion = conexion.transaction()?;
    transaccion.execute_batch(migracion.sql)?;
    transaccion.execute(
        "INSERT INTO version_esquema (version, nombre, aplicada_en) VALUES (?1, ?2, ?3)",
        params![migracion.version, migracion.nombre, ahora_utc],
    )?;
    transaccion.commit()
}

fn copiar_antes_de_migrar(conexion: &Connection, destino: &Path) -> Result<(), ErrorMigracion> {
    if let Some(carpeta) = destino.parent() {
        std::fs::create_dir_all(carpeta)
            .map_err(|error| ErrorMigracion::CopiaPrevia(error.to_string()))?;
    }
    let ruta = destino
        .to_str()
        .ok_or_else(|| ErrorMigracion::CopiaPrevia("ruta no válida".into()))?;
    conexion.execute("VACUUM INTO ?1", [ruta])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AHORA: &str = "2026-11-21T10:00:00Z";

    fn tablas(conexion: &Connection) -> Vec<String> {
        let mut consulta = conexion
            .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")
            .unwrap();
        consulta
            .query_map([], |fila| fila.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn migra_desde_cero_hasta_la_ultima_version() {
        let mut conexion = Connection::open_in_memory().unwrap();
        assert_eq!(migrar(&mut conexion, None, AHORA).unwrap(), ULTIMA_VERSION);
        for tabla in [
            "auditoria",
            "bloqueo_ip",
            "sesion",
            "usuario",
            "version_esquema",
        ] {
            assert!(tablas(&conexion).contains(&tabla.to_owned()), "{tabla}");
        }
        let integridad: String = conexion
            .query_row("PRAGMA integrity_check", [], |fila| fila.get(0))
            .unwrap();
        assert_eq!(integridad, "ok");
    }

    #[test]
    fn migrar_dos_veces_no_cambia_nada() {
        let mut conexion = Connection::open_in_memory().unwrap();
        migrar(&mut conexion, None, AHORA).unwrap();
        assert_eq!(migrar(&mut conexion, None, AHORA).unwrap(), ULTIMA_VERSION);
    }

    #[test]
    fn no_arranca_con_una_base_mas_nueva() {
        let mut conexion = Connection::open_in_memory().unwrap();
        migrar(&mut conexion, None, AHORA).unwrap();
        conexion
            .execute(
                "INSERT INTO version_esquema VALUES (99, 'futura', ?1)",
                [AHORA],
            )
            .unwrap();
        assert!(matches!(
            migrar(&mut conexion, None, AHORA),
            Err(ErrorMigracion::BaseMasNueva { en_base: 99, .. })
        ));
    }

    #[test]
    fn la_auditoria_no_se_puede_modificar_ni_borrar() {
        let mut conexion = Connection::open_in_memory().unwrap();
        migrar(&mut conexion, None, AHORA).unwrap();
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
    }
}
