//! Lo que se rellena solo al escribir un código postal o una localidad (ADR-0020). El listado
//! viene con el programa y se lee una vez; el servidor puede no tener internet.

use std::sync::OnceLock;

use dominio::codigos_postales::{IndiceDeCodigosPostales, provincia_de_codigo_postal};
use dominio::localidades::IndiceDeLocalidades;
use serde::Serialize;

const ARCHIVO: &str = include_str!("../../../../data/codigos_postales.tsv");

pub fn indice() -> &'static IndiceDeCodigosPostales {
    static INDICE: OnceLock<IndiceDeCodigosPostales> = OnceLock::new();
    INDICE.get_or_init(|| IndiceDeCodigosPostales::leer(ARCHIVO))
}

fn indice_de_localidades() -> &'static IndiceDeLocalidades {
    static INDICE: OnceLock<IndiceDeLocalidades> = OnceLock::new();
    INDICE.get_or_init(|| IndiceDeLocalidades::leer(ARCHIVO))
}

/// Las que caben en la lista del campo sin desplazarse mucho en la tablet.
const SUGERENCIAS_MAXIMAS: usize = 8;

/// Una opción de la lista de la localidad: al elegirla, la página escribe el código postal si
/// solo hay uno; con varios, deja el campo vacío y los ofrece (ADR-0020).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SugerenciaDeLocalidad {
    pub localidad: String,
    pub provincia: String,
    pub pais: String,
    pub codigos_postales: Vec<String>,
}

pub fn sugerir_localidades(escrito: &str) -> Vec<SugerenciaDeLocalidad> {
    indice_de_localidades()
        .sugerir(escrito, SUGERENCIAS_MAXIMAS)
        .into_iter()
        .map(|s| SugerenciaDeLocalidad {
            localidad: s.nombre.clone(),
            provincia: s.provincia.to_owned(),
            pais: "España".to_owned(),
            codigos_postales: s.codigos_postales.clone(),
        })
        .collect()
}

/// La página rellena solo los campos vacíos; con varias localidades, las ofrece para elegir.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct Poblacion {
    pub provincia: Option<String>,
    pub pais: Option<String>,
    pub localidades: Vec<String>,
    pub codigos_postales: Vec<String>,
}

pub fn buscar_por_codigo_postal(cp: &str) -> Poblacion {
    let Some(provincia) = provincia_de_codigo_postal(cp) else {
        return Poblacion::default();
    };
    Poblacion {
        provincia: Some(provincia.to_owned()),
        pais: Some("España".to_owned()),
        localidades: indice().localidades_de(cp).to_vec(),
        codigos_postales: Vec::new(),
    }
}

/// Con una localidad: su provincia solo si todos sus códigos son de la misma, y sus códigos
/// postales para ofrecerlos (no se escriben solos: el listado puede no tenerlos todos).
pub fn buscar_por_localidad(localidad: &str) -> Poblacion {
    let codigos = indice().codigos_de(localidad);
    let mut provincias: Vec<&str> = codigos
        .iter()
        .filter_map(|c| provincia_de_codigo_postal(c))
        .collect();
    provincias.dedup();
    if provincias.is_empty() {
        return Poblacion::default();
    }
    Poblacion {
        provincia: (provincias.len() == 1).then(|| provincias[0].to_owned()),
        pais: Some("España".to_owned()),
        localidades: Vec::new(),
        codigos_postales: codigos.to_vec(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_listado_incluido_se_lee_entero() {
        assert!(indice().len() > 10_000, "solo {} códigos", indice().len());
    }

    #[test]
    fn un_codigo_con_una_localidad_la_da_con_su_provincia() {
        let zafra = buscar_por_codigo_postal("06300");
        assert_eq!(zafra.provincia.as_deref(), Some("Badajoz"));
        assert_eq!(zafra.localidades, ["Zafra"]);
        assert_eq!(buscar_por_codigo_postal("123"), Poblacion::default());
    }

    #[test]
    fn una_localidad_con_un_codigo_lo_da_y_una_ciudad_grande_solo_la_provincia() {
        let zafra = buscar_por_localidad("zafra");
        assert_eq!(zafra.codigos_postales, ["06300"]);
        let madrid = buscar_por_localidad("Madrid");
        assert_eq!(madrid.provincia.as_deref(), Some("Madrid"));
        assert!(madrid.codigos_postales.len() > 1);
        assert_eq!(
            buscar_por_localidad("Sitio Que No Existe"),
            Poblacion::default()
        );
    }

    #[test]
    fn al_escribir_zaf_sugiere_primero_zafra_de_badajoz_con_su_codigo() {
        let sugerencias = sugerir_localidades("zaf");
        assert!(sugerencias.len() <= SUGERENCIAS_MAXIMAS);
        let zafra = &sugerencias[0];
        assert_eq!(zafra.localidad, "Zafra");
        assert_eq!(zafra.provincia, "Badajoz");
        assert_eq!(zafra.codigos_postales, ["06300"]);
        let madrid = sugerir_localidades("madrid");
        assert_eq!(madrid[0].provincia, "Madrid");
        assert!(madrid[0].codigos_postales.len() > 10);
    }
}
