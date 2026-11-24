//! Lo que se guarda del cálculo de una línea (`resultado_json`): basta para mostrarla e
//! imprimirla sin recalcular, y es la «fotografía» que se congela al aceptar.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::calculo_de_ventana::CalculoDeVentana;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PiezaResumida {
    pub numero: u32,
    pub perfil: String,
    pub cantidad: u32,
    pub longitud_dmm: i64,
    pub corte: Option<u16>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VidrioResumido {
    pub tipo: String,
    /// «Arriba», «Abajo» con parteluces. Los resúmenes anteriores no lo tienen.
    #[serde(default)]
    pub nombre: Option<String>,
    pub cantidad: u32,
    pub ancho_dmm: i64,
    pub alto_dmm: i64,
}

/// Chapa de un parteluces (ADR-0030); una medida que falta en la serie queda sin valor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChapaResumida {
    pub articulo: String,
    pub nombre: Option<String>,
    pub cantidad: u32,
    pub ancho_dmm: Option<i64>,
    pub alto_dmm: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResumenDeVentana {
    pub serie: String,
    pub tipologia: String,
    pub dibujo: String,
    pub piezas: Vec<PiezaResumida>,
    /// Piezas sin medida aún (el parteluces mientras falte su corte). No existían antes de la 0.20.1.
    #[serde(default)]
    pub piezas_pendientes: Vec<PiezaResumida>,
    pub vidrios: Vec<VidrioResumido>,
    /// Los resúmenes guardados antes de la 0.20.0 no tienen chapas: sin esto no se leerían.
    #[serde(default)]
    pub chapas: Vec<ChapaResumida>,
    pub accesorios: Vec<(String, String)>,
    pub barras: BTreeMap<String, u32>,
    pub perfiles_cent: i64,
    pub perfiles_por_barras_cent: Option<i64>,
    pub perfiles_por_metros_cent: Option<i64>,
    pub cristal_cent: i64,
    pub accesorios_cent: i64,
    pub venta_unitaria_cent: i64,
    pub fuera_de_limites: Vec<String>,
}

impl From<&CalculoDeVentana> for ResumenDeVentana {
    fn from(c: &CalculoDeVentana) -> Self {
        let r = &c.resultado;
        let m = &c.materiales;
        ResumenDeVentana {
            serie: c.serie.codigo.clone(),
            tipologia: c
                .serie
                .tipologia(&r.tipologia)
                .map(|t| t.nombre.clone())
                .unwrap_or_default(),
            dibujo: c.dibujo.clone(),
            piezas: r.piezas.iter().map(pieza).collect(),
            piezas_pendientes: r
                .piezas_pendientes
                .iter()
                .map(|p| PiezaResumida {
                    numero: p.numero,
                    perfil: p.perfil.clone(),
                    cantidad: p.cantidad.unwrap_or_default(),
                    longitud_dmm: 0,
                    corte: None,
                })
                .collect(),
            vidrios: r.vidrio.iter().flatten().map(vidrio).collect(),
            chapas: r.chapas.iter().map(chapa).collect(),
            accesorios: r
                .accesorios_sumados()
                .into_iter()
                .map(|(a, n)| (a, n.to_string().replace('.', ",")))
                .collect(),
            barras: c.barras.clone(),
            perfiles_cent: m.perfiles.centimos(),
            perfiles_por_barras_cent: m.perfiles_por_barras.map(|d| d.centimos()),
            perfiles_por_metros_cent: m.perfiles_por_metros.map(|d| d.centimos()),
            cristal_cent: m.cristal.centimos(),
            accesorios_cent: m.accesorios.centimos(),
            venta_unitaria_cent: c.precio.venta_unitaria.centimos(),
            fuera_de_limites: r.fuera_de_limites.clone(),
        }
    }
}

fn pieza(p: &dominio::series::PiezaCalculada) -> PiezaResumida {
    PiezaResumida {
        numero: p.numero,
        perfil: p.perfil.clone(),
        cantidad: p.cantidad,
        longitud_dmm: p.longitud.decimas_de_mm(),
        corte: p.corte,
    }
}

fn chapa(c: &dominio::series::ChapaCalculada) -> ChapaResumida {
    ChapaResumida {
        articulo: c.articulo.clone(),
        nombre: c.nombre.clone(),
        cantidad: c.cantidad,
        ancho_dmm: c.ancho.map(|l| l.decimas_de_mm()),
        alto_dmm: c.alto.map(|l| l.decimas_de_mm()),
    }
}

fn vidrio(v: &dominio::series::VidrioCalculado) -> VidrioResumido {
    VidrioResumido {
        tipo: v.tipo.clone(),
        nombre: v.nombre.clone(),
        cantidad: v.cantidad,
        ancho_dmm: v.ancho.decimas_de_mm(),
        alto_dmm: v.alto.decimas_de_mm(),
    }
}

impl ResumenDeVentana {
    pub fn leer(json: Option<&str>) -> ResumenDeVentana {
        json.and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un resumen guardado antes del parteluces (sin chapas ni nombre de cristal) se sigue
    /// leyendo entero: si no, la ventana de un trabajo antiguo saldría vacía.
    #[test]
    fn lee_un_resumen_anterior_al_parteluces() {
        let antiguo = r#"{"serie":"ALUPROM_21","tipologia":"Ventana corredera 2 hojas","dibujo":"<svg/>",
            "piezas":[{"numero":1,"perfil":"MARCO_LATERAL","cantidad":2,"longitud_dmm":12000,"corte":90}],
            "vidrios":[{"tipo":"CRISTAL","cantidad":2,"ancho_dmm":6520,"alto_dmm":10690}],
            "accesorios":[["KIT_CORREDERA","1"]],"barras":{},"perfiles_cent":0,
            "perfiles_por_barras_cent":null,"perfiles_por_metros_cent":null,"cristal_cent":0,
            "accesorios_cent":0,"venta_unitaria_cent":0,"fuera_de_limites":[]}"#;
        let resumen = ResumenDeVentana::leer(Some(antiguo));
        assert_eq!(resumen.serie, "ALUPROM_21");
        assert_eq!(resumen.vidrios.len(), 1);
        assert!(resumen.chapas.is_empty());
    }
}
