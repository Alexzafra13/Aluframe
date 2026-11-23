use super::*;

const TODAS: [Accion; 16] = [
    Accion::VerPanelDelPropietario,
    Accion::GestionarUsuarios,
    Accion::GestionarCatalogo,
    Accion::VerClientes,
    Accion::EditarClientes,
    Accion::GestionarClientes,
    Accion::VerPresupuestos,
    Accion::GestionarPresupuestos,
    Accion::VerCostes,
    Accion::GestionarDatos,
    Accion::VerTodosLosTrabajos,
    Accion::GestionarTrabajos,
    Accion::VerFacturas,
    Accion::GestionarFacturas,
    Accion::VerMiDia,
    Accion::MarcarPasos,
];

/// Lo que nunca puede hacer un operario, tenga los permisos que tenga.
const SOLO_DEL_DUENO: [Accion; 9] = [
    Accion::VerPanelDelPropietario,
    Accion::GestionarUsuarios,
    Accion::GestionarCatalogo,
    Accion::GestionarClientes,
    Accion::GestionarPresupuestos,
    Accion::VerCostes,
    Accion::GestionarDatos,
    Accion::GestionarTrabajos,
    Accion::GestionarFacturas,
];

#[test]
fn el_dueno_puede_todo_sin_permisos_extra() {
    for accion in TODAS {
        assert!(
            puede_realizar(Rol::Propietario, &Permisos::default(), accion),
            "{accion:?}"
        );
    }
}

#[test]
fn el_operario_sin_permisos_ve_su_dia_y_los_clientes() {
    let nada = Permisos::default();
    let permitidas: Vec<Accion> = TODAS
        .into_iter()
        .filter(|a| puede_realizar(Rol::Operario, &nada, *a))
        .collect();
    assert_eq!(
        permitidas,
        [Accion::VerClientes, Accion::VerMiDia, Accion::MarcarPasos]
    );
}

#[test]
fn cada_permiso_abre_solo_lo_suyo() {
    let casos = [
        (Permiso::EditarClientes, Accion::EditarClientes),
        (Permiso::VerPresupuestos, Accion::VerPresupuestos),
        (Permiso::VerFacturas, Accion::VerFacturas),
        (Permiso::VerTodosLosTrabajos, Accion::VerTodosLosTrabajos),
    ];
    for (permiso, accion) in casos {
        let uno = Permisos::con([permiso]);
        assert!(puede_realizar(Rol::Operario, &uno, accion));
        for (otro, otra_accion) in casos {
            if otro != permiso {
                assert!(!puede_realizar(Rol::Operario, &uno, otra_accion));
            }
        }
    }
}

#[test]
fn con_todos_los_permisos_el_operario_nunca_ve_costes_ni_gestiona() {
    let todos = Permisos::con(Permiso::TODOS);
    for accion in SOLO_DEL_DUENO {
        assert!(!puede_realizar(Rol::Operario, &todos, accion), "{accion:?}");
    }
}

#[test]
fn cada_permiso_se_guarda_y_se_lee_por_su_nombre() {
    for permiso in Permiso::TODOS {
        assert_eq!(Permiso::desde_texto(permiso.como_texto()), Ok(permiso));
    }
    assert_eq!(
        Permiso::desde_texto("ver_costes"),
        Err(ErrorAcceso::PermisoDesconocido("ver_costes".into()))
    );
}
