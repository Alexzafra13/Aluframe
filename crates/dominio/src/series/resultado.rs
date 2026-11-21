use super::{AplicaA, UnidadCantidad};
use crate::corte::PiezaACortar;
use crate::unidades::{Decimal, Longitud};

pub const MAXIMO_POR_PIEZA: u32 = 999;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiezaCalculada {
    pub numero: u32,
    pub perfil: String,
    pub cantidad: u32,
    pub longitud: Longitud,
    pub corte: Option<u16>,
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccesorioCalculado {
    pub articulo: String,
    pub cantidad: Decimal,
    pub aplica_a: AplicaA,
    pub unidad_cantidad: UnidadCantidad,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VidrioCalculado {
    pub tipo: String,
    pub cantidad: u32,
    pub ancho: Longitud,
    pub alto: Longitud,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoVentana {
    pub serie: String,
    pub tipologia: String,
    pub ancho: Decimal,
    pub alto: Decimal,
    pub piezas: Vec<PiezaCalculada>,
    pub accesorios: Vec<AccesorioCalculado>,
    pub vidrio: Option<Vec<VidrioCalculado>>,
    pub pendientes: Vec<String>,
    pub fuera_de_limites: Vec<String>,
}

impl ResultadoVentana {
    /// Incompleta: se calcula y se dibuja, pero no admite precio final ni aceptar el presupuesto.
    pub fn es_incompleta(&self) -> bool {
        !self.pendientes.is_empty()
    }

    /// Lista de compra: suma los accesorios con el mismo código, en el orden en que aparecen.
    pub fn accesorios_sumados(&self) -> Vec<(String, Decimal)> {
        let mut sumados: Vec<(String, Decimal)> = Vec::new();
        for accesorio in &self.accesorios {
            match sumados
                .iter_mut()
                .find(|(codigo, _)| *codigo == accesorio.articulo)
            {
                Some((_, total)) => *total = total.sumar(accesorio.cantidad).unwrap_or(*total),
                None => sumados.push((accesorio.articulo.clone(), accesorio.cantidad)),
            }
        }
        sumados
    }

    /// Piezas para el optimizador, agrupadas por perfil; el identificador es n.º de pieza × 1000 + copia.
    pub fn piezas_a_cortar(&self) -> Vec<(String, Vec<PiezaACortar>)> {
        let mut grupos: Vec<(String, Vec<PiezaACortar>)> = Vec::new();
        for pieza in &self.piezas {
            let copias = (1..=pieza.cantidad).map(|copia| PiezaACortar {
                id: pieza.numero * 1000 + copia,
                longitud: pieza.longitud,
            });
            match grupos
                .iter_mut()
                .find(|(perfil, _)| *perfil == pieza.perfil)
            {
                Some((_, lista)) => lista.extend(copias),
                None => grupos.push((pieza.perfil.clone(), copias.collect())),
            }
        }
        grupos
    }
}
