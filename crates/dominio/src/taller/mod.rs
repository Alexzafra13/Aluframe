//! Taller (H4): los pasos de cada ventana, el estado del trabajo y su columna en el tablero, y el
//! plan de corte de todo el trabajo. Sin entrada/salida: el servidor guarda y muestra.

mod pasos;
mod plan;
mod tablero;

pub use pasos::{AccionDePaso, EstadoDePaso, PASOS_POR_DEFECTO, aplicar_accion, leer_pasos};
pub use plan::{
    BarrasDePerfil, ClaveDePerfil, PiezaDeTrabajo, RetalesPorPerfil, planificar_trabajo,
};
pub use tablero::{ColumnaDelTablero, EstadoDeTrabajo, calcular_estado, columna_del_tablero};
