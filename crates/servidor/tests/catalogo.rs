#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use comun::{Contexto, destino, formulario, get, post, texto};
use dominio::acceso::Rol;
use dominio::catalogo::*;
use servidor::aplicacion::Actor;
use servidor::aplicacion::catalogo::*;
use servidor::bd::consultas::articulos::FiltroArticulos;
use servidor::errores::ErrorApp;

fn proveedor(nombre: &str) -> DatosDeProveedor {
    validar_proveedor(&EntradaDeProveedor {
        nombre: nombre.into(),
        descuento_habitual: "10".into(),
        ..EntradaDeProveedor::default()
    })
    .unwrap()
}

fn articulo(referencia: &str) -> DatosDeArticulo {
    validar_articulo(&EntradaDeArticulo {
        referencia: referencia.into(),
        descripcion: "Marco lateral".into(),
        tipo: "perfil".into(),
        unidad: "barra".into(),
        longitud_barra_cm: "640".into(),
        ..EntradaDeArticulo::default()
    })
    .unwrap()
}

#[tokio::test]
async fn no_duplica_proveedores_ni_referencias() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let id = crear_proveedor(&contexto.estado, &actor, proveedor("Aluval"))
        .await
        .unwrap();
    let otro = crear_proveedor(&contexto.estado, &actor, proveedor("aluval")).await;
    assert!(
        otro.unwrap_err()
            .to_string()
            .contains("Ya existe el proveedor")
    );
    crear_articulo(&contexto.estado, &actor, id, articulo("AL-1"))
        .await
        .unwrap();
    let repetido = crear_articulo(&contexto.estado, &actor, id, articulo("AL-1")).await;
    assert!(
        repetido
            .unwrap_err()
            .to_string()
            .contains("ya tiene la referencia «AL-1»")
    );
    let segundo = crear_proveedor(&contexto.estado, &actor, proveedor("Sopena"))
        .await
        .unwrap();
    crear_articulo(&contexto.estado, &actor, segundo, articulo("AL-1"))
        .await
        .unwrap();
    let filtro = FiltroArticulos {
        proveedor_id: None,
        texto: "al-1".into(),
        archivados: false,
    };
    assert_eq!(
        listar_articulos(&contexto.estado, &actor, filtro)
            .await
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn solo_hay_un_acabado_base() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let blanco = validar_acabado(&EntradaDeAcabado {
        nombre: "Blanco".into(),
        es_base: true,
        recargo: "".into(),
    })
    .unwrap();
    crear_acabado(&contexto.estado, &actor, blanco)
        .await
        .unwrap();
    let otro = validar_acabado(&EntradaDeAcabado {
        nombre: "Crema".into(),
        es_base: true,
        recargo: "".into(),
    })
    .unwrap();
    let error = crear_acabado(&contexto.estado, &actor, otro)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("El acabado base ya es «Blanco»"));
}

#[tokio::test]
async fn avisa_de_ediciones_simultaneas_y_archiva() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let id = crear_proveedor(&contexto.estado, &actor, proveedor("Aluval"))
        .await
        .unwrap();
    let art = crear_articulo(&contexto.estado, &actor, id, articulo("AL-1"))
        .await
        .unwrap();
    actualizar_articulo(&contexto.estado, &actor, (art, 1), articulo("AL-1B"))
        .await
        .unwrap();
    let tarde = actualizar_articulo(&contexto.estado, &actor, (art, 1), articulo("AL-1C")).await;
    assert!(matches!(tarde, Err(ErrorApp::Conflicto)));
    archivar(&contexto.estado, &actor, ("proveedor", id), true)
        .await
        .unwrap();
    assert!(
        listar_proveedores(&contexto.estado, &actor, false)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        listar_proveedores(&contexto.estado, &actor, true)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn las_pantallas_del_catalogo_guardan_y_validan() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let vacio = formulario(&[("_csrf", &actor.token_csrf), ("nombre", ""), ("nif", "X")]);
    let respuesta = contexto
        .pedir(post("/catalogo/proveedores", Some(&token), &vacio))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        texto(respuesta)
            .await
            .contains("escribe el nombre del proveedor")
    );
    let bueno = formulario(&[("_csrf", &actor.token_csrf), ("nombre", "Aluval")]);
    let respuesta = contexto
        .pedir(post("/catalogo/proveedores", Some(&token), &bueno))
        .await;
    assert_eq!(
        destino(&respuesta),
        Some("/catalogo/proveedores?guardado=si")
    );
    let alta = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("proveedor", "1"),
        ("referencia", "AL-1"),
        ("descripcion", "Marco"),
        ("tipo", "perfil"),
        ("unidad", "barra"),
        ("longitud_barra_cm", ""),
    ]);
    let respuesta = contexto
        .pedir(post("/catalogo/articulos/nuevo", Some(&token), &alta))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        texto(respuesta)
            .await
            .contains("necesita la longitud de barra")
    );
}

#[tokio::test]
async fn todas_las_pestanas_del_catalogo_se_abren() {
    let contexto = Contexto::nuevo().await;
    let (token, _) = contexto.con_propietario().await;
    for ruta in [
        "/catalogo/articulos",
        "/catalogo/proveedores",
        "/catalogo/acabados",
        "/catalogo/tarifas",
        "/catalogo/tarifas/importar",
        "/catalogo/articulos/nuevo",
    ] {
        let respuesta = contexto.pedir(get(ruta, Some(&token))).await;
        assert_eq!(respuesta.status(), StatusCode::OK, "{ruta}");
    }
}

#[tokio::test]
async fn el_operario_no_llega_al_catalogo() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let operario = Actor {
        rol: Rol::Operario,
        ..propietario
    };
    assert!(matches!(
        listar_proveedores(&contexto.estado, &operario, true).await,
        Err(ErrorApp::NoEncontrado)
    ));
    let alta = crear_proveedor(&contexto.estado, &operario, proveedor("Aluval")).await;
    assert!(matches!(alta, Err(ErrorApp::NoEncontrado)));
}
