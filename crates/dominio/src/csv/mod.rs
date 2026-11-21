//! Lector de CSV propio (ADR-0012): comillas dobles, separador `;`, `,` o tabulador
//! detectado en la cabecera, BOM y finales de línea de Windows.

mod lector;
mod tabla;

#[cfg(test)]
mod pruebas;

pub use lector::{ErrorCsv, TAMANO_MAXIMO, leer_csv};
pub use tabla::{FilaCsv, TablaCsv};
