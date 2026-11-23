//! Calles sugeridas mientras se escribe la dirección (ADR-0023), del callejero del INE que viene
//! con el programa (`data/callejero/`). Se busca en la localidad ya escrita; si no está en el
//! callejero, por su código postal (pedanías); y sin ninguna de las dos, en toda la provincia.

use crate::codigos_postales::limpiar_codigo_postal;
use crate::localidades::parecido;
use crate::texto::normalizar_para_buscar;

const LETRAS_MINIMAS: usize = 2;

/// Lo que se suele escribir abreviado al principio de una dirección.
const ABREVIATURAS: [(&str, &str); 12] = [
    ("c/", "calle"),
    ("cl", "calle"),
    ("avda", "avenida"),
    ("avd", "avenida"),
    ("av", "avenida"),
    ("pza", "plaza"),
    ("pl", "plaza"),
    ("ctra", "carretera"),
    ("cmno", "camino"),
    ("trva", "travesia"),
    ("urb", "urbanizacion"),
    ("po", "paseo"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SugerenciaDeCalle<'a> {
    /// Tipo y nombre: «Avenida Duques de Feria».
    pub calle: String,
    pub localidad: &'a str,
    pub provincia: &'static str,
    pub codigos_postales: &'a [String],
}

/// Lo que ya está escrito en el formulario, que dice dónde buscar.
#[derive(Debug, Clone, Copy, Default)]
pub struct DondeBuscar<'a> {
    pub localidad: &'a str,
    pub codigo_postal: &'a str,
}

#[derive(Debug, Clone)]
struct Municipio {
    /// La primera es el municipio; después, sus otras localidades (pedanías, urbanizaciones).
    localidades: Vec<String>,
    claves: Vec<String>,
    provincia: &'static str,
}

#[derive(Debug, Clone)]
struct Calle {
    tipo: String,
    nombre: String,
    clave_nombre: String,
    clave_completa: String,
    codigos_postales: Vec<String>,
    municipio: usize,
    localidades: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Callejero {
    municipios: Vec<Municipio>,
    calles: Vec<Calle>,
}

enum Filtro {
    Localidades(Vec<(usize, usize)>),
    CodigoPostal(String),
    Todo,
}

impl Callejero {
    /// Lee el callejero de una provincia (formato en `scripts/generar_callejero.py`); los
    /// renglones mal formados se saltan.
    pub fn anadir(&mut self, texto: &str, provincia: &'static str) {
        for renglon in texto.lines().filter(|r| !r.starts_with('#')) {
            if let Some(nombres) = renglon.strip_prefix('=') {
                let localidades: Vec<String> = nombres.split('|').map(str::to_owned).collect();
                let claves = localidades
                    .iter()
                    .map(|l| normalizar_para_buscar(l))
                    .collect();
                self.municipios.push(Municipio {
                    localidades,
                    claves,
                    provincia,
                });
            } else if let Some(calle) = self.leer_calle(renglon) {
                self.calles.push(calle);
            }
        }
    }

    fn leer_calle(&self, renglon: &str) -> Option<Calle> {
        let municipio = self.municipios.len().checked_sub(1)?;
        let total = self.municipios[municipio].localidades.len();
        let mut campos = renglon.split('\t');
        let (tipo, nombre) = (campos.next()?, campos.next()?);
        let codigos_postales = campos
            .next()?
            .split('|')
            .filter_map(limpiar_codigo_postal)
            .collect();
        let localidades = campos
            .next()?
            .split('|')
            .filter_map(|i| i.parse().ok())
            .filter(|i| *i < total)
            .collect();
        Some(Calle {
            clave_nombre: normalizar_para_buscar(nombre),
            clave_completa: normalizar_para_buscar(&format!("{tipo} {nombre}")),
            tipo: tipo.to_owned(),
            nombre: nombre.to_owned(),
            codigos_postales,
            municipio,
            localidades,
        })
    }

    /// Las `maximo` calles más parecidas a lo escrito, por el mismo orden que las localidades
    /// (exacto, empieza por, una palabra empieza por, contiene) y, a igualdad, las que tienen
    /// código postal y las más cortas.
    pub fn sugerir(
        &self,
        escrito: &str,
        donde: DondeBuscar,
        maximo: usize,
    ) -> Vec<SugerenciaDeCalle<'_>> {
        let buscado = expandir_abreviaturas(&normalizar_para_buscar(escrito));
        let Some(filtro) = self.filtro(donde) else {
            return Vec::new();
        };
        if buscado.chars().count() < LETRAS_MINIMAS {
            return Vec::new();
        }
        let mut encontradas: Vec<_> = self
            .calles
            .iter()
            .filter_map(|calle| {
                let localidad = self.localidad_si_cumple(calle, &filtro)?;
                let por_nombre = parecido(&calle.clave_nombre, &buscado);
                let por_completa = parecido(&calle.clave_completa, &buscado);
                let parecido = [por_nombre, por_completa].into_iter().flatten().min()?;
                Some((parecido, calle, localidad))
            })
            .collect();
        // Una calle sin código postal no tiene viviendas censadas: es menos probable que sea esa.
        encontradas.sort_by_key(|(p, c, l)| {
            let sin_codigo = c.codigos_postales.is_empty();
            (
                *p,
                sin_codigo,
                c.clave_completa.len(),
                &c.clave_completa,
                *l,
            )
        });
        encontradas
            .into_iter()
            .take(maximo)
            .map(|(_, calle, localidad)| SugerenciaDeCalle {
                calle: format!("{} {}", calle.tipo, calle.nombre),
                localidad,
                provincia: self.municipios[calle.municipio].provincia,
                codigos_postales: &calle.codigos_postales,
            })
            .collect()
    }

    /// `None` si la localidad escrita no está en el callejero y no hay código con que buscar:
    /// es de otra provincia y sugerir calles de esta sería equivocarse.
    fn filtro(&self, donde: DondeBuscar) -> Option<Filtro> {
        let localidad = normalizar_para_buscar(donde.localidad);
        let codigo = limpiar_codigo_postal(donde.codigo_postal);
        if localidad.is_empty() {
            return Some(codigo.map_or(Filtro::Todo, Filtro::CodigoPostal));
        }
        let coincidencias: Vec<(usize, usize)> = self
            .municipios
            .iter()
            .enumerate()
            .flat_map(|(m, municipio)| {
                let localidad = &localidad;
                municipio
                    .claves
                    .iter()
                    .enumerate()
                    .filter(move |(_, clave)| *clave == localidad)
                    .map(move |(i, _)| (m, i))
            })
            .collect();
        if coincidencias.is_empty() {
            codigo.map(Filtro::CodigoPostal)
        } else {
            Some(Filtro::Localidades(coincidencias))
        }
    }

    fn localidad_si_cumple(&self, calle: &Calle, filtro: &Filtro) -> Option<&str> {
        let localidades = &self.municipios[calle.municipio].localidades;
        let elegida = match filtro {
            Filtro::Localidades(lista) => lista
                .iter()
                .find(|(m, i)| *m == calle.municipio && calle.localidades.contains(i))
                .map(|(_, i)| *i)?,
            Filtro::CodigoPostal(cp) if !calle.codigos_postales.contains(cp) => return None,
            _ => calle.localidades.first().copied().unwrap_or(0),
        };
        localidades.get(elegida).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.calles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.calles.is_empty()
    }
}

/// «c/ sevilla» y «avda duques» se buscan como «calle sevilla» y «avenida duques».
fn expandir_abreviaturas(buscado: &str) -> String {
    let buscado = buscado.replacen("c/", "c/ ", 1);
    let (primera, resto) = buscado.split_once(' ').unwrap_or((&buscado, ""));
    let primera = primera.trim_end_matches('.');
    let completa = ABREVIATURAS
        .iter()
        .find(|(corta, _)| *corta == primera)
        .map_or(primera, |(_, larga)| larga);
    format!("{completa} {}", resto.trim()).trim().to_owned()
}

#[cfg(test)]
#[path = "callejero_pruebas.rs"]
mod pruebas;
