//! Medidas habituales e historial (ADR-0026): de ellos sale cómo se valoran los sobrantes del
//! plan de corte de cada trabajo. Las medidas se guardan como ventanas (serie, tipología, ancho
//! y alto) en `configuracion`; sus piezas se calculan con la versión vigente de la serie.

use dominio::series::calcular_ventana;
use dominio::taller::{PiezaCortada, ValoracionesPorPerfil, valorar_lo_habitual};
use dominio::unidades::{Longitud, Unidad};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::series::reconstruir;
use crate::bd::consultas::{configuracion, series, ventanas_trabajo};
use crate::errores::ErrorApp;
use crate::fecha::leer_utc;

pub(crate) const CLAVE: &str = "taller.medidas_habituales";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MedidaHabitual {
    pub serie: String,
    pub tipologia: String,
    pub ancho_dmm: i64,
    pub alto_dmm: i64,
}

/// La que dio el propietario para empezar (2026-11-23): Aluprom 21, corredera de 2 hojas,
/// 120 × 120 cm. Se cambia en Ajustes > Taller.
fn por_defecto() -> Vec<MedidaHabitual> {
    vec![MedidaHabitual {
        serie: "ALUPROM_21".into(),
        tipologia: "corredera_2_hojas".into(),
        ancho_dmm: 12_000,
        alto_dmm: 12_000,
    }]
}

/// Sin nada guardado, las de partida; una lista vacía guardada es «ninguna».
pub(crate) fn leer_de_la_base(con: &Connection) -> Result<Vec<MedidaHabitual>, ErrorApp> {
    Ok(match configuracion::leer(con, CLAVE)? {
        Some(texto) => serde_json::from_str(&texto)
            .map_err(|e| ErrorApp::Interno(format!("medidas habituales: {e}")))?,
        None => por_defecto(),
    })
}

/// Las piezas de una medida habitual con la versión vigente de su serie, o por qué no se puede.
pub(crate) fn piezas_de(
    con: &Connection,
    medida: &MedidaHabitual,
) -> Result<Result<Vec<PiezaCortada>, String>, ErrorApp> {
    let Some(version) = series::buscar_ultima(con, &medida.serie)? else {
        return Ok(Err("la serie no está en el programa".into()));
    };
    let serie = reconstruir(version)?.serie;
    let en_cm = |dmm| Longitud::desde_decimas_de_mm(dmm).a_decimal(Unidad::Cm);
    let (Ok(ancho), Ok(alto)) = (en_cm(medida.ancho_dmm), en_cm(medida.alto_dmm)) else {
        return Ok(Err("la medida no es válida".into()));
    };
    Ok(calcular_ventana(&serie, &medida.tipologia, ancho, alto)
        .map(|resultado| {
            resultado
                .piezas
                .into_iter()
                .map(|p| PiezaCortada {
                    serie: serie.codigo.clone(),
                    perfil: p.perfil,
                    longitud: p.longitud,
                    cantidad: p.cantidad,
                    dias: 0,
                })
                .collect()
        })
        .map_err(|error| error.to_string()))
}

/// Cómo valorar los sobrantes de un trabajo: medidas habituales e historial de los trabajos
/// aceptados antes que él, con la antigüedad contada desde su aceptación. Así su plan no cambia
/// por lo que se acepte después.
pub fn valoraciones_del_trabajo(
    con: &Connection,
    (trabajo, aceptado_en): (i64, &str),
) -> Result<ValoracionesPorPerfil, ErrorApp> {
    let mut habituales = Vec::new();
    for medida in leer_de_la_base(con)? {
        habituales.extend(piezas_de(con, &medida)?.unwrap_or_default());
    }
    let referencia = leer_utc(aceptado_en);
    let historial: Vec<PiezaCortada> = ventanas_trabajo::piezas_anteriores(con, trabajo)?
        .into_iter()
        .map(|(cuando, serie, perfil, largo, cantidad)| PiezaCortada {
            serie,
            perfil,
            longitud: Longitud::desde_decimas_de_mm(largo),
            cantidad: u32::try_from(cantidad).unwrap_or(0),
            dias: match (referencia, leer_utc(&cuando)) {
                (Some(r), Some(c)) => (r - c).whole_days(),
                _ => 0,
            },
        })
        .collect();
    Ok(valorar_lo_habitual(&habituales, &historial))
}
