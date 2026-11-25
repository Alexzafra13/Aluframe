//! Taller (H4): los pasos de cada ventana, el estado del trabajo y su columna en el tablero, y el
//! plan de corte de todo el trabajo. Sin entrada/salida: el servidor guarda y muestra.

mod habituales;
mod material;
mod pasos;
mod plan;
mod tablero;

pub use habituales::{PiezaCortada, peso_por_antiguedad, valorar_lo_habitual};
pub use material::{EstadoDeMaterial, MedidaDeMaterial, estado_tras_cambiar_la_medida};
pub use pasos::{AccionDePaso, EstadoDePaso, PASOS_POR_DEFECTO, aplicar_accion, leer_pasos};
pub use plan::{
    BarrasDePerfil, ClaveDePerfil, PiezaDeTrabajo, RetalesPorPerfil, ValoracionesPorPerfil,
    planificar_trabajo,
};
pub use tablero::{ColumnaDelTablero, EstadoDeTrabajo, calcular_estado, columna_del_tablero};
