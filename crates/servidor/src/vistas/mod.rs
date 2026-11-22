mod catalogo;
mod clientes;
mod datos;
mod editor_linea;
mod escritorio;
mod menu;
mod paginas;
mod presupuestos;
mod series;
mod taller;
mod tarifas;
mod trabajos;

pub use catalogo::*;
pub use clientes::*;
pub use datos::*;
pub use editor_linea::*;
pub use escritorio::*;
pub use menu::{GrupoMenu, MENU};
pub use paginas::*;
pub use presupuestos::*;
pub use series::*;
pub use taller::*;
pub use tarifas::*;
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
