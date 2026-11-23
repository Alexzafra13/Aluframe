#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use comun::{Contexto, destino, get, post, texto};
use servidor::aplicacion::importar_serie::{importar_serie, previsualizar_serie};
use servidor::aplicacion::series::{cargar_serie, listar_series};
use servidor::errores::ErrorApp;

fn archivo(ruta: &str) -> String {
    std::fs::read_to_string(format!("{}/../../{ruta}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn formulario(csrf: &str, yaml: &str) -> String {
    let mut codificado = String::new();
    for byte in yaml.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                codificado.push(char::from(byte))
            }
            otro => codificado.push_str(&format!("%{otro:02X}")),
        }
    }
    format!("_csrf={csrf}&texto={codificado}")
}

#[tokio::test]
async fn importa_las_series_reales_y_las_lista() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    for ruta in ["data/series/aluprom-21.yaml", "data/series/sopena-v94.yaml"] {
        let vista = previsualizar_serie(&contexto.estado, &actor, archivo(ruta))
            .await
            .unwrap();
        assert!(vista.se_puede_importar(), "{ruta}: {:?}", vista.errores);
        assert!(!vista.avisos.is_empty());
        importar_serie(&contexto.estado, &actor, archivo(ruta))
            .await
            .unwrap();
    }
    let series = listar_series(&contexto.estado, &actor).await.unwrap();
    let codigos: Vec<&str> = series.iter().map(|s| s.serie.codigo.as_str()).collect();
    assert_eq!(codigos, vec!["ALUPROM_21", "SOPENA_V94"]);
}

#[tokio::test]
async fn una_version_igual_no_se_importa_dos_veces_y_una_distinta_es_la_siguiente() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let original = archivo("data/series/aluprom-21.yaml");
    importar_serie(&contexto.estado, &actor, original.clone())
        .await
        .unwrap();
    let repetida = previsualizar_serie(&contexto.estado, &actor, original.clone())
        .await
        .unwrap();
    assert!(
        repetida
            .errores
            .iter()
            .any(|e| e.contains("no hay cambios"))
    );
    assert!(
        importar_serie(&contexto.estado, &actor, original.clone())
            .await
            .is_err()
    );
    let cambiada = original.replace(
        "descuento_hoja_vertical: 5.3",
        "descuento_hoja_vertical: 5.4",
    );
    let (_, version) = importar_serie(&contexto.estado, &actor, cambiada)
        .await
        .unwrap();
    assert_eq!(version, 2);
    let guardada = cargar_serie(&contexto.estado, &actor, "ALUPROM_21".into())
        .await
        .unwrap();
    assert_eq!(guardada.numero_version, 2);
}

#[tokio::test]
async fn un_archivo_con_errores_no_guarda_nada() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let roto = archivo("data/series/aluprom-21.yaml")
        .replace("ancho - descuento_marco_horizontal", "ancho - descuentto");
    let vista = previsualizar_serie(&contexto.estado, &actor, roto.clone())
        .await
        .unwrap();
    assert!(!vista.se_puede_importar());
    assert!(
        vista.errores.iter().any(|e| e.contains("descuentto")),
        "{:?}",
        vista.errores
    );
    assert!(matches!(
        importar_serie(&contexto.estado, &actor, roto).await,
        Err(ErrorApp::Validacion(_))
    ));
    let no_yaml = previsualizar_serie(&contexto.estado, &actor, "serie: [".into())
        .await
        .unwrap();
    assert!(no_yaml.errores[0].contains("YAML"));
    assert!(
        listar_series(&contexto.estado, &actor)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn la_serie_demo_solo_se_acepta_si_se_permite() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let demo = archivo("fixtures/demo/serie-demo.yaml");
    let vista = previsualizar_serie(&contexto.estado, &actor, demo.clone())
        .await
        .unwrap();
    assert!(
        vista.errores.iter().any(|e| e.contains("demostración")),
        "{:?}",
        vista.errores
    );
    let estado_demo = contexto.estado.clone().con_datos_demo(true);
    importar_serie(&estado_demo, &actor, demo).await.unwrap();
}

#[tokio::test]
async fn importar_y_probar_por_las_pantallas() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cuerpo = formulario(&actor.token_csrf, &archivo("data/series/sopena-v94.yaml"));
    let revision = contexto
        .pedir(post(
            "/catalogo/series/importar/revisar",
            Some(&token),
            &cuerpo,
        ))
        .await;
    assert!(texto(revision).await.contains("Confirmar e importar"));
    let importada = contexto
        .pedir(post("/catalogo/series/importar", Some(&token), &cuerpo))
        .await;
    assert_eq!(
        destino(&importada),
        Some("/catalogo/series?importada=SOPENA_V94&version=1")
    );
    let ruta = "/catalogo/series/SOPENA_V94/probar?tipologia=europea_abatible_2_hojas&ancho=120,0&alto=140";
    let prueba = texto(contexto.pedir(get(ruta, Some(&token))).await).await;
    assert!(
        prueba.contains("135,9") && prueba.contains("57,5") && prueba.contains("BISAGRA"),
        "{prueba}"
    );
    let mala = texto(
        contexto
            .pedir(get(
                "/catalogo/series/SOPENA_V94/probar?ancho=abc&alto=1",
                Some(&token),
            ))
            .await,
    )
    .await;
    assert!(mala.contains("Escribe el ancho en cm"));
    let inexistente = contexto
        .pedir(get("/catalogo/series/NO_EXISTE/probar", Some(&token)))
        .await;
    assert_eq!(inexistente.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn el_operario_no_ve_el_catalogo() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    contexto.con_operario(&propietario).await;
    let operario = servidor::aplicacion::Actor {
        id: 2,
        nombre: "Juan".into(),
        rol: dominio::acceso::Rol::Operario,
        token_csrf: String::new(),
        huella_sesion: String::new(),
        permisos: Default::default(),
        en_demostracion: false,
    };
    let resultado = listar_series(&contexto.estado, &operario).await;
    assert!(matches!(resultado, Err(ErrorApp::NoEncontrado)));
}
