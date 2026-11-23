#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use comun::Contexto;
use dominio::acceso::{Permiso, Permisos, Rol};
use dominio::apuntes::{EntradaDeApunte, EstadoApunte, validar_apunte};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::Actor;
use servidor::aplicacion::apuntes::*;
use servidor::aplicacion::clientes::{borrar_cliente, crear_cliente};
use servidor::aplicacion::exportar_cliente::exportar_cliente;
use servidor::aplicacion::presupuestos::cargar_presupuesto;
use servidor::errores::ErrorApp;

fn peticion(texto: &str) -> PeticionDeApunte {
    let entrada = EntradaDeApunte {
        texto: texto.into(),
        medidas: "Cocina 120 × 100".into(),
        ..EntradaDeApunte::default()
    };
    PeticionDeApunte {
        cliente: None,
        direccion: None,
        para: None,
        datos: validar_apunte(&entrada).unwrap(),
    }
}

/// Un taller con el dueño, un cliente y un operario (id 2) sin permisos extra.
async fn taller() -> (Contexto, Actor, Actor) {
    let contexto = Contexto::nuevo().await;
    let (_, dueno) = contexto.con_propietario().await;
    let cliente = validar_cliente(&EntradaDeCliente {
        nombre: "Ana".into(),
        ..EntradaDeCliente::default()
    })
    .unwrap();
    crear_cliente(&contexto.estado, &dueno, cliente, vec![])
        .await
        .unwrap();
    let id = contexto.con_operario(&dueno).await;
    let operario = Actor {
        id,
        nombre: "Juan".into(),
        rol: Rol::Operario,
        permisos: Permisos::default(),
        ..dueno.clone()
    };
    (contexto, dueno, operario)
}

#[tokio::test]
async fn el_operario_apunta_para_el_dueno_y_solo_ve_lo_suyo() {
    let (contexto, dueno, operario) = taller().await;
    let estado = &contexto.estado;
    let suyo = PeticionDeApunte {
        cliente: Some(1),
        ..peticion("Quiere presupuesto de mosquiteras")
    };
    let id = crear_apunte(estado, &operario, suyo).await.unwrap();
    crear_apunte(estado, &dueno, peticion("Llamar al proveedor"))
        .await
        .unwrap();
    let del_operario = listar_apuntes(estado, &operario, true, None).await.unwrap();
    assert_eq!(del_operario.len(), 1);
    assert_eq!(del_operario[0].id, id);
    assert_eq!(del_operario[0].cliente.as_deref(), Some("Ana"));
    assert_eq!(del_operario[0].medidas.as_deref(), Some("Cocina 120 × 100"));
    let todos = listar_apuntes(estado, &dueno, true, None).await.unwrap();
    assert_eq!(todos.len(), 2);
}

#[tokio::test]
async fn el_operario_no_manda_tareas_ni_cierra_lo_que_no_es_suyo() {
    let (contexto, dueno, operario) = taller().await;
    let estado = &contexto.estado;
    let tarea = PeticionDeApunte {
        para: Some(dueno.id),
        ..peticion("Tarea")
    };
    let mandar = crear_apunte(estado, &operario, tarea).await;
    assert!(matches!(mandar, Err(ErrorApp::NoEncontrado)));
    let nota = crear_apunte(estado, &operario, peticion("Para el dueño"))
        .await
        .unwrap();
    let cerrar = cambiar_apunte(estado, &operario, (nota, 1), EstadoApunte::Hecho).await;
    assert!(matches!(cerrar, Err(ErrorApp::NoEncontrado)));
    let presupuestar = pasar_a_presupuesto(estado, &operario, (nota, 1)).await;
    assert!(matches!(presupuestar, Err(ErrorApp::NoEncontrado)));
}

#[tokio::test]
async fn el_dueno_manda_una_tarea_y_el_operario_la_marca_hecha() {
    let (contexto, dueno, operario) = taller().await;
    let estado = &contexto.estado;
    let tarea = PeticionDeApunte {
        para: Some(operario.id),
        ..peticion("Cambiar la cerradura del portal")
    };
    let id = crear_apunte(estado, &dueno, tarea).await.unwrap();
    let tareas = mis_tareas(estado, &operario).await.unwrap();
    assert_eq!(tareas.len(), 1);
    assert_eq!(tareas[0].para.as_deref(), Some("Juan"));
    cambiar_apunte(estado, &operario, (id, 1), EstadoApunte::Hecho)
        .await
        .unwrap();
    assert!(mis_tareas(estado, &operario).await.unwrap().is_empty());
    let otra_vez = cambiar_apunte(estado, &operario, (id, 1), EstadoApunte::Pendiente).await;
    assert!(
        matches!(otra_vez, Err(ErrorApp::Validacion(_))),
        "versión vieja"
    );
    cambiar_apunte(estado, &dueno, (id, 2), EstadoApunte::Pendiente)
        .await
        .unwrap();
}

#[tokio::test]
async fn hacer_presupuesto_crea_el_borrador_y_cierra_el_apunte_a_la_vez() {
    let (contexto, dueno, operario) = taller().await;
    let estado = &contexto.estado;
    let sin_cliente = crear_apunte(estado, &operario, peticion("Sin cliente"))
        .await
        .unwrap();
    let fallo = pasar_a_presupuesto(estado, &dueno, (sin_cliente, 1)).await;
    assert!(matches!(fallo, Err(ErrorApp::Validacion(_))));
    let con_cliente = PeticionDeApunte {
        cliente: Some(1),
        ..peticion("Ventanas del salón")
    };
    let id = crear_apunte(estado, &operario, con_cliente).await.unwrap();
    let presupuesto = pasar_a_presupuesto(estado, &dueno, (id, 1)).await.unwrap();
    let ficha = cargar_presupuesto(estado, &dueno, presupuesto)
        .await
        .unwrap();
    assert_eq!(ficha.cliente.id, 1);
    let cerrados = listar_apuntes(estado, &dueno, false, None).await.unwrap();
    assert_eq!(cerrados[0].estado, "presupuestado");
    assert_eq!(cerrados[0].presupuesto_id, Some(presupuesto));
    let dos_veces = pasar_a_presupuesto(estado, &dueno, (id, 2)).await;
    assert!(matches!(dos_veces, Err(ErrorApp::Validacion(_))));
}

#[tokio::test]
async fn los_apuntes_van_en_la_exportacion_y_se_borran_con_el_cliente() {
    let (contexto, dueno, operario) = taller().await;
    let estado = &contexto.estado;
    let con_cliente = PeticionDeApunte {
        cliente: Some(1),
        ..peticion("Persiana del dormitorio")
    };
    crear_apunte(estado, &operario, con_cliente).await.unwrap();
    let csv = exportar_cliente(estado, &dueno, 1).await.unwrap();
    assert!(csv.contains("apunte_1_texto;Persiana del dormitorio"));
    borrar_cliente(estado, &dueno, 1).await.unwrap();
    assert!(
        listar_apuntes(estado, &dueno, true, None)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn una_direccion_de_otro_cliente_o_sin_cliente_no_vale() {
    let (contexto, dueno, operario) = taller().await;
    let sin_cliente = PeticionDeApunte {
        direccion: Some(1),
        ..peticion("x")
    };
    let resultado = crear_apunte(&contexto.estado, &operario, sin_cliente).await;
    assert!(matches!(resultado, Err(ErrorApp::Validacion(_))));
    let ajena = PeticionDeApunte {
        cliente: Some(1),
        direccion: Some(99),
        ..peticion("x")
    };
    let resultado = crear_apunte(&contexto.estado, &dueno, ajena).await;
    assert!(matches!(resultado, Err(ErrorApp::Validacion(_))));
    let con_permiso = Actor {
        permisos: Permisos::con([Permiso::EditarClientes]),
        ..operario
    };
    let tarea = PeticionDeApunte {
        para: Some(2),
        ..peticion("x")
    };
    let mandar = crear_apunte(&contexto.estado, &con_permiso, tarea).await;
    assert!(
        matches!(mandar, Err(ErrorApp::NoEncontrado)),
        "ni con permisos manda tareas"
    );
}
