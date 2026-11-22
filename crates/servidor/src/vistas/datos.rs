use askama::Template;

use super::{GrupoMenu, UsuarioEnPantalla};
use crate::bd::copia::ResumenDeDatos;
use crate::fecha::{formatear_para_pantalla, leer_utc};

pub struct ResumenEnPantalla {
    pub usuarios: i64,
    pub clientes: i64,
    pub presupuestos: i64,
    pub series: i64,
    pub ultimo_cambio: String,
}

impl From<&ResumenDeDatos> for ResumenEnPantalla {
    fn from(r: &ResumenDeDatos) -> Self {
        ResumenEnPantalla {
            usuarios: r.usuarios,
            clientes: r.clientes,
            presupuestos: r.presupuestos,
            series: r.series,
            ultimo_cambio: r
                .ultimo_cambio
                .as_deref()
                .and_then(leer_utc)
                .map_or_else(|| "sin cambios".to_owned(), formatear_para_pantalla),
        }
    }
}

#[derive(Template)]
#[template(path = "ajustes_datos.html")]
pub struct PaginaDatos {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub pestana: &'static str,
    pub resumen: ResumenEnPantalla,
    pub confirmacion: Option<String>,
}

#[derive(Template)]
#[template(path = "importar_datos.html")]
pub struct PaginaImportarDatos {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub pestana: &'static str,
    pub clave: String,
    pub actual: ResumenEnPantalla,
    pub archivo: ResumenEnPantalla,
}
