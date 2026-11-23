mod apuntes;
mod avisos;
mod catalogo;
mod clientes;
mod copias;
mod datos;
mod editor_linea;
mod escritorio;
mod factura_impresa;
mod facturas;
mod menu;
mod orden;
mod paginas;
mod presupuestos;
mod series;
mod stock;
mod taller;
mod tarifas;
mod terminar;
mod trabajos;

pub use apuntes::*;
pub use avisos::*;
pub use catalogo::*;
pub use clientes::*;
pub use copias::*;
pub use datos::*;
pub use editor_linea::*;
pub use escritorio::*;
pub use factura_impresa::*;
pub use facturas::*;
pub use menu::{GrupoMenu, MENU};
pub use orden::*;
pub use paginas::*;
pub use presupuestos::*;
pub use series::*;
pub use stock::*;
pub use taller::*;
pub use tarifas::*;
pub use terminar::*;
pub use trabajos::*;

use askama::Template;
use axum::response::Html;

use crate::errores::ErrorApp;

/// Va en las direcciones de estilos y scripts: cada versión nueva obliga al navegador a
/// pedirlos otra vez en lugar de usar los guardados (que duran una hora).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn renderizar(plantilla: &impl Template) -> Result<Html<String>, ErrorApp> {
    Ok(Html(plantilla.render()?))
}
