//! Calles sugeridas al escribir una dirección (ADR-0023). El callejero viene con el programa y
//! se lee una vez; por ahora, solo la provincia de Badajoz, donde trabaja el taller.

use std::sync::OnceLock;

use dominio::callejero::{Callejero, DondeBuscar};
use serde::Serialize;

const BADAJOZ: &str = include_str!("../../../../data/callejero/06.tsv");

/// Las que caben en la lista del campo sin desplazarse mucho en la tablet.
const SUGERENCIAS_MAXIMAS: usize = 8;

fn callejero() -> &'static Callejero {
    static CALLEJERO: OnceLock<Callejero> = OnceLock::new();
    CALLEJERO.get_or_init(|| {
        let mut callejero = Callejero::default();
        callejero.anadir(BADAJOZ, "Badajoz");
        callejero
    })
}

/// Al elegirla, la página escribe la calle y rellena localidad, provincia, país y el código
/// postal si la calle tiene uno solo; con varios, los ofrece (como con las localidades).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SugerenciaDeCalle {
    pub calle: String,
    pub localidad: String,
    pub provincia: String,
    pub pais: String,
    pub codigos_postales: Vec<String>,
}

pub fn sugerir_calles(
    escrito: &str,
    localidad: &str,
    codigo_postal: &str,
) -> Vec<SugerenciaDeCalle> {
    let donde = DondeBuscar {
        localidad,
        codigo_postal,
    };
    callejero()
        .sugerir(escrito, donde, SUGERENCIAS_MAXIMAS)
        .into_iter()
        .map(|s| SugerenciaDeCalle {
            calle: s.calle,
            localidad: s.localidad.to_owned(),
            provincia: s.provincia.to_owned(),
            pais: "España".to_owned(),
            codigos_postales: s.codigos_postales.to_vec(),
        })
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_callejero_incluido_se_lee_entero() {
        assert!(
            callejero().len() > 15_000,
            "solo {} calles",
            callejero().len()
        );
    }

    #[test]
    fn en_zafra_sugiere_sus_calles_con_el_06300() {
        let calles = sugerir_calles("duques de feria", "Zafra", "");
        assert_eq!(calles[0].calle, "Avenida Duques de Feria");
        assert_eq!(calles[0].localidad, "Zafra");
        assert_eq!(calles[0].provincia, "Badajoz");
        assert_eq!(calles[0].codigos_postales, ["06300"]);
        assert!(
            sugerir_calles("calle", "Zafra", "")
                .iter()
                .all(|c| c.localidad == "Zafra")
        );
    }

    #[test]
    fn en_badajoz_cada_calle_trae_su_codigo_y_las_pedanias_tienen_las_suyas() {
        let calles = sugerir_calles("calle", "Badajoz", "");
        assert!(calles.iter().all(|c| c.localidad == "Badajoz"));
        let codigos: std::collections::BTreeSet<_> = sugerir_calles("avenida", "Badajoz", "")
            .into_iter()
            .flat_map(|c| c.codigos_postales)
            .collect();
        assert!(codigos.iter().all(|c| c.starts_with("060")), "{codigos:?}");
        assert!(!sugerir_calles("calle", "Valuengo", "").is_empty());
        assert!(sugerir_calles("calle", "Sevilla", "").is_empty());
    }

    #[test]
    fn los_santos_de_maimona_se_encuentra_con_el_articulo_delante() {
        assert!(!sugerir_calles("calle", "Los Santos de Maimona", "").is_empty());
    }
}
