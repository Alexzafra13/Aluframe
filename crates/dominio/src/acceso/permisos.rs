use super::Rol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accion {
    VerPanelDelPropietario,
    GestionarUsuarios,
    VerMiDia,
}

pub fn puede_realizar(rol: Rol, accion: Accion) -> bool {
    match accion {
        Accion::VerPanelDelPropietario | Accion::GestionarUsuarios => rol.es_propietario(),
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
    }
}
