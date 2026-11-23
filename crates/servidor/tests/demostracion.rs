#![allow(clippy::unwrap_used, clippy::panic)]

//! La demostración (ADR-0028): se rellena con todo el recorrido, va aparte de los datos de
//! verdad y se entra y se sale sin perder la sesión real.

mod comun;

use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use comun::{Contexto, formulario, texto};
use servidor::aplicacion::demostracion::{CONTRASENA, asegurar_rellena};
use servidor::bd::{BaseDeDatos, preparar_esquema};
use servidor::estado::EstadoApp;
use servidor::http::crear_router;
use tower::ServiceExt;

/// La base de verdad (vacía salvo el dueño) con su demostración aparte, ya rellena.
async fn con_demostracion() -> (Contexto, EstadoApp, (String, String)) {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let bd = BaseDeDatos::en_memoria().unwrap();
    bd.con_escritor(|con| preparar_esquema(con, "2026-11-21T08:00:00Z").unwrap())
        .unwrap();
    let demo = EstadoApp::nuevo(bd, contexto.reloj.clone())
        .await
        .unwrap()
        .como_demostracion();
    servidor::aplicacion::series_incluidas::cargar_series_incluidas(&demo)
        .await
        .unwrap();
    servidor::aplicacion::acabados_incluidos::cargar_acabados_incluidos(&demo)
        .await
        .unwrap();
    asegurar_rellena(&demo).await.unwrap();
    (contexto, demo, (token, actor.token_csrf))
}

async fn contar(estado: &EstadoApp, sql: &'static str) -> i64 {
    estado
        .bd
        .leer(move |con| Ok(con.query_row(sql, [], |f| f.get(0))?))
        .await
        .unwrap()
}

async fn pedir(estado: &EstadoApp, peticion: Request<Body>) -> Response<Body> {
    crear_router(estado.clone())
        .oneshot(peticion)
        .await
        .unwrap()
}

fn con_cookies(metodo: &str, ruta: &str, cookies: &str, cuerpo: &str) -> Request<Body> {
    Request::builder()
        .method(metodo)
        .uri(ruta)
        .header(header::COOKIE, cookies)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(cuerpo.to_owned()))
        .unwrap()
}

fn cookies_puestas(respuesta: &Response<Body>) -> Vec<String> {
    respuesta
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn se_rellena_con_todo_el_recorrido_y_no_toca_la_base_real() {
    let (contexto, demo, _) = con_demostracion().await;
    assert_eq!(contar(&demo, "SELECT count(*) FROM cliente").await, 12);
    assert_eq!(contar(&demo, "SELECT count(*) FROM usuario").await, 2);
    assert_eq!(contar(&demo, "SELECT count(*) FROM trabajo").await, 4);
    let estados = "SELECT count(DISTINCT estado) FROM presupuesto";
    assert_eq!(
        contar(&demo, estados).await,
        4,
        "borrador, enviado, aceptado y rechazado"
    );
    let emitidas = "SELECT count(*) FROM factura WHERE estado = 'emitida'";
    assert_eq!(contar(&demo, emitidas).await, 5);
    let terminadas = "SELECT count(*) FROM orden_taller WHERE estado = 'terminada'";
    assert_eq!(contar(&demo, terminadas).await, 2);
    assert_eq!(contar(&demo, "SELECT count(*) FROM apunte").await, 2);
    let marca = "SELECT count(*) FROM configuracion WHERE clave = 'demostracion' AND valor = 'si'";
    assert_eq!(contar(&demo, marca).await, 1);
    assert_eq!(
        contar(&contexto.estado, "SELECT count(*) FROM cliente").await,
        0
    );
    assert_eq!(contar(&contexto.estado, marca).await, 0);
}

/// El dueño real entra en la demostración y luego como «Dueño (demo)». Devuelve el estado, su
/// token real y las cookies que lleva el navegador dentro de la demostración.
async fn dentro_de_la_demostracion() -> (EstadoApp, String, String) {
    let (contexto, demo, (real, csrf_real)) = con_demostracion().await;
    let estado = contexto.estado.clone().con_demostracion(demo);
    let sesion_real = format!("alumframe_sesion={real}");
    let cuerpo = formulario(&[("_csrf", &csrf_real)]);
    let entrar = pedir(
        &estado,
        con_cookies("POST", "/demostracion/entrar", &sesion_real, &cuerpo),
    )
    .await;
    assert!(entrar.status().is_redirection(), "{}", entrar.status());
    let puestas = cookies_puestas(&entrar);
    let volver = format!("alumframe_volver={real}");
    assert!(
        puestas
            .iter()
            .any(|c| c.starts_with("alumframe_demostracion=si"))
    );
    assert!(puestas.iter().any(|c| c.starts_with(&volver)));
    let en_demo = format!("alumframe_demostracion=si; {volver}");
    let entrada = texto(pedir(&estado, con_cookies("GET", "/entrar", &en_demo, "")).await).await;
    assert!(entrada.contains("Dueño (demo)") && entrada.contains(CONTRASENA));
    let datos = formulario(&[("usuario_id", "1"), ("clave", CONTRASENA)]);
    let dentro = pedir(&estado, con_cookies("POST", "/entrar", &en_demo, &datos)).await;
    let token_demo = comun::token_de_la_cookie(&cookies_puestas(&dentro)[0]);
    (
        estado,
        real,
        format!("{en_demo}; alumframe_sesion={token_demo}"),
    )
}

#[tokio::test]
async fn dentro_se_ven_los_datos_inventados_y_fuera_los_de_verdad() {
    let (estado, real, todo_demo) = dentro_de_la_demostracion().await;
    let clientes = con_cookies("GET", "/clientes", &todo_demo, "");
    let clientes = texto(pedir(&estado, clientes).await).await;
    assert!(clientes.contains("Lucía Demo Pérez") && clientes.contains("Salir de la demostración"));
    let reales = con_cookies("GET", "/clientes", &format!("alumframe_sesion={real}"), "");
    let reales = texto(pedir(&estado, reales).await).await;
    assert!(!reales.contains("Lucía Demo") && !reales.contains("Salir de la demostración"));
}

#[tokio::test]
async fn al_salir_se_recupera_la_sesion_de_verdad() {
    let (estado, real, todo_demo) = dentro_de_la_demostracion().await;
    let sin_token = con_cookies("POST", "/demostracion/salir", &todo_demo, "");
    let sin_token = pedir(&estado, sin_token).await;
    assert_eq!(
        sin_token.status(),
        StatusCode::FORBIDDEN,
        "con sesión pide el token CSRF"
    );
    let token_demo = comun::token_de_la_cookie(todo_demo.rsplit("; ").next().unwrap());
    let demo = estado.demostracion().unwrap();
    let actor = servidor::aplicacion::sesion::identificar(demo, &token_demo)
        .await
        .unwrap()
        .unwrap();
    let cuerpo = formulario(&[("_csrf", &actor.token_csrf)]);
    let salir = con_cookies("POST", "/demostracion/salir", &todo_demo, &cuerpo);
    let puestas = cookies_puestas(&pedir(&estado, salir).await);
    assert!(puestas.contains(&format!(
        "alumframe_sesion={real}; HttpOnly; SameSite=Lax; Path=/; Max-Age=43200"
    )));
    assert!(
        puestas
            .iter()
            .any(|c| c.starts_with("alumframe_demostracion=;"))
    );
}

#[tokio::test]
async fn en_la_demostracion_no_hay_copias_ni_exportar_datos() {
    let (contexto, demo, _) = con_demostracion().await;
    let estado = contexto.estado.clone().con_demostracion(demo.clone());
    let en_demo = "alumframe_demostracion=si";
    let datos = formulario(&[("usuario_id", "1"), ("clave", CONTRASENA)]);
    let dentro = pedir(&estado, con_cookies("POST", "/entrar", en_demo, &datos)).await;
    let token_demo = comun::token_de_la_cookie(&cookies_puestas(&dentro)[0]);
    let todo = format!("{en_demo}; alumframe_sesion={token_demo}");
    for ruta in [
        "/ajustes/copias",
        "/ajustes/datos",
        "/ajustes/datos/exportar",
    ] {
        let respuesta = pedir(&estado, con_cookies("GET", ruta, &todo, "")).await;
        assert_eq!(respuesta.status(), StatusCode::NOT_FOUND, "{ruta}");
    }
    let taller =
        texto(pedir(&estado, con_cookies("GET", "/ajustes/taller", &todo, "")).await).await;
    assert!(
        !taller.contains("/ajustes/copias"),
        "sin la pestaña de copias"
    );
}

/// Fallo corregido: al pasar a la demostración, la petición arrastraba los parámetros de ruta
/// del enrutador de verdad y toda dirección con parámetros (estilos, fichas) daba error 500.
#[tokio::test]
async fn dentro_funcionan_las_direcciones_con_parametros() {
    let (estado, _, todo_demo) = dentro_de_la_demostracion().await;
    for ruta in [
        "/recursos/estilos.css",
        "/presupuestos/1",
        "/clientes/1",
        "/trabajos/1",
        "/facturas/1",
    ] {
        let respuesta = pedir(&estado, con_cookies("GET", ruta, &todo_demo, "")).await;
        assert_eq!(respuesta.status(), StatusCode::OK, "{ruta}");
    }
}
