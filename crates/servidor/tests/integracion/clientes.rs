use crate::comun;

use comun::Contexto;
use dominio::acceso::{Permiso, Permisos, Rol};
use dominio::clientes::{
    DatosDeCliente, EntradaDeCliente, EntradaDeDireccion, validar_cliente, validar_direccion,
};
use servidor::aplicacion::Actor;
use servidor::aplicacion::clientes::*;
use servidor::aplicacion::exportar_cliente::exportar_cliente;
use servidor::errores::ErrorApp;

fn datos(nombre: &str, nif: &str) -> DatosDeCliente {
    validar_cliente(&EntradaDeCliente {
        nombre: nombre.into(),
        nif: nif.into(),
        fiscal_localidad: "Zafra".into(),
        ..EntradaDeCliente::default()
    })
    .unwrap()
}

fn consulta(texto: &str) -> ConsultaDeClientes {
    ConsultaDeClientes {
        texto: texto.into(),
        archivados: false,
        pagina: 1,
    }
}

#[tokio::test]
async fn crea_un_cliente_sin_datos_y_lo_muestra_por_su_numero() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let id = crear_cliente(&contexto.estado, &actor, DatosDeCliente::default(), vec![])
        .await
        .unwrap();
    let ficha = cargar_cliente(&contexto.estado, &actor, id).await.unwrap();
    assert_eq!(
        ficha.cliente.nombre_visible(),
        format!("Cliente sin nombre (n.º {id})")
    );
    assert_eq!(ficha.cliente.tipo, "particular");
}

#[tokio::test]
async fn busca_sin_tildes_y_pagina() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    crear_cliente(&contexto.estado, &actor, datos("José Peña", ""), vec![])
        .await
        .unwrap();
    crear_cliente(
        &contexto.estado,
        &actor,
        datos("Ana Ruiz", "12345678Z"),
        vec![],
    )
    .await
    .unwrap();
    let lista = listar_clientes(&contexto.estado, &actor, consulta("PENA"))
        .await
        .unwrap();
    assert_eq!(lista.total, 1);
    assert_eq!(lista.clientes[0].nombre.as_deref(), Some("José Peña"));
    let por_nif = listar_clientes(&contexto.estado, &actor, consulta("12345678z"))
        .await
        .unwrap();
    assert_eq!(por_nif.total, 1);
    let comodin = listar_clientes(&contexto.estado, &actor, consulta("%"))
        .await
        .unwrap();
    assert_eq!(
        comodin.total, 0,
        "el % se busca como texto, no como comodín"
    );
    let todos = listar_clientes(&contexto.estado, &actor, consulta(""))
        .await
        .unwrap();
    assert_eq!((todos.total, todos.paginas), (2, 1));
}

#[tokio::test]
async fn no_duplica_un_nif() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    crear_cliente(&contexto.estado, &actor, datos("Ana", "12345678Z"), vec![])
        .await
        .unwrap();
    let error = crear_cliente(&contexto.estado, &actor, datos("Otra", "12345678Z"), vec![])
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Ya existe un cliente con el NIF 12345678Z: Ana")
    );
}

#[tokio::test]
async fn detecta_que_otra_persona_edito_antes() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let id = crear_cliente(&contexto.estado, &actor, datos("Ana", ""), vec![])
        .await
        .unwrap();
    actualizar_cliente(&contexto.estado, &actor, (id, 1), datos("Ana María", ""))
        .await
        .unwrap();
    let tarde = actualizar_cliente(&contexto.estado, &actor, (id, 1), datos("Ana Luisa", "")).await;
    assert!(matches!(tarde, Err(ErrorApp::Conflicto)));
    let ficha = cargar_cliente(&contexto.estado, &actor, id).await.unwrap();
    assert_eq!(ficha.cliente.nombre.as_deref(), Some("Ana María"));
    assert_eq!(ficha.cliente.version, 2);
}

fn direccion_de_obra() -> dominio::clientes::DatosDeDireccion {
    validar_direccion(&EntradaDeDireccion {
        etiqueta: "Obra".into(),
        calle: "Calle Mayor 3".into(),
        ..EntradaDeDireccion::default()
    })
    .unwrap()
    .unwrap()
}

#[tokio::test]
async fn gestiona_direcciones_y_exporta_la_ficha() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let obra = direccion_de_obra();
    let id = crear_cliente(
        &contexto.estado,
        &actor,
        datos("Ana", ""),
        vec![obra.clone()],
    )
    .await
    .unwrap();
    anadir_direccion(&contexto.estado, &actor, id, obra)
        .await
        .unwrap();
    let ficha = cargar_cliente(&contexto.estado, &actor, id).await.unwrap();
    assert_eq!(ficha.direcciones.len(), 2);
    let primera = ficha.direcciones[0].id;
    quitar_direccion(&contexto.estado, &actor, (id, primera))
        .await
        .unwrap();
    let csv = exportar_cliente(&contexto.estado, &actor, id)
        .await
        .unwrap();
    assert!(csv.contains("nombre;Ana\r\n"));
    assert!(csv.contains("direccion_1_etiqueta;Obra\r\n"));
    assert!(!csv.contains("direccion_2_"));
}

#[tokio::test]
async fn archiva_y_borra_del_todo_con_sus_direcciones() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let id = crear_cliente(
        &contexto.estado,
        &actor,
        datos("Ana", ""),
        vec![direccion_de_obra()],
    )
    .await
    .unwrap();
    archivar_cliente(&contexto.estado, &actor, id, true)
        .await
        .unwrap();
    assert_eq!(
        listar_clientes(&contexto.estado, &actor, consulta(""))
            .await
            .unwrap()
            .total,
        0
    );
    borrar_cliente(&contexto.estado, &actor, id).await.unwrap();
    assert!(matches!(
        cargar_cliente(&contexto.estado, &actor, id).await,
        Err(ErrorApp::NoEncontrado)
    ));
}

fn operario_con(propietario: &Actor, permisos: &[Permiso]) -> Actor {
    Actor {
        rol: Rol::Operario,
        permisos: Permisos::con(permisos.iter().copied()),
        ..propietario.clone()
    }
}

#[tokio::test]
async fn el_operario_ve_los_clientes_pero_sin_permiso_no_los_da_de_alta() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let id = crear_cliente(&contexto.estado, &propietario, datos("Ana", ""), vec![])
        .await
        .unwrap();
    let operario = operario_con(&propietario, &[]);
    let lista = listar_clientes(&contexto.estado, &operario, consulta(""))
        .await
        .unwrap();
    assert_eq!(lista.total, 1);
    assert!(
        cargar_cliente(&contexto.estado, &operario, id)
            .await
            .is_ok()
    );
    let alta = crear_cliente(&contexto.estado, &operario, datos("Luis", ""), vec![]).await;
    assert!(matches!(alta, Err(ErrorApp::NoEncontrado)));
}

#[tokio::test]
async fn con_permiso_da_de_alta_pero_nunca_archiva_borra_ni_exporta() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let operario = operario_con(&propietario, &Permiso::TODOS);
    let id = crear_cliente(&contexto.estado, &operario, datos("Luis", ""), vec![])
        .await
        .unwrap();
    let archivar = archivar_cliente(&contexto.estado, &operario, id, true).await;
    assert!(matches!(archivar, Err(ErrorApp::NoEncontrado)));
    let borrar = borrar_cliente(&contexto.estado, &operario, id).await;
    assert!(matches!(borrar, Err(ErrorApp::NoEncontrado)));
    let exportar = exportar_cliente(&contexto.estado, &operario, id).await;
    assert!(matches!(exportar, Err(ErrorApp::NoEncontrado)));
}

#[tokio::test]
async fn el_operario_no_ve_los_clientes_archivados() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let id = crear_cliente(&contexto.estado, &propietario, datos("Ana", ""), vec![])
        .await
        .unwrap();
    archivar_cliente(&contexto.estado, &propietario, id, true)
        .await
        .unwrap();
    let operario = operario_con(&propietario, &Permiso::TODOS);
    let archivados = ConsultaDeClientes {
        archivados: true,
        ..consulta("")
    };
    let lista = listar_clientes(&contexto.estado, &operario, archivados)
        .await
        .unwrap();
    assert_eq!(lista.total, 0);
    let ficha = cargar_cliente(&contexto.estado, &operario, id).await;
    assert!(matches!(ficha, Err(ErrorApp::NoEncontrado)));
}
