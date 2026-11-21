use serde::{Deserialize, Serialize};

use super::ErrorCorte;
use crate::unidades::Longitud;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PiezaACortar {
    pub id: u32,
    pub longitud: Longitud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParametrosDeCorte {
    pub longitud_barra: Longitud,
    pub grosor_sierra: Longitud,
    pub recorte_extremos: Longitud,
    pub retal_minimo: Longitud,
}

impl ParametrosDeCorte {
    pub fn longitud_util(&self) -> Result<Longitud, ErrorCorte> {
        self.longitud_barra
            .restar(self.recorte_extremos)
            .filter(|util| util.decimas_de_mm() > 0)
            .ok_or(ErrorCorte::SinLongitudUtil)
    }

    /// Regla conservadora de BACKEND.md (5.3): cada pieza gasta su longitud más un corte de sierra.
    pub fn consumo(&self, pieza: &PiezaACortar) -> Result<Longitud, ErrorCorte> {
        pieza
            .longitud
            .sumar(self.grosor_sierra)
            .ok_or(ErrorCorte::Desbordamiento)
    }
}
