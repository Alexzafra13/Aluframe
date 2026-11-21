mod clientes;
mod menu;
mod paginas;
mod series;

pub use clientes::*;
pub use menu::{GrupoMenu, MENU};
pub use paginas::*;
pub use series::*;

use askama::Template;
use axum::response::Html;

use crate::errores::ErrorApp;

pub fn renderizar(plantilla: &impl Template) -> Result<Html<String>, ErrorApp> {
    Ok(Html(plantilla.render()?))
}
