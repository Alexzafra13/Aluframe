#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use comun::{Contexto, destino, formulario, get, post, texto};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::Actor;
use servidor::aplicacion::clientes::{
    ConsultaDeClientes, actualizar_cliente, crear_cliente, listar_clientes,
};
use servidor::aplicacion::importar_clientes::*;
use servidor::errores::ErrorApp;

const CORRECTO: &str =
    "nombre;nif;telefono;localidad\nAna Ruiz;12345678Z;924000000;Zafra\nLuis;;;Mérida\n";

async fn total(contexto: &Contexto, actor: &Actor) -> i64 {
    let consulta = ConsultaDeClientes {
        texto: String::new(),
        archivados: false,
        pagina: 1,
    };
    listar_clientes(&contexto.estado, actor, consulta)
        .await
        .unwrap()
        .total
}

#[tokio::test]
async fn un_csv_con_errores_avisa_y_no_guarda_nada() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let malo = "nombre;nif\nAna;12345678A\nLuis;\n";
    let cuerpo = formulario(&[("_csrf", &actor.token_csrf), ("texto", malo)]);
    let respuesta = contexto
        .pedir(post("/clientes/importar/revisar", Some(&token), &cuerpo))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = texto(respuesta).await;
    assert!(html.contains("línea 2: nif:"));
    assert!(!html.contains("confirmar e importar"));
    assert_eq!(total(&contexto, &actor).await, 0);
    let sha = previsualizar_clientes(&contexto.estado, &actor, malo.into())
        .await
        .unwrap()
        .sha256;
    let forzado = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("texto", malo),
        ("sha256", &sha),
    ]);
    let respuesta = contexto
        .pedir(post("/clientes/importar", Some(&token), &forzado))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(total(&contexto, &actor).await, 0);
}

#[tokio::test]
async fn importa_tras_revisar_y_salta_los_nif_existentes() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let ana = validar_cliente(&EntradaDeCliente {
        nif: "12345678Z".into(),
        ..EntradaDeCliente::default()
    })
    .unwrap();
    crear_cliente(&contexto.estado, &actor, ana, vec![])
        .await
        .unwrap();
    let vista = previsualizar_clientes(&contexto.estado, &actor, CORRECTO.into())
        .await
        .unwrap();
    assert!(vista.se_puede_importar());
    assert_eq!(vista.clientes.len(), 1);
    assert!(vista.avisos[0].contains("ya hay un cliente con el NIF 12345678Z"));
    let cuerpo = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("texto", CORRECTO),
        ("sha256", &vista.sha256),
    ]);
    let respuesta = contexto
        .pedir(post("/clientes/importar", Some(&token), &cuerpo))
        .await;
    assert_eq!(
        destino(&respuesta),
        Some("/clientes?importados=1&importacion=1")
    );
    let lista = texto(
        contexto
            .pedir(get("/clientes?importados=1&importacion=1", Some(&token)))
            .await,
    )
    .await;
    assert!(lista.contains("Importados 1 clientes."));
    assert!(lista.contains("Deshacer esta importación"));
    assert_eq!(total(&contexto, &actor).await, 2);
}

#[tokio::test]
async fn rechaza_un_archivo_distinto_del_revisado() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let vista = previsualizar_clientes(&contexto.estado, &actor, CORRECTO.into())
        .await
        .unwrap();
    let cambiado = CORRECTO.replace("Luis", "Pedro");
    let error = importar_clientes(&contexto.estado, &actor, cambiado, vista.sha256)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("no es el que se revisó"));
    assert_eq!(total(&contexto, &actor).await, 0);
}

#[tokio::test]
async fn deshacer_quita_lo_importado_salvo_lo_modificado() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let vista = previsualizar_clientes(&contexto.estado, &actor, CORRECTO.into())
        .await
        .unwrap();
    let (importacion, n) =
        importar_clientes(&contexto.estado, &actor, CORRECTO.into(), vista.sha256)
            .await
            .unwrap();
    assert_eq!(n, 2);
    let luis = validar_cliente(&EntradaDeCliente {
        nombre: "Luis Gómez".into(),
        ..EntradaDeCliente::default()
    })
    .unwrap();
    actualizar_cliente(&contexto.estado, &actor, (2, 1), luis)
        .await
        .unwrap();
    let resultado = deshacer_importacion_de_clientes(&contexto.estado, &actor, importacion)
        .await
        .unwrap();
    assert_eq!(resultado, (1, 1));
    assert_eq!(total(&contexto, &actor).await, 1);
    let otra_vez = deshacer_importacion_de_clientes(&contexto.estado, &actor, importacion).await;
    assert!(matches!(otra_vez, Err(ErrorApp::Validacion(_))));
}
