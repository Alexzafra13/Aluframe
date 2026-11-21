#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use comun::{CONTRASENA, Contexto, PIN, destino, get, post, texto, token_de_la_cookie};
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};
use servidor::aplicacion::sesion::identificar;
use time::Duration;

fn solicitud(usuario_id: i64, clave: &str) -> SolicitudDeEntrada {
    SolicitudDeEntrada {
        usuario_id,
        clave: clave.into(),
        ip: "192.168.1.20".into(),
        duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
    }
}

#[tokio::test]
async fn sin_usuarios_todo_lleva_al_primer_arranque() {
    let contexto = Contexto::nuevo().await;
    let respuesta = contexto.pedir(get("/", None)).await;
    assert_eq!(respuesta.status(), StatusCode::SEE_OTHER);
    assert_eq!(destino(&respuesta), Some("/primer-arranque"));
    let salud = contexto.pedir(get("/health", None)).await;
    assert_eq!(salud.status(), StatusCode::OK);
    assert!(texto(salud).await.contains("\"estado\":\"ok\""));
}

#[tokio::test]
async fn el_primer_arranque_solo_funciona_una_vez() {
    let contexto = Contexto::nuevo().await;
    contexto.con_propietario().await;
    let formulario = "nombre=Otro&contrasena=otra-clave-larga&confirmacion=otra-clave-larga";
    let respuesta = contexto
        .pedir(post("/primer-arranque", None, formulario))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(texto(respuesta).await.contains("ya está configurado"));
}

#[tokio::test]
async fn el_propietario_entra_con_su_contrasena() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let formulario = format!("usuario_id={}&clave={CONTRASENA}", propietario.id);
    let respuesta = contexto.pedir(post("/entrar", None, &formulario)).await;
    assert_eq!(destino(&respuesta), Some("/"));
    let cookie = respuesta
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cookie.contains("HttpOnly"));
}

#[tokio::test]
async fn el_operario_entra_con_pin_y_va_a_su_dia() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let operario = contexto.con_operario(&propietario).await;
    let formulario = format!("usuario_id={operario}&clave={PIN}");
    let respuesta = contexto.pedir(post("/entrar", None, &formulario)).await;
    assert_eq!(destino(&respuesta), Some("/mi-dia"));
}

#[tokio::test]
async fn bloquea_tras_cinco_fallos_y_desbloquea_al_pasar_el_tiempo() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let operario = contexto.con_operario(&propietario).await;
    for _ in 0..4 {
        let resultado = entrar(&contexto.estado, solicitud(operario, "999999")).await;
        assert!(matches!(resultado, Ok(ResultadoDeEntrada::Incorrecta)));
    }
    let quinto = entrar(&contexto.estado, solicitud(operario, "999999")).await;
    assert!(matches!(
        quinto,
        Ok(ResultadoDeEntrada::Bloqueada { minutos: 1 })
    ));
    let con_pin_bueno = entrar(&contexto.estado, solicitud(operario, PIN)).await;
    assert!(matches!(
        con_pin_bueno,
        Ok(ResultadoDeEntrada::Bloqueada { .. })
    ));
    contexto.reloj.avanzar(Duration::minutes(2));
    let despues = entrar(&contexto.estado, solicitud(operario, PIN)).await;
    assert!(matches!(despues, Ok(ResultadoDeEntrada::Correcta { .. })));
}

#[tokio::test]
async fn un_operario_no_puede_entrar_con_algo_que_no_sea_un_pin() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let operario = contexto.con_operario(&propietario).await;
    let resultado = entrar(&contexto.estado, solicitud(operario, "12345")).await;
    assert!(matches!(resultado, Ok(ResultadoDeEntrada::Incorrecta)));
}

#[tokio::test]
async fn la_sesion_caduca_tras_dos_horas_sin_actividad() {
    let contexto = Contexto::nuevo().await;
    let (token, _) = contexto.con_propietario().await;
    contexto.reloj.avanzar(Duration::minutes(119));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_some()
    );
    contexto.reloj.avanzar(Duration::minutes(119));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_some()
    );
    contexto.reloj.avanzar(Duration::hours(2));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn la_sesion_caduca_a_las_doce_horas_aunque_haya_actividad() {
    let contexto = Contexto::nuevo().await;
    let (token, _) = contexto.con_propietario().await;
    for _ in 0..11 {
        contexto.reloj.avanzar(Duration::hours(1));
        assert!(
            identificar(&contexto.estado, &token)
                .await
                .unwrap()
                .is_some()
        );
    }
    contexto.reloj.avanzar(Duration::hours(1));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn salir_cierra_la_sesion() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let formulario = format!("_csrf={}", actor.token_csrf);
    let respuesta = contexto
        .pedir(post("/salir", Some(&token), &formulario))
        .await;
    assert_eq!(destino(&respuesta), Some("/entrar"));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn el_propietario_puede_mantener_la_sesion_hasta_cerrar_el_navegador() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let formulario = format!(
        "usuario_id={}&clave={CONTRASENA}&mantener_sesion=si",
        propietario.id
    );
    let respuesta = contexto.pedir(post("/entrar", None, &formulario)).await;
    let cookie = respuesta
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(!cookie.contains("Max-Age"), "{cookie}");
    let token = token_de_la_cookie(cookie);
    contexto.reloj.avanzar(Duration::hours(10));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_some()
    );
    contexto.reloj.avanzar(Duration::days(5));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_some()
    );
    contexto.reloj.avanzar(Duration::days(30));
    assert!(
        identificar(&contexto.estado, &token)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn por_defecto_la_sesion_caduca_y_la_cookie_tiene_duracion() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let formulario = format!("usuario_id={}&clave={CONTRASENA}", propietario.id);
    let respuesta = contexto.pedir(post("/entrar", None, &formulario)).await;
    let cookie = respuesta
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cookie.contains("Max-Age=43200"), "{cookie}");
}

#[tokio::test]
async fn el_operario_no_puede_mantener_la_sesion_abierta() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    let operario = contexto.con_operario(&propietario).await;
    let formulario = format!("usuario_id={operario}&clave={PIN}&mantener_sesion=si");
    let respuesta = contexto.pedir(post("/entrar", None, &formulario)).await;
    let cookie = respuesta
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cookie.contains("Max-Age"), "{cookie}");
}
