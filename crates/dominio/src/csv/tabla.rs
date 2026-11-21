use crate::texto::{limpiar_opcional, normalizar_clave};

/// Una fila de datos con el número de línea del archivo donde empieza, para los mensajes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaCsv {
    pub linea: usize,
    pub campos: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablaCsv {
    /// Cabeceras normalizadas: «Descripción» se guarda como `descripcion`.
    pub cabecera: Vec<String>,
    pub filas: Vec<FilaCsv>,
}

impl TablaCsv {
    pub fn columna(&self, nombre: &str) -> Option<usize> {
        self.cabecera.iter().position(|c| c == nombre)
    }

    /// Cabeceras que no están en la lista de columnas conocidas.
    pub fn columnas_desconocidas(&self, conocidas: &[&str]) -> Vec<String> {
        self.cabecera
            .iter()
            .filter(|c| !c.is_empty() && !conocidas.contains(&c.as_str()))
            .cloned()
            .collect()
    }
}

impl FilaCsv {
    /// El campo de una columna, sin espacios a los lados; vacío o ausente es `None`.
    pub fn campo(&self, columna: Option<usize>) -> Option<String> {
        columna
            .and_then(|indice| self.campos.get(indice))
            .and_then(|valor| limpiar_opcional(valor))
    }
}

pub(super) fn crear_tabla(mut registros: Vec<FilaCsv>) -> Option<TablaCsv> {
    if registros.is_empty() {
        return None;
    }
    let cabecera = registros.remove(0);
    Some(TablaCsv {
        cabecera: cabecera
            .campos
            .iter()
            .map(|c| normalizar_clave(c))
            .collect(),
        filas: registros,
    })
}
