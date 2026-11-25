//! Cuando la base de datos de este equipo no se puede usar con esta versión, la ventana lo
//! explica, dice dónde está la carpeta de datos y ofrece abrirla. Si la base es de una versión de
//! desarrollo antigua (no se puede actualizar), también ofrece empezar de cero: la base se aparta
//! a `apartadas`, nunca se borra, por si tuviera algo que hiciera falta.

use std::path::{Path, PathBuf};

use crate::bd::ErrorEsquema;

pub struct Recuperacion {
    pub carpeta: PathBuf,
    pub explicacion: String,
    pub puede_empezar_de_cero: bool,
}

impl Recuperacion {
    pub fn desde(error: &ErrorEsquema, carpeta: &Path) -> Recuperacion {
        let (explicacion, puede_empezar_de_cero) = match error {
            ErrorEsquema::Desconocida => (
                "La base de datos de este equipo es de una versión de desarrollo antigua de \
                 Aluframe y no se puede actualizar."
                    .to_owned(),
                true,
            ),
            ErrorEsquema::MasNueva { .. } => (
                "La base de datos de este equipo es de una versión más nueva de Aluframe. \
                 Instala la versión más reciente: así se abre con todos sus datos."
                    .to_owned(),
                false,
            ),
            ErrorEsquema::Sqlite(e) => (format!("No se pudo abrir la base de datos: {e}."), false),
        };
        Recuperacion {
            carpeta: carpeta.to_path_buf(),
            explicacion,
            puede_empezar_de_cero,
        }
    }
}

/// Mueve la base (y sus archivos de diario) a `apartadas/alumframe-<marca>.db`. Lo demás de la
/// carpeta (copias, demostración) no se toca. Devuelve dónde ha quedado.
pub fn apartar_base(carpeta: &Path, marca: &str) -> std::io::Result<PathBuf> {
    let apartadas = carpeta.join("apartadas");
    std::fs::create_dir_all(&apartadas)?;
    let destino = apartadas.join(format!("alumframe-{marca}.db"));
    for sufijo in ["", "-wal", "-shm"] {
        let origen = carpeta.join(format!("alumframe.db{sufijo}"));
        if origen.exists() {
            let nombre = format!("alumframe-{marca}.db{sufijo}");
            std::fs::rename(&origen, apartadas.join(nombre))?;
        }
    }
    Ok(destino)
}

/// Abre la carpeta en el explorador de archivos del sistema.
pub fn abrir_carpeta(carpeta: &Path) {
    let programa = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    if let Err(error) = std::process::Command::new(programa).arg(carpeta).spawn() {
        tracing::warn!("no se pudo abrir la carpeta {}: {error}", carpeta.display());
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn aparta_la_base_y_sus_diarios_sin_tocar_lo_demas() {
        let carpeta =
            std::env::temp_dir().join(format!("alumframe-apartar-{}", std::process::id()));
        std::fs::create_dir_all(carpeta.join("copias")).unwrap();
        for nombre in ["alumframe.db", "alumframe.db-wal", "copias/una.alumframe"] {
            std::fs::write(carpeta.join(nombre), b"x").unwrap();
        }
        let destino = apartar_base(&carpeta, "2026-11-23T16-00-00Z").unwrap();
        assert!(destino.exists());
        assert!(
            carpeta
                .join("apartadas/alumframe-2026-11-23T16-00-00Z.db-wal")
                .exists()
        );
        assert!(!carpeta.join("alumframe.db").exists());
        assert!(
            carpeta.join("copias/una.alumframe").exists(),
            "las copias no se tocan"
        );
        let _ = std::fs::remove_dir_all(&carpeta);
    }

    #[test]
    fn solo_se_ofrece_empezar_de_cero_con_una_base_antigua() {
        let carpeta = Path::new("datos");
        assert!(Recuperacion::desde(&ErrorEsquema::Desconocida, carpeta).puede_empezar_de_cero);
        let nueva = ErrorEsquema::MasNueva {
            base: 9,
            programa: 4,
        };
        let recuperacion = Recuperacion::desde(&nueva, carpeta);
        assert!(!recuperacion.puede_empezar_de_cero);
        assert!(recuperacion.explicacion.contains("más reciente"));
    }
}
