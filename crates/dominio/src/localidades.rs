//! Sugerencias de localidad mientras se escribe (ADR-0020): las más parecidas a lo escrito, con
//! su provincia (para distinguir pueblos con el mismo nombre) y sus códigos postales.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use crate::codigos_postales::{provincia_de_codigo_postal, renglones};
use crate::texto::normalizar_para_buscar;

/// Con menos letras saldrían miles de localidades sin orden útil.
const LETRAS_MINIMAS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SugerenciaDeLocalidad {
    pub nombre: String,
    pub provincia: &'static str,
    /// Si es municipio, solo los códigos donde lo es (como en `IndiceDeCodigosPostales::codigos_de`).
    pub codigos_postales: Vec<String>,
    pub es_municipio: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IndiceDeLocalidades {
    entradas: Vec<(String, SugerenciaDeLocalidad)>,
}

#[derive(Default)]
struct Acumulado {
    nombre: Option<String>,
    como_municipio: Vec<String>,
    como_nucleo: Vec<String>,
}

impl IndiceDeLocalidades {
    /// Lee el mismo listado que `IndiceDeCodigosPostales` y agrupa por nombre y provincia.
    pub fn leer(texto: &str) -> IndiceDeLocalidades {
        let mut grupos: BTreeMap<(String, &'static str), Acumulado> = BTreeMap::new();
        for (cp, nombres) in renglones(texto) {
            let Some(provincia) = provincia_de_codigo_postal(&cp) else {
                continue;
            };
            for (posicion, nombre) in nombres.into_iter().enumerate() {
                let grupo = grupos
                    .entry((normalizar_para_buscar(&nombre), provincia))
                    .or_default();
                if posicion == 0 {
                    // El nombre del municipio manda sobre cómo esté escrito el núcleo.
                    grupo.nombre = Some(nombre);
                    grupo.como_municipio.push(cp.clone());
                } else {
                    grupo.nombre.get_or_insert(nombre);
                    grupo.como_nucleo.push(cp.clone());
                }
            }
        }
        let entradas = grupos
            .into_iter()
            .filter_map(|((clave, provincia), a)| {
                let es_municipio = !a.como_municipio.is_empty();
                let mut codigos = if es_municipio {
                    a.como_municipio
                } else {
                    a.como_nucleo
                };
                codigos.sort();
                codigos.dedup();
                let sugerencia = SugerenciaDeLocalidad {
                    nombre: a.nombre?,
                    provincia,
                    codigos_postales: codigos,
                    es_municipio,
                };
                Some((clave, sugerencia))
            })
            .collect();
        IndiceDeLocalidades { entradas }
    }

    /// Las `maximo` localidades más parecidas a lo escrito, sin distinguir tildes ni mayúsculas:
    /// primero el nombre exacto, luego las que empiezan así, luego las que tienen una palabra que
    /// empieza así y por último las que lo contienen. A igualdad, los municipios van antes.
    pub fn sugerir(&self, escrito: &str, maximo: usize) -> Vec<&SugerenciaDeLocalidad> {
        let buscado = normalizar_para_buscar(escrito);
        if buscado.chars().count() < LETRAS_MINIMAS {
            return Vec::new();
        }
        let mut encontradas: Vec<_> = self
            .entradas
            .iter()
            .filter_map(|(clave, s)| Some((parecido(clave, &buscado)?, clave, s)))
            .collect();
        encontradas.sort_by_key(|(parecido, clave, s)| {
            (*parecido, Reverse(s.es_municipio), clave.len(), *clave)
        });
        encontradas
            .into_iter()
            .take(maximo)
            .map(|(_, _, s)| s)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entradas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entradas.is_empty()
    }
}

/// Cuanto menor, más parecido; `None` si no lo contiene.
fn parecido(clave: &str, buscado: &str) -> Option<u8> {
    if clave == buscado {
        Some(0)
    } else if clave.starts_with(buscado) {
        Some(1)
    } else if clave
        .match_indices(buscado)
        .any(|(i, _)| clave[..i].ends_with(|c: char| !c.is_alphanumeric()))
    {
        Some(2)
    } else if clave.contains(buscado) {
        Some(3)
    } else {
        None
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const DATOS: &str = "# comentario\n06300\tZafra\n32510\tCarballiño, O|Zafra\n16192\tZafra de Záncara\n18128\tZafarraya\n04002\tAlmería|El Palmer\n04001\tAlmería\n06200\tAlmendralejo\n18129\tVentas de Zafarraya\n";

    fn nombres(indice: &IndiceDeLocalidades, escrito: &str) -> Vec<String> {
        indice
            .sugerir(escrito, 10)
            .iter()
            .map(|s| format!("{} ({})", s.nombre, s.provincia))
            .collect()
    }

    #[test]
    fn ordena_exacto_empieza_palabra_y_contiene_con_los_municipios_antes() {
        let indice = IndiceDeLocalidades::leer(DATOS);
        assert_eq!(
            nombres(&indice, "zaf"),
            [
                "Zafra (Badajoz)",
                "Zafarraya (Granada)",
                "Zafra de Záncara (Cuenca)",
                "Zafra (Ourense)",
                "Ventas de Zafarraya (Granada)",
            ]
        );
        assert_eq!(
            nombres(&indice, "ZAFRA")[..2],
            ["Zafra (Badajoz)", "Zafra (Ourense)"]
        );
        assert_eq!(nombres(&indice, "zancara"), ["Zafra de Záncara (Cuenca)"]);
        assert_eq!(
            nombres(&indice, "arraya"),
            ["Zafarraya (Granada)", "Ventas de Zafarraya (Granada)"]
        );
    }

    #[test]
    fn da_los_codigos_sin_repetir_y_respeta_el_maximo() {
        let indice = IndiceDeLocalidades::leer(DATOS);
        let almeria = indice.sugerir("almeria", 1);
        assert_eq!(almeria.len(), 1);
        assert_eq!(almeria[0].nombre, "Almería");
        assert_eq!(almeria[0].codigos_postales, ["04001", "04002"]);
        assert!(almeria[0].es_municipio);
        let palmer = indice.sugerir("palmer", 5);
        assert_eq!(palmer[0].codigos_postales, ["04002"]);
        assert!(!palmer[0].es_municipio);
    }

    #[test]
    fn con_menos_de_dos_letras_o_sin_parecido_no_sugiere() {
        let indice = IndiceDeLocalidades::leer(DATOS);
        assert!(indice.sugerir("z", 10).is_empty());
        assert!(indice.sugerir("  ", 10).is_empty());
        assert!(indice.sugerir("mérida", 10).is_empty());
        assert_eq!(indice.len(), 9);
    }
}
