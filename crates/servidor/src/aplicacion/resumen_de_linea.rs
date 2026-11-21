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
    pub cantidad: u32,
    pub ancho_dmm: i64,
    pub alto_dmm: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResumenDeVentana {
    pub serie: String,
    pub tipologia: String,
    pub dibujo: String,
    pub piezas: Vec<PiezaResumida>,
    pub vidrios: Vec<VidrioResumido>,
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
            vidrios: r.vidrio.iter().flatten().map(vidrio).collect(),
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

fn vidrio(v: &dominio::series::VidrioCalculado) -> VidrioResumido {
    VidrioResumido {
        tipo: v.tipo.clone(),
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
