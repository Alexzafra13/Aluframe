mod error;
mod mejor_ajuste;
mod plan;
mod problema;

#[cfg(test)]
mod pruebas;

pub use error::ErrorCorte;
pub use mejor_ajuste::{
    RetalDisponible, calcular_minimo_de_barras, optimizar_con_retales, optimizar_cortes,
};
pub use plan::{BarraCortada, PlanDeCorte};
pub use problema::{ParametrosDeCorte, PiezaACortar};
