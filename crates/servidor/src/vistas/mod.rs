mod menu;
mod paginas;

pub use menu::{GrupoMenu, MENU};
pub use paginas::*;

use askama::Template;
use axum::response::Html;

use crate::errores::ErrorApp;

pub fn renderizar(plantilla: &impl Template) -> Result<Html<String>, ErrorApp> {
    Ok(Html(plantilla.render()?))
}
