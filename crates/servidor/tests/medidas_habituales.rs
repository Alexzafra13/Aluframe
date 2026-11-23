#![allow(clippy::unwrap_used, clippy::panic)]

//! Medidas habituales e historial para valorar los sobrantes del corte (ADR-0026).

mod comun;

use axum::http::StatusCode;
use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use comun::{Contexto, destino, formulario, get, post, texto};
use dominio::unidades::{Longitud, Porcentaje};
use servidor::aplicacion::Actor;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::medidas_habituales::valoraciones_del_trabajo;
use servidor::aplicacion::presupuestos::crear_presupuesto;
use servidor::aplicacion::series_incluidas::cargar_series_incluidas;
use servidor::estado::EstadoApp;

fn cm(valor: i64) -> Longitud {
    Longitud::desde_decimas_de_mm(valor * 100)
}

fn clave(serie: &str, perfil: &str) -> (String, String) {
    (serie.into(), perfil.into())
}

#[tokio::test]
async fn la_medida_de_partida_es_la_120_por_120_de_aluprom_21() {
    let contexto = Contexto::nuevo().await;
    let (token, _) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let pagina = texto(contexto.pedir(get("/ajustes/taller", Some(&token))).await).await;
    assert!(
        pagina.contains("Aluprom 21 · Ventana corredera 2 hojas · 120 × 120 cm"),
        "{pagina}"
    );
    assert!(!pagina.contains("No se usa"));
    let valoraciones = contexto
        .estado
        .bd
        .leer(|con| valoraciones_del_trabajo(con, (1, "2026-11-21T08:00:00Z")))
        .await
        .unwrap();
    assert_eq!(
        valoraciones[&clave("ALUPROM_21", "MARCO_LATERAL")].habituales,
        vec![(cm(120), 10_000)],
        "2 marcos laterales de 120 cm, como cinco ventanas"
    );
}

const PRUEBA: &str = "PRUEBA_COMPLETA|corredera_2_hojas_demo";
const RUTA: &str = "/ajustes/taller/habituales";

/// La serie de prueba cargada y una medida suya de 150 × 120 añadida desde Ajustes > Taller.
async fn con_una_anadida(contexto: &Contexto) -> (String, Actor) {
    let (token, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    let bien = contexto
        .pedir(post(RUTA, Some(&token), &anadir(&actor, PRUEBA, "150")))
        .await;
    assert_eq!(destino(&bien), Some("/ajustes/taller?guardado=si"));
    (token, actor)
}

fn anadir(actor: &Actor, tipologia: &str, ancho: &str) -> String {
    formulario(&[
        ("_csrf", &actor.token_csrf),
        ("tipologia", tipologia),
        ("ancho", ancho),
        ("alto", "120"),
    ])
}

#[tokio::test]
async fn se_anade_una_medida_y_se_rechaza_lo_que_no_vale() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = con_una_anadida(&contexto).await;
    let pagina = texto(contexto.pedir(get("/ajustes/taller", Some(&token))).await).await;
    assert!(pagina.contains("Prueba completa · DEMO corredera 2 hojas (ficticia) · 150 × 120 cm"));
    assert!(
        pagina.contains("No se usa: la serie no está en el programa"),
        "Aluprom 21 no está cargada en esta prueba"
    );
    for (tipologia, ancho, motivo) in [
        (PRUEBA, "150", "ya está"),
        (PRUEBA, "abc", "ancho"),
        ("NO_EXISTE|x", "100", "No se puede usar"),
    ] {
        let mal = contexto
            .pedir(post(RUTA, Some(&token), &anadir(&actor, tipologia, ancho)))
            .await;
        assert_eq!(mal.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(texto(mal).await.contains(motivo), "{motivo}");
    }
}

#[tokio::test]
async fn se_quita_una_medida() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = con_una_anadida(&contexto).await;
    let csrf = formulario(&[("_csrf", &actor.token_csrf)]);
    let quitar = contexto
        .pedir(post(
            "/ajustes/taller/habituales/0/quitar",
            Some(&token),
            &csrf,
        ))
        .await;
    assert_eq!(destino(&quitar), Some("/ajustes/taller?guardado=si"));
    let pagina = texto(contexto.pedir(get("/ajustes/taller", Some(&token))).await).await;
    assert!(!pagina.contains("Aluprom 21 · Corredera"));
    assert!(pagina.contains("150 × 120 cm"));
}

async fn aceptar_una(estado: &EstadoApp, actor: &Actor, (ancho, alto): (i64, i64)) {
    let id = crear_presupuesto(estado, actor, 1, None).await.unwrap();
    let iva = Porcentaje::desde_puntos_basicos(2_100);
    guardar_ventana(estado, actor, (id, 0), (ventana(ancho, alto), iva, false))
        .await
        .unwrap();
    aceptar_presupuesto(estado, actor, id).await.unwrap();
}

/// Cada trabajo aprende de los aceptados antes que él, no de sí mismo ni de los posteriores: así
/// su hoja no cambia por lo que se acepte después.
#[tokio::test]
async fn el_historial_es_el_de_los_trabajos_aceptados_antes() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    aceptar_una(&estado, &actor, (150, 120)).await;
    aceptar_una(&estado, &actor, (100, 80)).await;
    let de = |trabajo: i64| {
        estado
            .bd
            .leer(move |con| valoraciones_del_trabajo(con, (trabajo, "2026-11-21T08:00:00Z")))
    };
    let primero = de(1).await.unwrap();
    assert!(!primero.contains_key(&clave("PRUEBA_COMPLETA", "DEMO_MARCO_LATERAL")));
    let segundo = de(2).await.unwrap();
    assert_eq!(
        segundo[&clave("PRUEBA_COMPLETA", "DEMO_MARCO_LATERAL")].habituales,
        vec![(cm(120), 2_000)],
        "los 2 marcos laterales de 120 del primer trabajo, aceptado el mismo día"
    );
}
