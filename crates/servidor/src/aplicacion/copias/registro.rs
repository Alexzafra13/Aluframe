//! Lo que pasó en la última copia, en un archivo junto a las copias locales y no en la base:
//! es de este equipo y debe seguir ahí aunque se restaure una copia antigua.

use std::path::Path;

use serde::{Deserialize, Serialize};

const ARCHIVO: &str = "registro.json";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistroDeCopias {
    /// Instante UTC de la última copia guardada y comprobada en al menos un disco.
    pub ultima_completa: Option<String>,
    pub ultimo_intento: Option<String>,
    /// Qué falló en el último intento; `None` si salió bien.
    pub aviso: Option<String>,
    /// Los discos en que se guardó la última copia.
    pub discos: Vec<String>,
}

pub fn leer_registro(carpeta: &Path) -> RegistroDeCopias {
    std::fs::read_to_string(carpeta.join(ARCHIVO))
        .ok()
        .and_then(|texto| serde_json::from_str(&texto).ok())
        .unwrap_or_default()
}

pub fn guardar_registro(carpeta: &Path, registro: &RegistroDeCopias) -> std::io::Result<()> {
    let texto = serde_json::to_string_pretty(registro).map_err(std::io::Error::other)?;
    let temporal = carpeta.join(format!("{ARCHIVO}.tmp"));
    std::fs::write(&temporal, texto)?;
    std::fs::rename(temporal, carpeta.join(ARCHIVO))
}
