mod catalogo;
mod clientes;
mod datos;
mod editor_linea;
mod escritorio;
mod menu;
mod paginas;
mod presupuestos;
mod series;
mod tarifas;

pub use catalogo::*;
pub use clientes::*;
pub use datos::*;
pub use editor_linea::*;
pub use escritorio::*;
pub use menu::{GrupoMenu, MENU};
pub use paginas::*;
pub use presupuestos::*;
pub use series::*;
pub use tarifas::*;

use askama::Template;
use axum::response::Html;

use crate::errores::ErrorApp;

pub fn renderizar(plantilla: &impl Template) -> Result<Html<String>, ErrorApp> {
    Ok(Html(plantilla.render()?))
}
