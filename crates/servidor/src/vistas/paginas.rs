use askama::Template;
use dominio::acceso::Rol;

use super::GrupoMenu;
use crate::aplicacion::Actor;
use crate::aplicacion::usuarios::UsuarioVisible;

pub struct UsuarioEnPantalla {
    pub nombre: String,
    pub token_csrf: String,
    pub es_propietario: bool,
}

impl From<&Actor> for UsuarioEnPantalla {
    fn from(actor: &Actor) -> Self {
        UsuarioEnPantalla {
            nombre: actor.nombre.clone(),
            token_csrf: actor.token_csrf.clone(),
            es_propietario: actor.rol.es_propietario(),
        }
    }
}

pub struct OpcionUsuario {
    pub id: i64,
    pub nombre: String,
    pub inicial: String,
    pub rol: &'static str,
    pub activo: bool,
    pub es_operario: bool,
}

impl From<UsuarioVisible> for OpcionUsuario {
    fn from(usuario: UsuarioVisible) -> Self {
        OpcionUsuario {
            id: usuario.id,
            inicial: usuario
                .nombre
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect())
                .unwrap_or_default(),
            nombre: usuario.nombre,
            rol: match usuario.rol {
                Rol::Propietario => "Propietario",
                Rol::Operario => "Operario",
            },
            activo: usuario.activo,
            es_operario: usuario.rol == Rol::Operario,
        }
    }
}

#[derive(Template)]
#[template(path = "usuarios.html")]
pub struct PaginaUsuarios {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub pestana: &'static str,
    pub usuarios: Vec<OpcionUsuario>,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

#[derive(Template)]
#[template(path = "entrar.html")]
pub struct PaginaEntrar {
    pub usuarios: Vec<OpcionUsuario>,
    pub elegido: Option<OpcionUsuario>,
    pub aviso: Option<String>,
}

#[derive(Template)]
#[template(path = "primer_arranque.html")]
pub struct PaginaPrimerArranque {
    pub nombre: String,
    pub aviso: Option<String>,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct PaginaError {
    pub titulo: &'static str,
    pub mensaje: String,
    pub codigo: Option<String>,
}

#[derive(Template)]
#[template(path = "ajustes_taller.html")]
pub struct PaginaTaller {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub pestana: &'static str,
    pub grosor_sierra_mm: String,
    pub recorte_extremos_mm: String,
    pub retal_minimo_cm: String,
    /// Un paso por renglón (decisión 5 del plan de H4).
    pub pasos: String,
    pub provisional: bool,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}
