use askama::Template;
use dominio::acceso::{Accion, Permiso, Rol};

use super::GrupoMenu;
use crate::aplicacion::Actor;
use crate::aplicacion::usuarios::UsuarioVisible;

/// Quién mira la página y qué se le enseña. Lo que puede hacer lo decide `autorizar` en el
/// servidor: esto solo evita enseñar enlaces y botones que llevarían a «no encontrado».
pub struct UsuarioEnPantalla {
    pub nombre: String,
    pub token_csrf: String,
    pub es_propietario: bool,
    pub puede_editar_clientes: bool,
    pub ve_presupuestos: bool,
    pub ve_facturas: bool,
}

impl From<&Actor> for UsuarioEnPantalla {
    fn from(actor: &Actor) -> Self {
        UsuarioEnPantalla {
            nombre: actor.nombre.clone(),
            token_csrf: actor.token_csrf.clone(),
            es_propietario: actor.rol.es_propietario(),
            puede_editar_clientes: actor.puede(Accion::EditarClientes),
            ve_presupuestos: actor.puede(Accion::VerPresupuestos),
            ve_facturas: actor.puede(Accion::VerFacturas),
        }
    }
}

impl UsuarioEnPantalla {
    /// Las entradas del menú de cada uno (ADR-0024). «Mi día» es la pantalla del operario.
    pub fn ve_seccion(&self, seccion: &str) -> bool {
        match seccion {
            "mi-dia" => !self.es_propietario,
            "clientes" => true,
            "presupuestos" => self.ve_presupuestos,
            "facturas" => self.ve_facturas,
            _ => self.es_propietario,
        }
    }
}

/// Una casilla de permiso en Ajustes > Usuarios: nombre del campo, texto y si está marcada.
pub struct CasillaDePermiso {
    pub campo: &'static str,
    pub texto: &'static str,
    pub marcada: bool,
}

/// Lo que ve el dueño al marcar cada permiso (ADR-0024).
pub fn texto_de_permiso(permiso: Permiso) -> &'static str {
    match permiso {
        Permiso::EditarClientes => "Dar de alta clientes y corregir sus datos",
        Permiso::VerPresupuestos => "Ver presupuestos, con su precio de venta",
        Permiso::VerFacturas => "Ver facturas y cobros",
        Permiso::VerTodosLosTrabajos => "Ver y marcar los trabajos de todo el taller",
    }
}

pub struct OpcionUsuario {
    pub permisos: Vec<CasillaDePermiso>,
    pub id: i64,
    pub nombre: String,
    pub inicial: String,
    pub rol: &'static str,
    pub activo: bool,
    pub es_operario: bool,
}

impl OpcionUsuario {
    pub fn permisos_marcados(&self) -> usize {
        self.permisos.iter().filter(|c| c.marcada).count()
    }
}

impl From<UsuarioVisible> for OpcionUsuario {
    fn from(usuario: UsuarioVisible) -> Self {
        OpcionUsuario {
            permisos: Permiso::TODOS
                .into_iter()
                .map(|p| CasillaDePermiso {
                    campo: p.como_texto(),
                    texto: texto_de_permiso(p),
                    marcada: usuario.permisos.tiene(p),
                })
                .collect(),
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
