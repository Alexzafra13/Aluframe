//! Series que vienen con el programa (`data/series/`, ADR-0014): se cargan solas al arrancar para
//! que una instalación nueva funcione sin importar nada a mano.

use rust_embed::RustEmbed;

use super::importar_serie::{Importacion, analizar, guardar};
use crate::autenticacion::calcular_sha256;
use crate::bd::consultas::series;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

#[derive(RustEmbed)]
#[folder = "../../data/series/"]
struct SeriesIncluidas;

/// Archivo incluido, por nombre y en orden alfabético (el orden de carga es estable).
pub fn listar_series_incluidas() -> Vec<(String, String)> {
    let mut nombres: Vec<String> = SeriesIncluidas::iter()
        .map(|n| n.into_owned())
        .filter(|n| n.ends_with(".yaml"))
        .collect();
    nombres.sort();
    nombres
        .into_iter()
        .filter_map(|nombre| {
            let archivo = SeriesIncluidas::get(&nombre)?;
            let texto = String::from_utf8(archivo.data.into_owned()).ok()?;
            Some((nombre, texto))
        })
        .collect()
}

/// Qué pasó con cada serie incluida al arrancar.
#[derive(Debug, PartialEq, Eq)]
pub enum CargaDeSerie {
    /// Se guardó como versión nueva (la vigente desde ahora).
    Cargada { codigo: String, numero: i64 },
    /// Esa serie ya tuvo alguna vez exactamente este archivo: no se toca.
    SinCambios { codigo: String },
    /// No pasó la validación: se salta y el programa arranca igual (las pruebas lo impiden antes).
    ConErrores { archivo: String, errores: String },
}

impl CargaDeSerie {
    /// Lo que se anota en el registro al arrancar (`true` si es un error); nada si no cambió.
    pub fn describir(&self) -> Option<(bool, String)> {
        match self {
            CargaDeSerie::Cargada { codigo, numero } => Some((
                false,
                format!("serie incluida {codigo} cargada como versión {numero}"),
            )),
            CargaDeSerie::ConErrores { archivo, errores } => Some((
                true,
                format!("la serie incluida {archivo} no se carga: {errores}"),
            )),
            CargaDeSerie::SinCambios { .. } => None,
        }
    }
}

/// Carga cada serie incluida cuyo archivo exacto (SHA-256) no esté aún en la base de datos.
/// Si el propietario importó después otra versión a mano, no se pisa mientras el archivo incluido
/// no cambie; cuando cambia, entra como versión nueva y las anteriores se conservan.
pub async fn cargar_series_incluidas(estado: &EstadoApp) -> Result<Vec<CargaDeSerie>, ErrorApp> {
    let mut resultado = Vec::new();
    for (nombre, texto) in listar_series_incluidas() {
        resultado.push(cargar_una(estado, &nombre, texto).await?);
    }
    Ok(resultado)
}

async fn cargar_una(
    estado: &EstadoApp,
    nombre: &str,
    texto: String,
) -> Result<CargaDeSerie, ErrorApp> {
    // Las incluidas son datos reales: nunca se admite una serie de demostración entre ellas.
    let (serie, informe) = analizar(&texto, false);
    let Some(serie) = serie.filter(|_| !informe.tiene_errores()) else {
        let errores: Vec<String> = informe.errores.iter().map(ToString::to_string).collect();
        return Ok(CargaDeSerie::ConErrores {
            archivo: nombre.to_owned(),
            errores: errores.join("; "),
        });
    };
    let avisos = i64::try_from(informe.avisos.len()).unwrap_or(i64::MAX);
    let ahora = formatear_utc(estado.ahora());
    estado
        .bd
        .escribir(move |tx| {
            let sha = calcular_sha256(&texto);
            if series::existe_version(tx, &serie.codigo, &sha)? {
                return Ok(CargaDeSerie::SinCambios {
                    codigo: serie.codigo.clone(),
                });
            }
            let importacion = Importacion {
                serie: &serie,
                texto: &texto,
                avisos,
                autor: None,
                ahora: &ahora,
            };
            let (codigo, numero) = guardar(tx, &importacion)?;
            Ok(CargaDeSerie::Cargada { codigo, numero })
        })
        .await
}
