#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use comun::{Contexto, PIN, destino, formulario, get, post, texto};
use dominio::unidades::Porcentaje;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;

/// Trabajo 1 aceptado; devuelve el token y el CSRF del propietario y el id del operario.
async fn preparado(contexto: &Contexto) -> (String, String, i64) {
    let (token, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    let linea = (
        ventana(150, 120),
        Porcentaje::desde_puntos_basicos(2_100),
        false,
    );
    guardar_ventana(&estado, &actor, (id, 0), linea)
        .await
        .unwrap();
    aceptar_presupuesto(&estado, &actor, id).await.unwrap();
    let operario = contexto.con_operario(&actor).await;
    (token, actor.token_csrf, operario)
}

async fn token_de_operario(contexto: &Contexto, operario: i64) -> String {
    let solicitud = SolicitudDeEntrada {
        usuario_id: operario,
        clave: PIN.into(),
        ip: "10.0.0.7".into(),
        duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
    };
    match entrar(&contexto.estado, solicitud).await.unwrap() {
        ResultadoDeEntrada::Correcta { token, .. } => token,
        otro => panic!("el operario no pudo entrar: {otro:?}"),
    }
}

#[tokio::test]
async fn el_inicio_y_la_ficha_muestran_el_trabajo_y_la_orden_se_imprime_en_mm() {
    let contexto = Contexto::nuevo().await;
    let (token, _, _) = preparado(&contexto).await;
    let inicio = texto(contexto.pedir(get("/", Some(&token))).await).await;
    assert!(inicio.contains("T-2026-0001") && inicio.contains("Sin asignar"));
    let ficha = texto(contexto.pedir(get("/trabajos/1", Some(&token))).await).await;
    assert!(ficha.contains("Ana Ruiz") && ficha.contains("Orden de taller"));
    let orden = texto(contexto.pedir(get("/trabajos/1/orden", Some(&token))).await).await;
    assert!(orden.contains("ventana 1 de 1"));
    assert!(
        orden.contains(">1500 × 1200<"),
        "la hoja de taller va en mm"
    );
    assert!(orden.contains("Plan de corte del trabajo") && orden.contains("barra-corte"));
    assert!(!orden.contains('€'), "la orden no lleva precios");
}

/// Asigna el trabajo 1 al operario y devuelve su token y su CSRF (leído de su pantalla).
async fn operario_asignado(contexto: &Contexto) -> (String, String) {
    let (token, csrf, operario) = preparado(contexto).await;
    let id = operario.to_string();
    let asignar = formulario(&[("_csrf", &csrf), ("version", "1"), ("operario", &id)]);
    let respuesta = contexto
        .pedir(post("/trabajos/1/datos", Some(&token), &asignar))
        .await;
    assert_eq!(destino(&respuesta), Some("/trabajos/1?aviso=guardado"));
    let juan = token_de_operario(contexto, operario).await;
    let ventana = texto(contexto.pedir(get("/mi-dia/1", Some(&juan))).await).await;
    let csrf = ventana.split("name=\"_csrf\" value=\"").nth(1).unwrap();
    let csrf = csrf.split('"').next().unwrap().to_owned();
    (juan, csrf)
}

#[tokio::test]
async fn el_operario_ve_su_ventana_sin_precios_y_no_entra_en_los_trabajos() {
    let contexto = Contexto::nuevo().await;
    let (juan, _) = operario_asignado(&contexto).await;
    let mi_dia = texto(contexto.pedir(get("/mi-dia", Some(&juan))).await).await;
    assert!(mi_dia.contains("/mi-dia/1") && !mi_dia.contains('€'));
    let ventana = texto(contexto.pedir(get("/mi-dia/1", Some(&juan))).await).await;
    assert!(ventana.contains("Cortar") && !ventana.contains('€'));
    for ruta in ["/trabajos", "/trabajos/1", "/trabajos/1/orden"] {
        let respuesta = contexto.pedir(get(ruta, Some(&juan))).await;
        assert_eq!(respuesta.status(), StatusCode::NOT_FOUND, "{ruta}");
    }
}

#[tokio::test]
async fn el_operario_marca_un_paso_y_un_segundo_toque_avisa() {
    let contexto = Contexto::nuevo().await;
    let (juan, csrf) = operario_asignado(&contexto).await;
    let campos = [
        ("_csrf", csrf.as_str()),
        ("version", "1"),
        ("ventana", "1"),
        ("accion", "terminar"),
    ];
    let terminar = formulario(&campos);
    let respuesta = contexto
        .pedir(post("/mi-dia/pasos/1", Some(&juan), &terminar))
        .await;
    assert_eq!(destino(&respuesta), Some("/mi-dia/1"));
    let otra_vez = contexto
        .pedir(post("/mi-dia/pasos/1", Some(&juan), &terminar))
        .await;
    assert_eq!(
        otra_vez.status(),
        StatusCode::CONFLICT,
        "el segundo toque avisa y recarga"
    );
}
