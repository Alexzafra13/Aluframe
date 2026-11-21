#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use comun::{Contexto, PIN, destino, get, post};
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};

async fn token_de_operario(contexto: &Contexto) -> String {
    let (_, propietario) = contexto.con_propietario().await;
    let operario = contexto.con_operario(&propietario).await;
    let solicitud = SolicitudDeEntrada {
        usuario_id: operario,
        clave: PIN.into(),
        ip: "10.0.0.5".into(),
    };
    match entrar(&contexto.estado, solicitud).await.unwrap() {
        ResultadoDeEntrada::Correcta { token, .. } => token,
        otro => panic!("no pudo entrar: {otro:?}"),
    }
}

#[tokio::test]
async fn sin_sesion_se_pide_entrar() {
    let contexto = Contexto::nuevo().await;
    contexto.con_propietario().await;
    let respuesta = contexto.pedir(get("/ajustes/usuarios", None)).await;
    assert_eq!(destino(&respuesta), Some("/entrar"));
}

#[tokio::test]
async fn el_operario_no_ve_las_pantallas_del_propietario() {
    let contexto = Contexto::nuevo().await;
    let token = token_de_operario(&contexto).await;
    for ruta in ["/ajustes/usuarios", "/clientes", "/facturas"] {
        let respuesta = contexto.pedir(get(ruta, Some(&token))).await;
        assert_eq!(respuesta.status(), StatusCode::NOT_FOUND, "{ruta}");
    }
    let inicio = contexto.pedir(get("/", Some(&token))).await;
    assert_eq!(destino(&inicio), Some("/mi-dia"));
    let mi_dia = contexto.pedir(get("/mi-dia", Some(&token))).await;
    assert_eq!(mi_dia.status(), StatusCode::OK);
}

#[tokio::test]
async fn el_propietario_ve_sus_pantallas() {
    let contexto = Contexto::nuevo().await;
    let (token, _) = contexto.con_propietario().await;
    for ruta in ["/", "/ajustes/usuarios", "/clientes", "/mi-dia"] {
        let respuesta = contexto.pedir(get(ruta, Some(&token))).await;
        assert_eq!(respuesta.status(), StatusCode::OK, "{ruta}");
    }
}

#[tokio::test]
async fn rechaza_un_cambio_sin_token_csrf() {
    let contexto = Contexto::nuevo().await;
    let (token, _) = contexto.con_propietario().await;
    let formulario = "nombre=Intruso&pin=111111";
    let respuesta = contexto
        .pedir(post("/ajustes/usuarios", Some(&token), formulario))
        .await;
    assert_eq!(respuesta.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn acepta_un_cambio_con_token_csrf_valido() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let formulario = format!("_csrf={}&nombre=Pedro&pin=222222", actor.token_csrf);
    let respuesta = contexto
        .pedir(post("/ajustes/usuarios", Some(&token), &formulario))
        .await;
    assert_eq!(destino(&respuesta), Some("/ajustes/usuarios?hecho=alta"));
}

#[tokio::test]
async fn rechaza_peticiones_de_otra_web() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let peticion = Request::post("/ajustes/usuarios")
        .header(header::COOKIE, format!("alumframe_sesion={token}"))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header("sec-fetch-site", "cross-site")
        .body(Body::from(format!(
            "_csrf={}&nombre=X&pin=333333",
            actor.token_csrf
        )))
        .unwrap();
    let respuesta = contexto.pedir(peticion).await;
    assert_eq!(respuesta.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn un_operario_desactivado_pierde_su_sesion() {
    let contexto = Contexto::nuevo().await;
    let token = token_de_operario(&contexto).await;
    let (token_dueno, actor) = contexto.con_propietario_existente().await;
    let formulario = format!("_csrf={}", actor.token_csrf);
    let ruta = "/ajustes/usuarios/2/desactivar";
    let respuesta = contexto
        .pedir(post(ruta, Some(&token_dueno), &formulario))
        .await;
    assert_eq!(respuesta.status(), StatusCode::SEE_OTHER);
    let despues = contexto.pedir(get("/mi-dia", Some(&token))).await;
    assert_eq!(destino(&despues), Some("/entrar"));
}
