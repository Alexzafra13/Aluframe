use super::Rol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accion {
    VerPanelDelPropietario,
    GestionarUsuarios,
    GestionarCatalogo,
    GestionarClientes,
    /// Exportar e importar todos los datos (ADR-0016).
    GestionarDatos,
    /// Asignar, modificar medidas, entregar o anular un trabajo; ver precios del trabajo.
    GestionarTrabajos,
    /// Facturas, cobros, señales y albaranes.
    GestionarFacturas,
    VerMiDia,
    /// Marcar los pasos de las ventanas que tiene asignadas (el propietario, de cualquiera).
    MarcarPasos,
}

pub fn puede_realizar(rol: Rol, accion: Accion) -> bool {
    match accion {
        Accion::VerPanelDelPropietario
        | Accion::GestionarUsuarios
        | Accion::GestionarCatalogo
        | Accion::GestionarClientes
        | Accion::GestionarDatos
        | Accion::GestionarTrabajos
        | Accion::GestionarFacturas => rol.es_propietario(),
        Accion::VerMiDia | Accion::MarcarPasos => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_propietario_puede_todo() {
        for accion in [
            Accion::VerPanelDelPropietario,
            Accion::GestionarUsuarios,
            Accion::GestionarCatalogo,
            Accion::GestionarClientes,
            Accion::GestionarDatos,
            Accion::GestionarTrabajos,
            Accion::GestionarFacturas,
            Accion::VerMiDia,
            Accion::MarcarPasos,
        ] {
            assert!(puede_realizar(Rol::Propietario, accion));
        }
    }

    #[test]
    fn el_operario_solo_ve_su_dia() {
        assert!(puede_realizar(Rol::Operario, Accion::VerMiDia));
        assert!(!puede_realizar(
            Rol::Operario,
            Accion::VerPanelDelPropietario
        ));
        assert!(!puede_realizar(Rol::Operario, Accion::GestionarUsuarios));
        assert!(!puede_realizar(Rol::Operario, Accion::GestionarCatalogo));
        assert!(!puede_realizar(Rol::Operario, Accion::GestionarClientes));
        assert!(!puede_realizar(Rol::Operario, Accion::GestionarDatos));
        assert!(!puede_realizar(Rol::Operario, Accion::GestionarTrabajos));
        assert!(!puede_realizar(Rol::Operario, Accion::GestionarFacturas));
        assert!(puede_realizar(Rol::Operario, Accion::MarcarPasos));
    }
}
