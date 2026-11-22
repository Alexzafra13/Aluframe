use super::Rol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accion {
    VerPanelDelPropietario,
    GestionarUsuarios,
    GestionarCatalogo,
    GestionarClientes,
    /// Exportar e importar todos los datos (ADR-0016).
    GestionarDatos,
    VerMiDia,
}

pub fn puede_realizar(rol: Rol, accion: Accion) -> bool {
    match accion {
        Accion::VerPanelDelPropietario
        | Accion::GestionarUsuarios
        | Accion::GestionarCatalogo
        | Accion::GestionarClientes
        | Accion::GestionarDatos => rol.es_propietario(),
        Accion::VerMiDia => true,
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
            Accion::VerMiDia,
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
    }
}
