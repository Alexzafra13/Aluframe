//! Migraciones numeradas (ADR-0021, sustituye al esquema único de ADR-0011). Cada una se aplica
//! en una transacción y **nunca se edita** una vez publicada: un cambio es otra migración.
//! La versión de la base es `PRAGMA user_version`, que va en la misma transacción.

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::autenticacion::calcular_sha256;

#[derive(Debug, Clone, Copy)]
pub struct Migracion {
    pub numero: u32,
    pub sql: &'static str,
}

pub const MIGRACIONES: &[Migracion] = &[
    Migracion {
        numero: 1,
        sql: include_str!("migraciones/0001_inicial.sql"),
    },
    Migracion {
        numero: 2,
        sql: include_str!("migraciones/0002_permisos_de_operario.sql"),
    },
    Migracion {
        numero: 3,
        sql: include_str!("migraciones/0003_apuntes.sql"),
    },
    Migracion {
        numero: 4,
        sql: include_str!("migraciones/0004_validez_de_presupuestos.sql"),
    },
];

/// Huella del esquema único de las versiones 0.8.0 a 0.10.0, que es la migración 0001: sus
/// bases se adoptan como versión 1 sin tocar los datos.
const HUELLA_DE_LA_0001: &str = "56113df2d94d8fdd5eab5a3bc3a0512a092b6124c6e6515eda162bcfed4bc70e";
/// La misma, del archivo con finales de línea CRLF: así la guardaron las versiones de Windows,
/// compiladas con el código sacado por Git en Windows.
const HUELLA_DE_LA_0001_EN_WINDOWS: &str =
    "22d808e2a546eb8d641fe2815d785971d1a67d7988e7fdd360e82f432e5c41f9";

#[derive(Debug, Error)]
pub enum ErrorEsquema {
    #[error(
        "La base de datos es de una versión más nueva de AlumFrame (esquema {base}; este programa \
         llega al {programa}). Instala la versión más reciente."
    )]
    MasNueva { base: u32, programa: u32 },
    #[error(
        "La base de datos es de una versión de desarrollo antigua de AlumFrame y no se puede \
         actualizar. Si no tiene datos reales, borra la carpeta de datos y vuelve a arrancar."
    )]
    Desconocida,
    #[error("Error de SQLite al preparar la base de datos: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub fn version_del_programa() -> u32 {
    MIGRACIONES.last().map_or(0, |m| m.numero)
}

/// La versión del esquema de una base; 0 si está vacía y `None` si no es de AlumFrame o es de
/// un desarrollo anterior a las migraciones.
pub fn version_de_la_base(con: &Connection) -> Result<Option<u32>, rusqlite::Error> {
    let version: u32 = con.query_row("PRAGMA user_version", [], |f| f.get(0))?;
    if version > 0 {
        return Ok(Some(version));
    }
    if contar_tablas(con)? == 0 {
        return Ok(Some(0));
    }
    let adoptable = leer_huella_guardada(con)?
        .is_some_and(|h| h == HUELLA_DE_LA_0001 || h == HUELLA_DE_LA_0001_EN_WINDOWS);
    Ok(adoptable.then_some(1))
}

/// Las migraciones que faltan por aplicar a esta base.
pub fn migraciones_pendientes(con: &Connection) -> Result<u32, ErrorEsquema> {
    let base = comprobar_version(con)?;
    Ok(version_del_programa() - base)
}

fn comprobar_version(con: &Connection) -> Result<u32, ErrorEsquema> {
    let programa = version_del_programa();
    match version_de_la_base(con)? {
        Some(base) if base > programa => Err(ErrorEsquema::MasNueva { base, programa }),
        Some(base) => Ok(base),
        None => Err(ErrorEsquema::Desconocida),
    }
}

/// Crea la base o la lleva a la versión de este programa.
pub fn preparar_esquema(conexion: &mut Connection, ahora_utc: &str) -> Result<(), ErrorEsquema> {
    aplicar_migraciones(conexion, MIGRACIONES, ahora_utc)
}

/// Aplica en orden las de `migraciones` que faltan. Público para probar migraciones nuevas.
pub fn aplicar_migraciones(
    conexion: &mut Connection,
    migraciones: &[Migracion],
    ahora_utc: &str,
) -> Result<(), ErrorEsquema> {
    let programa = migraciones.last().map_or(0, |m| m.numero);
    let base = match version_de_la_base(conexion)? {
        Some(base) if base > programa => return Err(ErrorEsquema::MasNueva { base, programa }),
        Some(base) => base,
        None => return Err(ErrorEsquema::Desconocida),
    };
    if base == 1 {
        // Base de 0.8.0–0.10.0 (sin número): se anota que ya está en la versión 1.
        conexion.pragma_update(None, "user_version", 1)?;
    }
    for migracion in migraciones.iter().filter(|m| m.numero > base) {
        let transaccion = conexion.transaction()?;
        transaccion.execute_batch(migracion.sql)?;
        if migracion.numero == 1 {
            transaccion.execute(
                "INSERT INTO esquema (huella, creado_en) VALUES (?1, ?2)",
                params![HUELLA_DE_LA_0001, ahora_utc],
            )?;
        } else {
            // Las versiones 0.10.0 y anteriores solo miran esta huella: así no abren una base
            // que ya no entienden.
            transaccion.execute(
                "UPDATE esquema SET huella = ?1",
                [calcular_sha256(&format!("migracion-{}", migracion.numero))],
            )?;
        }
        transaccion.pragma_update(None, "user_version", migracion.numero)?;
        transaccion.commit()?;
    }
    Ok(())
}

fn contar_tablas(conexion: &Connection) -> Result<i64, rusqlite::Error> {
    conexion.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |fila| fila.get(0),
    )
}

fn leer_huella_guardada(conexion: &Connection) -> Result<Option<String>, rusqlite::Error> {
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
#[path = "esquema_pruebas.rs"]
mod pruebas;
