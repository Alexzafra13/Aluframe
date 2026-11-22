//! Lo que el taller necesita de cada ventana, copiado al aceptar (DATOS, apartado 4): cambiar
//! después la serie, la tarifa o el catálogo no altera el trabajo.

use std::collections::BTreeMap;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::aplicacion::resumen_de_linea::ResumenDeVentana;
use crate::aplicacion::series::reconstruir;
use crate::bd::consultas::lineas_presupuesto::FilaLinea;
use crate::bd::consultas::{acabados, series};
use crate::errores::ErrorApp;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VentanaCongelada {
    pub serie_version_id: i64,
    pub tipologia: String,
    pub ancho_dmm: i64,
    pub alto_dmm: i64,
    pub opciones: BTreeMap<String, String>,
    pub acabado_id: Option<i64>,
    /// «Blanco» si no se eligió color: así sale en la hoja y agrupa el corte.
    pub acabado: String,
    pub vidrio_articulo_id: Option<i64>,
    pub descripcion: String,
    pub resumen: ResumenDeVentana,
    pub longitud_barra_dmm: Option<i64>,
}

impl VentanaCongelada {
    pub fn leer(json: &str) -> VentanaCongelada {
        serde_json::from_str(json).unwrap_or_default()
    }
}

pub fn congelar_linea(con: &Connection, linea: &FilaLinea) -> Result<VentanaCongelada, ErrorApp> {
    let serie_version_id = linea.serie_version_id.ok_or(ErrorApp::NoEncontrado)?;
    let version = series::buscar_version(con, serie_version_id)?.ok_or(ErrorApp::NoEncontrado)?;
    let serie = reconstruir(version)?.serie;
    let acabado = match linea.acabado_id {
        Some(id) => acabados::buscar(con, id)?.map(|a| a.nombre),
        None => None,
    };
    Ok(VentanaCongelada {
        serie_version_id,
        tipologia: linea.tipologia.clone().unwrap_or_default(),
        ancho_dmm: linea.ancho_dmm.unwrap_or_default(),
        alto_dmm: linea.alto_dmm.unwrap_or_default(),
        opciones: serde_json::from_str(&linea.opciones_json).unwrap_or_default(),
        acabado_id: linea.acabado_id,
        acabado: acabado.unwrap_or_else(|| "Blanco".into()),
        vidrio_articulo_id: linea.vidrio_articulo_id,
        descripcion: linea.descripcion.clone(),
        resumen: ResumenDeVentana::leer(linea.resultado_json.as_deref()),
        longitud_barra_dmm: serie.longitud_barra.valor().map(|l| l.decimas_de_mm()),
    })
}
