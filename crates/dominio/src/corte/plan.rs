use serde::{Deserialize, Serialize};

use super::PiezaACortar;
use crate::unidades::Longitud;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarraCortada {
    pub piezas: Vec<PiezaACortar>,
    pub sobrante: Longitud,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDeCorte {
    pub barras: Vec<BarraCortada>,
    pub retal_minimo: Longitud,
}

impl PlanDeCorte {
    pub fn numero_de_barras(&self) -> usize {
        self.barras.len()
    }

    pub fn retales(&self) -> Vec<Longitud> {
        self.barras
            .iter()
            .map(|b| b.sobrante)
            .filter(|s| *s >= self.retal_minimo && s.decimas_de_mm() > 0)
            .collect()
    }

    pub fn desperdicio(&self) -> Longitud {
        let total = self
            .barras
            .iter()
            .map(|b| b.sobrante)
            .filter(|s| *s < self.retal_minimo)
            .map(Longitud::decimas_de_mm)
            .sum();
        Longitud::desde_decimas_de_mm(total)
    }

    pub fn piezas(&self) -> impl Iterator<Item = &PiezaACortar> {
        self.barras.iter().flat_map(|b| b.piezas.iter())
    }
}
