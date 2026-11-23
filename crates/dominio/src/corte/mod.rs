//! Optimización de barras (SPEC, apartado 6): Best Fit Decreasing, rápido, y la búsqueda del
//! mejor aprovechamiento que parte de él (ADR-0026).

mod error;
mod exacto;
mod mejor_ajuste;
mod plan;
mod problema;
mod valoracion;

#[cfg(test)]
mod pruebas;
#[cfg(test)]
mod pruebas_exacto;

pub use error::ErrorCorte;
pub use exacto::PASOS_MAXIMOS;
pub use mejor_ajuste::{
    RetalDisponible, calcular_minimo_de_barras, optimizar_con_retales, optimizar_cortes,
};
pub use plan::{BarraCortada, PlanDeCorte};
pub use problema::{ParametrosDeCorte, PiezaACortar};
pub use valoracion::ValoracionDeSobrantes;

/// Cómo se reparte (la interfaz deja elegir, SPEC 6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MetodoDeCorte {
    /// Best Fit Decreasing tal cual.
    Rapido,
    /// Menos barras nuevas y, con las mismas, mejores sobrantes.
    #[default]
    MejorAprovechamiento,
}

impl MetodoDeCorte {
    pub fn como_texto(self) -> &'static str {
        match self {
            MetodoDeCorte::Rapido => "rapido",
            MetodoDeCorte::MejorAprovechamiento => "aprovechamiento",
        }
    }

    pub fn desde_texto(texto: &str) -> Option<MetodoDeCorte> {
        match texto {
            "rapido" => Some(MetodoDeCorte::Rapido),
            "aprovechamiento" => Some(MetodoDeCorte::MejorAprovechamiento),
            _ => None,
        }
    }
}

/// Reparte las piezas entre los retales del stock y barras nuevas con el método elegido.
pub fn planificar_cortes(
    (piezas, retales): (&[PiezaACortar], &[RetalDisponible]),
    parametros: &ParametrosDeCorte,
    (metodo, valoracion): (MetodoDeCorte, &ValoracionDeSobrantes),
) -> Result<PlanDeCorte, ErrorCorte> {
    let rapido = optimizar_con_retales(piezas, retales, parametros)?;
    Ok(match metodo {
        MetodoDeCorte::Rapido => rapido,
        MetodoDeCorte::MejorAprovechamiento => {
            exacto::mejorar(rapido, (piezas, retales), parametros, valoracion)
        }
    })
}
