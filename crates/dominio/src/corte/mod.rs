mod error;
mod mejor_ajuste;
mod plan;
mod problema;

#[cfg(test)]
mod pruebas;

pub use error::ErrorCorte;
pub use mejor_ajuste::{calcular_minimo_de_barras, optimizar_cortes};
pub use plan::{BarraCortada, PlanDeCorte};
pub use problema::{ParametrosDeCorte, PiezaACortar};
