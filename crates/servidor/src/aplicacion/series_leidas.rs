//! Las series guardadas, ya leídas. Una versión guardada no cambia nunca y volver a leerla (el
//! YAML y la prueba de todas sus combinaciones) cuesta unos 5 ms, que se pagaban en cada recálculo
//! del editor (ADR-0035). Se recuerda por el texto y no por el id: en un mismo proceso conviven la
//! base real y la de la demostración, cada una con sus ids.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use dominio::series::{Serie, importar_serie};

use crate::series::leer_yaml;

type Leidas = Mutex<HashMap<String, Serie>>;

fn leidas() -> &'static Leidas {
    static LEIDAS: OnceLock<Leidas> = OnceLock::new();
    LEIDAS.get_or_init(Leidas::default)
}

/// La serie de ese texto guardado; el error junta los del informe de importación.
pub(crate) fn leer_serie_guardada(origen_yaml: &str) -> Result<Serie, String> {
    let recordada = leidas()
        .lock()
        .ok()
        .and_then(|mapa| mapa.get(origen_yaml).cloned());
    if let Some(serie) = recordada {
        return Ok(serie);
    }
    let serie = leer_de_nuevo(origen_yaml)?;
    if let Ok(mut mapa) = leidas().lock() {
        mapa.insert(origen_yaml.to_owned(), serie.clone());
    }
    Ok(serie)
}

fn leer_de_nuevo(origen_yaml: &str) -> Result<Serie, String> {
    let valor = leer_yaml(origen_yaml)?;
    let (serie, _) = importar_serie(&valor).map_err(|informe| {
        informe
            .errores
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    Ok(serie)
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used)]
    use super::*;

    const ALUPROM: &str = include_str!("../../../../data/series/aluprom-21.yaml");

    #[test]
    fn la_serie_recordada_es_la_misma_que_leida_de_nuevo() {
        let primera = leer_serie_guardada(ALUPROM).unwrap();
        let recordada = leer_serie_guardada(ALUPROM).unwrap();
        assert_eq!(recordada, leer_de_nuevo(ALUPROM).unwrap());
        assert_eq!(primera, recordada);
    }

    #[test]
    fn un_texto_que_no_se_lee_da_error_y_no_se_recuerda() {
        assert!(leer_serie_guardada("codigo: [").is_err());
        let recordado = leidas().lock().unwrap().contains_key("codigo: [");
        assert!(!recordado);
    }
}
