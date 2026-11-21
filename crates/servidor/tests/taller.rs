#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use comun::{Contexto, destino, get, post, texto};
use dominio::unidades::Longitud;
use servidor::aplicacion::importar_serie::importar_serie;
use servidor::aplicacion::taller::leer_ajustes_de_taller;

#[tokio::test]
async fn por_defecto_la_sierra_es_de_2_mm_provisional() {
    let contexto = Contexto::nuevo().await;
    let ajustes = leer_ajustes_de_taller(&contexto.estado).await.unwrap();
    assert_eq!(ajustes.grosor_sierra, Longitud::desde_decimas_de_mm(20));
    assert_eq!(ajustes.recorte_extremos, Longitud::CERO);
    assert!(!ajustes.confirmados);
}

#[tokio::test]
async fn el_propietario_guarda_los_ajustes_del_taller() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let formulario = format!(
        "_csrf={}&grosor_sierra_mm=2,5&recorte_extremos_mm=10&retal_minimo_cm=50",
        actor.token_csrf
    );
    let respuesta = contexto
        .pedir(post("/ajustes/taller", Some(&token), &formulario))
        .await;
    assert_eq!(destino(&respuesta), Some("/ajustes/taller?guardado=si"));
    let ajustes = leer_ajustes_de_taller(&contexto.estado).await.unwrap();
    assert_eq!(ajustes.grosor_sierra, Longitud::desde_decimas_de_mm(25));
    assert_eq!(ajustes.recorte_extremos, Longitud::desde_decimas_de_mm(100));
    assert_eq!(ajustes.retal_minimo, Longitud::desde_decimas_de_mm(5_000));
    assert!(ajustes.confirmados);
    let pagina = texto(
        contexto
            .pedir(get("/ajustes/taller?guardado=si", Some(&token)))
            .await,
    )
    .await;
    assert!(pagina.contains("Ajustes del taller guardados.") && pagina.contains("value=\"2,5\""));
}

#[tokio::test]
async fn rechaza_valores_imposibles_sin_guardar_nada() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let formulario = format!(
        "_csrf={}&grosor_sierra_mm=-1&recorte_extremos_mm=0&retal_minimo_cm=0",
        actor.token_csrf
    );
    let respuesta = contexto
        .pedir(post("/ajustes/taller", Some(&token), &formulario))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        !leer_ajustes_de_taller(&contexto.estado)
            .await
            .unwrap()
            .confirmados
    );
}

#[tokio::test]
async fn el_corte_de_la_abatible_usa_la_barra_de_640_y_la_sierra() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let yaml = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/series/sopena-v94.yaml"
    ))
    .unwrap();
    importar_serie(&contexto.estado, &actor, yaml)
        .await
        .unwrap();
    let ruta =
        "/catalogo/series/SOPENA_V94/probar?tipologia=europea_abatible_2_hojas&ancho=120&alto=140";
    let pagina = texto(contexto.pedir(get(ruta, Some(&token))).await).await;
    assert!(
        pagina.contains("Sierra 2 mm") && pagina.contains("provisionales"),
        "{pagina}"
    );
    assert!(pagina.contains("HOJA_ALTO: 1 barra"));
    assert!(
        pagina.contains("sobra 95,6 cm"),
        "4 × (135,9 + 0,2) = 544,4 → sobran 95,6"
    );
}

#[tokio::test]
async fn el_operario_no_ve_los_ajustes_del_taller() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let operario = contexto.con_operario(&propietario).await;
    let solicitud = servidor::aplicacion::entrar::SolicitudDeEntrada {
        usuario_id: operario,
        clave: comun::PIN.into(),
        ip: "10.0.0.9".into(),
        duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
    };
    let servidor::aplicacion::entrar::ResultadoDeEntrada::Correcta { token, .. } =
        servidor::aplicacion::entrar::entrar(&contexto.estado, solicitud)
            .await
            .unwrap()
    else {
        panic!("el operario no pudo entrar");
    };
    let respuesta = contexto.pedir(get("/ajustes/taller", Some(&token))).await;
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
}
