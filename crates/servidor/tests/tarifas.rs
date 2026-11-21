#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use comun::{Contexto, destino, formulario, post, texto};
use dominio::catalogo::{EntradaDeProveedor, validar_proveedor};
use dominio::unidades::Porcentaje;
use servidor::aplicacion::Actor;
use servidor::aplicacion::catalogo::{crear_proveedor, listar_articulos};
use servidor::aplicacion::importar_tarifa::*;
use servidor::aplicacion::tarifas::*;
use servidor::bd::consultas::articulos::FiltroArticulos;

const TARIFA_1: &str = "referencia;descripcion;unidad;precio_tarifa;longitud_barra\nAL-1;Marco lateral;barra;20,00;640\nAL-2;Hoja;barra;15,5;640\n";
const TARIFA_2: &str = "referencia;descripción;unidad;precio_tarifa;longitud_barra\nAL-1;Marco lateral;barra;22,00;640\nAL-3;Inversor;barra;9;640\n";
const CON_ERRORES: &str = "referencia;descripcion;unidad;precio_tarifa;longitud_barra\nAL-1;Marco;barra;0;640\nAL-2;Hoja;saco;3;\n";

async fn preparar() -> (Contexto, String, Actor) {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let datos = validar_proveedor(&EntradaDeProveedor {
        nombre: "Aluval".into(),
        ..EntradaDeProveedor::default()
    })
    .unwrap();
    crear_proveedor(&contexto.estado, &actor, datos)
        .await
        .unwrap();
    (contexto, token, actor)
}

async fn importar(contexto: &Contexto, actor: &Actor, texto: &str) -> i64 {
    let previa = previsualizar_tarifa(&contexto.estado, actor, 1, texto.into())
        .await
        .unwrap();
    assert!(previa.se_puede_importar(), "{:?}", previa.errores);
    let cabecera = CabeceraDeTarifa {
        proveedor_id: 1,
        nombre: "Tarifa".into(),
        ..CabeceraDeTarifa::default()
    };
    importar_tarifa(
        &contexto.estado,
        actor,
        (cabecera, texto.into(), previa.sha256),
    )
    .await
    .unwrap()
    .0
}

async fn precios(contexto: &Contexto, actor: &Actor) -> Vec<(String, Option<i64>)> {
    let filtro = FiltroArticulos {
        proveedor_id: Some(1),
        texto: String::new(),
        archivados: false,
    };
    listar_articulos(&contexto.estado, actor, filtro)
        .await
        .unwrap()
        .into_iter()
        .map(|a| (a.referencia, a.precio_pu))
        .collect()
}

/// Criterio de aceptación de H2: un CSV con errores muestra los avisos y no guarda nada.
#[tokio::test]
async fn un_csv_con_errores_muestra_los_avisos_y_no_guarda_nada() {
    let (contexto, token, actor) = preparar().await;
    let revisar = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("proveedor", "1"),
        ("texto", CON_ERRORES),
    ]);
    let respuesta = contexto
        .pedir(post(
            "/catalogo/tarifas/importar/revisar",
            Some(&token),
            &revisar,
        ))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = texto(respuesta).await;
    assert!(html.contains("línea 2: el precio «0» debe ser mayor que cero"));
    assert!(html.contains("línea 3: unidad desconocida «saco»"));
    assert!(!html.contains("Paso 3: confirmar"));
    let sha = previsualizar_tarifa(&contexto.estado, &actor, 1, CON_ERRORES.into())
        .await
        .unwrap()
        .sha256;
    let forzar = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("proveedor", "1"),
        ("texto", CON_ERRORES),
        ("sha256", &sha),
    ]);
    let respuesta = contexto
        .pedir(post("/catalogo/tarifas/importar", Some(&token), &forzar))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(precios(&contexto, &actor).await.is_empty());
    assert!(
        listar_tarifas(&contexto.estado, &actor)
            .await
            .unwrap()
            .is_empty()
    );
}

/// Criterio de aceptación de H2: deshacer restaura los precios.
#[tokio::test]
async fn deshacer_restaura_los_precios_y_quita_los_articulos_nuevos() {
    let (contexto, _, actor) = preparar().await;
    importar(&contexto, &actor, TARIFA_1).await;
    let antes = precios(&contexto, &actor).await;
    assert_eq!(
        antes,
        vec![
            ("AL-1".into(), Some(200_000)),
            ("AL-2".into(), Some(155_000))
        ]
    );
    let segunda = importar(&contexto, &actor, TARIFA_2).await;
    let despues = precios(&contexto, &actor).await;
    assert_eq!(
        despues,
        vec![
            ("AL-1".into(), Some(220_000)),
            ("AL-2".into(), Some(155_000)),
            ("AL-3".into(), Some(90_000))
        ]
    );
    deshacer_tarifa(&contexto.estado, &actor, segunda)
        .await
        .unwrap();
    assert_eq!(precios(&contexto, &actor).await, antes);
    let tarifas = listar_tarifas(&contexto.estado, &actor).await.unwrap();
    assert_eq!(tarifas.len(), 1);
    assert_eq!(tarifas[0].estado, "vigente");
}

#[tokio::test]
async fn la_vista_previa_compara_con_la_vigente_sin_guardar() {
    let (contexto, _, actor) = preparar().await;
    importar(&contexto, &actor, TARIFA_1).await;
    let previa = previsualizar_tarifa(&contexto.estado, &actor, 1, TARIFA_2.into())
        .await
        .unwrap();
    assert_eq!(previa.comparacion.nuevos.len(), 1);
    assert_eq!(previa.comparacion.cambios.len(), 1);
    assert_eq!(previa.comparacion.ausentes, vec!["AL-2"]);
    assert_eq!(
        precios(&contexto, &actor).await.len(),
        2,
        "la vista previa no crea artículos"
    );
    let repetida = previsualizar_tarifa(&contexto.estado, &actor, 1, TARIFA_1.into())
        .await
        .unwrap();
    assert!(repetida.errores[0].contains("no hay cambios"));
}

#[tokio::test]
async fn solo_se_deshace_la_version_vigente_y_un_archivo_cambiado_se_rechaza() {
    let (contexto, _, actor) = preparar().await;
    let primera = importar(&contexto, &actor, TARIFA_1).await;
    importar(&contexto, &actor, TARIFA_2).await;
    let error = deshacer_tarifa(&contexto.estado, &actor, primera)
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Solo se puede deshacer la última versión")
    );
    let previa = previsualizar_tarifa(&contexto.estado, &actor, 1, TARIFA_1.into())
        .await
        .unwrap();
    let cabecera = CabeceraDeTarifa {
        proveedor_id: 1,
        nombre: "X".into(),
        ..CabeceraDeTarifa::default()
    };
    let cambiado = TARIFA_1.replace("20,00", "21,00");
    let error = importar_tarifa(
        &contexto.estado,
        &actor,
        (cabecera, cambiado, previa.sha256),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("no es el que se revisó"));
}

#[tokio::test]
async fn el_ajuste_y_la_edicion_crean_versiones_que_se_deshacen() {
    let (contexto, _, actor) = preparar().await;
    importar(&contexto, &actor, TARIFA_1).await;
    let subir = Porcentaje::desde_puntos_basicos(1_000);
    assert_eq!(
        aplicar_ajuste(&contexto.estado, &actor, (1, vec![1]), subir)
            .await
            .unwrap(),
        1
    );
    assert_eq!(precios(&contexto, &actor).await[0].1, Some(220_000));
    let escritos = vec![(2, "16,25".to_owned()), (1, "22,00".to_owned())];
    assert_eq!(
        guardar_precios(&contexto.estado, &actor, 1, escritos)
            .await
            .unwrap(),
        1
    );
    assert_eq!(precios(&contexto, &actor).await[1].1, Some(162_500));
    let mal = guardar_precios(&contexto.estado, &actor, 1, vec![(2, "gratis".into())])
        .await
        .unwrap_err();
    assert!(mal.to_string().contains("No se ha guardado nada. AL-2:"));
    let vigente = listar_tarifas(&contexto.estado, &actor).await.unwrap()[0].id;
    deshacer_tarifa(&contexto.estado, &actor, vigente)
        .await
        .unwrap();
    assert_eq!(precios(&contexto, &actor).await[1].1, Some(155_000));
    let cero = aplicar_ajuste(&contexto.estado, &actor, (1, vec![1]), Porcentaje::CERO).await;
    assert!(cero.is_err());
}

#[tokio::test]
async fn la_importacion_por_pantalla_lleva_a_las_tarifas() {
    let (contexto, token, actor) = preparar().await;
    let previa = previsualizar_tarifa(&contexto.estado, &actor, 1, TARIFA_1.into())
        .await
        .unwrap();
    let confirmar = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("proveedor", "1"),
        ("nombre", ""),
        ("fecha", "2026-11-01"),
        ("descuento", "12,5"),
        ("texto", TARIFA_1),
        ("sha256", &previa.sha256),
    ]);
    let respuesta = contexto
        .pedir(post("/catalogo/tarifas/importar", Some(&token), &confirmar))
        .await;
    assert_eq!(destino(&respuesta), Some("/catalogo/tarifas?importada=2"));
    let tarifa = &listar_tarifas(&contexto.estado, &actor).await.unwrap()[0];
    assert_eq!(tarifa.descuento_compra_bp, 1_250);
    assert_eq!(tarifa.fecha_tarifa.as_deref(), Some("2026-11-01"));
    assert!(tarifa.nombre.starts_with("Importada el"));
}
