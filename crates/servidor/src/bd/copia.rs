//! Copia completa de la base de datos para exportar e importar (ADR-0016). Nunca se copia el
//! archivo en caliente: se exporta con `VACUUM INTO` y se restaura con la API de copia de SQLite.

use std::path::Path;

use rusqlite::{Connection, MAIN_DB, OpenFlags, OptionalExtension};

use super::esquema::{huella_del_esquema, leer_huella_guardada};
use crate::errores::ErrorApp;

const CABECERA_SQLITE: &[u8; 16] = b"SQLite format 3\0";

/// Lo justo para reconocer una copia antes de sustituir nada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumenDeDatos {
    pub usuarios: i64,
    pub clientes: i64,
    pub presupuestos: i64,
    pub series: i64,
    pub ultimo_cambio: Option<String>,
}

/// Escribe una copia coherente en `destino`, que no debe existir.
pub fn exportar(con: &Connection, destino: &Path) -> Result<(), ErrorApp> {
    let ruta = destino
        .to_str()
        .ok_or_else(|| ErrorApp::Interno("ruta de la copia no válida".into()))?;
    con.execute("VACUUM INTO ?1", [ruta])?;
    Ok(())
}

pub fn resumir(con: &Connection) -> Result<ResumenDeDatos, ErrorApp> {
    let contar = |tabla: &str| -> Result<i64, ErrorApp> {
        Ok(con.query_row(&format!("SELECT count(*) FROM {tabla}"), [], |f| f.get(0))?)
    };
    Ok(ResumenDeDatos {
        usuarios: contar("usuario")?,
        clientes: contar("cliente")?,
        presupuestos: contar("presupuesto")?,
        series: contar("serie")?,
        ultimo_cambio: con
            .query_row("SELECT max(cuando_en) FROM auditoria", [], |f| f.get(0))
            .optional()?
            .flatten(),
    })
}

/// Comprueba que el archivo es una copia íntegra de AlumFrame con este mismo esquema.
pub fn comprobar_copia(ruta: &Path) -> Result<ResumenDeDatos, ErrorApp> {
    if !empieza_como_sqlite(ruta) {
        return Err(no_es_copia());
    }
    let con = Connection::open_with_flags(ruta, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| no_es_copia())?;
    let integridad: String = con
        .query_row("PRAGMA integrity_check", [], |f| f.get(0))
        .map_err(|_| no_es_copia())?;
    if integridad != "ok" {
        return Err(ErrorApp::Validacion(
            "El archivo está dañado: no se puede importar.".into(),
        ));
    }
    match leer_huella_guardada(&con).map_err(|_| no_es_copia())? {
        Some(huella) if huella == huella_del_esquema() => resumir(&con),
        Some(_) => Err(ErrorApp::Validacion(
            "La copia es de otra versión de AlumFrame: expórtala e impórtala con la misma versión."
                .into(),
        )),
        None => Err(no_es_copia()),
    }
}

/// Sustituye todo el contenido de `con` por el de la copia (ya comprobada).
pub fn restaurar(con: &mut Connection, origen: &Path) -> Result<(), ErrorApp> {
    con.restore(MAIN_DB, origen, None::<fn(rusqlite::backup::Progress)>)?;
    // La copia trae el modo de diario de un archivo suelto; la base en uso sigue en WAL.
    con.pragma_update(None, "journal_mode", "WAL")?;
    Ok(())
}

fn empieza_como_sqlite(ruta: &Path) -> bool {
    use std::io::Read;
    let mut cabecera = [0u8; 16];
    std::fs::File::open(ruta)
        .and_then(|mut archivo| archivo.read_exact(&mut cabecera))
        .is_ok_and(|()| &cabecera == CABECERA_SQLITE)
}

fn no_es_copia() -> ErrorApp {
    ErrorApp::Validacion(
        "El archivo no es una copia de AlumFrame (se exporta en Ajustes → Datos).".into(),
    )
}
