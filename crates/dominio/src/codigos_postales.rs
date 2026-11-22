//! Código postal español: la provincia sale de sus dos primeras cifras (regla de Correos) y las
//! localidades, del listado que viene con el programa (`data/codigos_postales.tsv`, ADR-0020).
//! Nunca se adivina: si a un código le corresponden varias localidades, se ofrecen para elegir.

use std::collections::BTreeMap;

use crate::texto::normalizar_para_buscar;

/// Por orden de código: 01 Álava … 52 Melilla.
const PROVINCIAS: [&str; 52] = [
    "Álava",
    "Albacete",
    "Alicante",
    "Almería",
    "Ávila",
    "Badajoz",
    "Baleares",
    "Barcelona",
    "Burgos",
    "Cáceres",
    "Cádiz",
    "Castellón",
    "Ciudad Real",
    "Córdoba",
    "A Coruña",
    "Cuenca",
    "Girona",
    "Granada",
    "Guadalajara",
    "Guipúzcoa",
    "Huelva",
    "Huesca",
    "Jaén",
    "León",
    "Lleida",
    "La Rioja",
    "Lugo",
    "Madrid",
    "Málaga",
    "Murcia",
    "Navarra",
    "Ourense",
    "Asturias",
    "Palencia",
    "Las Palmas",
    "Pontevedra",
    "Salamanca",
    "Santa Cruz de Tenerife",
    "Cantabria",
    "Segovia",
    "Sevilla",
    "Soria",
    "Tarragona",
    "Teruel",
    "Toledo",
    "Valencia",
    "Valladolid",
    "Vizcaya",
    "Zamora",
    "Zaragoza",
    "Ceuta",
    "Melilla",
];

/// Las 5 cifras de un código postal español, sin espacios; `None` si no lo es.
pub fn limpiar_codigo_postal(texto: &str) -> Option<String> {
    let cifras: String = texto.chars().filter(|c| !c.is_whitespace()).collect();
    let numero: u32 = cifras.get(..2)?.parse().ok()?;
    let valido = cifras.len() == 5 && cifras.chars().all(|c| c.is_ascii_digit());
    (valido && (1..=52).contains(&numero)).then_some(cifras)
}

pub fn provincia_de_codigo_postal(texto: &str) -> Option<&'static str> {
    let cp = limpiar_codigo_postal(texto)?;
    let numero: usize = cp.get(..2)?.parse().ok()?;
    PROVINCIAS.get(numero.checked_sub(1)?).copied()
}

#[derive(Debug, Clone, Default)]
pub struct IndiceDeCodigosPostales {
    localidades: BTreeMap<String, Vec<String>>,
    /// Por nombre sin tildes: los códigos donde es el municipio y aquellos donde es otro núcleo.
    codigos: BTreeMap<String, (Vec<String>, Vec<String>)>,
}

impl IndiceDeCodigosPostales {
    /// Lee `código<TAB>localidad|localidad…`; las líneas con `#` y las mal formadas se saltan.
    pub fn leer(texto: &str) -> IndiceDeCodigosPostales {
        let mut indice = IndiceDeCodigosPostales::default();
        for renglon in texto.lines().filter(|r| !r.starts_with('#')) {
            let Some((cp, nombres)) = renglon.split_once('\t') else {
                continue;
            };
            let Some(cp) = limpiar_codigo_postal(cp) else {
                continue;
            };
            let nombres: Vec<String> = nombres
                .split('|')
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map(str::to_owned)
                .collect();
            for (posicion, nombre) in nombres.iter().enumerate() {
                let entrada = indice.codigos.entry(normalizar_para_buscar(nombre));
                let (municipio, otros) = entrada.or_default();
                let lista = if posicion == 0 { municipio } else { otros };
                lista.push(cp.clone());
            }
            indice.localidades.insert(cp, nombres);
        }
        indice
    }

    /// Las localidades del código, la primera es el municipio.
    pub fn localidades_de(&self, cp: &str) -> &[String] {
        limpiar_codigo_postal(cp)
            .and_then(|cp| self.localidades.get(&cp))
            .map_or(&[], Vec::as_slice)
    }

    /// Los códigos postales de una localidad escrita con o sin tildes. Si es el nombre de un
    /// municipio, solo los suyos: «Zafra» es el de Badajoz y no la aldea de Ourense.
    pub fn codigos_de(&self, localidad: &str) -> &[String] {
        match self.codigos.get(&normalizar_para_buscar(localidad)) {
            Some((municipio, _)) if !municipio.is_empty() => municipio,
            Some((_, otros)) => otros,
            None => &[],
        }
    }

    pub fn len(&self) -> usize {
        self.localidades.len()
    }

    pub fn is_empty(&self) -> bool {
        self.localidades.is_empty()
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const DATOS: &str = "# comentario\n06300\tZafra\n04002\tAlmería|El Palmer\n04001\tAlmería\n32510\tCarballiño, O|Zafra\nmal\n99999\tNada\n";

    #[test]
    fn la_provincia_sale_de_las_dos_primeras_cifras() {
        assert_eq!(provincia_de_codigo_postal("06300"), Some("Badajoz"));
        assert_eq!(provincia_de_codigo_postal(" 28 001"), Some("Madrid"));
        assert_eq!(provincia_de_codigo_postal("01001"), Some("Álava"));
        assert_eq!(provincia_de_codigo_postal("52001"), Some("Melilla"));
        assert_eq!(provincia_de_codigo_postal("53001"), None);
        assert_eq!(provincia_de_codigo_postal("00001"), None);
        assert_eq!(provincia_de_codigo_postal("0630"), None);
        assert_eq!(provincia_de_codigo_postal("06A00"), None);
    }

    #[test]
    fn da_las_localidades_de_un_codigo_y_los_codigos_de_una_localidad() {
        let indice = IndiceDeCodigosPostales::leer(DATOS);
        assert_eq!(indice.len(), 4);
        assert_eq!(indice.localidades_de("06300"), ["Zafra"]);
        assert_eq!(indice.localidades_de("04002"), ["Almería", "El Palmer"]);
        assert!(indice.localidades_de("10000").is_empty());
        assert_eq!(indice.codigos_de("ZAFRA"), ["06300"]);
        assert_eq!(indice.codigos_de("almeria"), ["04002", "04001"]);
        assert!(indice.codigos_de("Mérida").is_empty());
        assert_eq!(indice.codigos_de("el palmer"), ["04002"]);
    }
}
