use std::collections::BTreeSet;

use super::{ErrorAcceso, Rol};

/// Lo que el dueño puede permitir a cada operario, además de lo básico (ADR-0024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permiso {
    /// Dar de alta clientes y corregir sus datos y direcciones.
    EditarClientes,
    /// Ver los presupuestos con su precio de venta (nunca costes ni márgenes).
    VerPresupuestos,
    /// Ver facturas, cobros y albaranes (sin emitir ni cobrar).
    VerFacturas,
    /// Ver y marcar los trabajos de todo el taller, no solo los suyos.
    VerTodosLosTrabajos,
}

impl Permiso {
    pub const TODOS: [Permiso; 4] = [
        Permiso::EditarClientes,
        Permiso::VerPresupuestos,
        Permiso::VerFacturas,
        Permiso::VerTodosLosTrabajos,
    ];

    pub fn como_texto(self) -> &'static str {
        match self {
            Permiso::EditarClientes => "editar_clientes",
            Permiso::VerPresupuestos => "ver_presupuestos",
            Permiso::VerFacturas => "ver_facturas",
            Permiso::VerTodosLosTrabajos => "ver_todos_los_trabajos",
        }
    }

    pub fn desde_texto(texto: &str) -> Result<Permiso, ErrorAcceso> {
        Permiso::TODOS
            .into_iter()
            .find(|p| p.como_texto() == texto)
            .ok_or_else(|| ErrorAcceso::PermisoDesconocido(texto.to_owned()))
    }
}

/// Los permisos extra de un operario. Al dueño no le hacen falta: lo puede todo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Permisos(BTreeSet<Permiso>);

impl Permisos {
    pub fn con(permisos: impl IntoIterator<Item = Permiso>) -> Permisos {
        Permisos(permisos.into_iter().collect())
    }

    pub fn tiene(&self, permiso: Permiso) -> bool {
        self.0.contains(&permiso)
    }

    pub fn iter(&self) -> impl Iterator<Item = Permiso> + '_ {
        self.0.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accion {
    VerPanelDelPropietario,
    GestionarUsuarios,
    GestionarCatalogo,
    /// Buscar clientes y ver sus datos de contacto y direcciones.
    VerClientes,
    /// Dar de alta clientes y corregir sus datos.
    EditarClientes,
    /// Archivar, borrar, exportar e importar clientes (protección de datos).
    GestionarClientes,
    VerPresupuestos,
    /// Crear y cambiar presupuestos.
    GestionarPresupuestos,
    /// Costes, márgenes y tarifas de compra: solo el dueño, sin excepción.
    VerCostes,
    /// Exportar e importar todos los datos (ADR-0016).
    GestionarDatos,
    /// Ver y marcar los trabajos de todo el taller.
    VerTodosLosTrabajos,
    /// Asignar, modificar medidas, entregar o anular un trabajo; ver precios del trabajo.
    GestionarTrabajos,
    VerFacturas,
    /// Emitir facturas, apuntar cobros y señales, y albaranes.
    GestionarFacturas,
    VerMiDia,
    /// Escribir apuntes y ver los suyos y las tareas que le mandan (ADR-0025).
    TomarApuntes,
    /// Ver todos los apuntes, mandar tareas a un operario y pasarlos a presupuesto.
    GestionarApuntes,
    /// Marcar los pasos de las ventanas que tiene asignadas (con permiso, de cualquiera).
    MarcarPasos,
}

pub fn puede_realizar(rol: Rol, permisos: &Permisos, accion: Accion) -> bool {
    if rol.es_propietario() {
        return true;
    }
    match accion {
        Accion::VerMiDia | Accion::MarcarPasos | Accion::VerClientes | Accion::TomarApuntes => true,
        Accion::EditarClientes => permisos.tiene(Permiso::EditarClientes),
        Accion::VerPresupuestos => permisos.tiene(Permiso::VerPresupuestos),
        Accion::VerFacturas => permisos.tiene(Permiso::VerFacturas),
        Accion::VerTodosLosTrabajos => permisos.tiene(Permiso::VerTodosLosTrabajos),
        Accion::VerPanelDelPropietario
        | Accion::GestionarUsuarios
        | Accion::GestionarCatalogo
        | Accion::GestionarClientes
        | Accion::GestionarPresupuestos
        | Accion::VerCostes
        | Accion::GestionarDatos
        | Accion::GestionarTrabajos
        | Accion::GestionarFacturas
        | Accion::GestionarApuntes => false,
    }
}

#[cfg(test)]
#[path = "permisos_pruebas.rs"]
mod pruebas;
