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
mod ficha_de_ventana;
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
pub use ficha_de_ventana::*;
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

/// Marca de cada formulario pintado: si llega dos veces, se atiende una (ADR-0036). Sin fuente
/// aleatoria va vacía y el formulario se atiende como siempre.
pub fn nuevo_envio() -> String {
    crate::autenticacion::generar_token().unwrap_or_default()
}

pub fn renderizar(plantilla: &impl Template) -> Result<Html<String>, ErrorApp> {
    Ok(Html(plantilla.render()?))
}
