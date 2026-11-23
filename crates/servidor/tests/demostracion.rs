#![allow(clippy::unwrap_used, clippy::panic)]

//! La demostración (ADR-0028): se rellena con todo el recorrido, va aparte de los datos de
//! verdad y se entra y se sale sin perder la sesión real.

mod comun;

use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use comun::{Contexto, destino, formulario, texto};
use servidor::aplicacion::demostracion::{QuienEntra, asegurar_rellena, entrar_sin_clave};
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

/// El dueño real pulsa «Probar la demostración» y entra directamente como «Dueño (demo)», sin
/// contraseña. Devuelve el estado, su token real y las cookies que lleva el navegador dentro.
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
    assert_eq!(destino(&entrar), Some("/"), "{}", entrar.status());
    let puestas = cookies_puestas(&entrar);
    let volver = format!("alumframe_volver={real}");
    assert!(
        puestas
            .iter()
            .any(|c| c.starts_with("alumframe_demostracion=si"))
    );
    assert!(puestas.iter().any(|c| c.starts_with(&volver)));
    let sesion = puestas
        .iter()
        .find(|c| c.starts_with("alumframe_sesion="))
        .unwrap();
    let token_demo = comun::token_de_la_cookie(sesion);
    assert_ne!(token_demo, real, "la sesión de dentro es otra");
    let todo = format!("alumframe_demostracion=si; {volver}; alumframe_sesion={token_demo}");
    (estado, real, todo)
}

/// Entra en la demostración como el usuario elegido, sin clave. Devuelve la respuesta.
async fn entrar_como(estado: &EstadoApp, cookies: &str, usuario: i64) -> Response<Body> {
    let cuerpo = formulario(&[("usuario_id", &usuario.to_string())]);
    let peticion = con_cookies("POST", "/demostracion/entrar-como", cookies, &cuerpo);
    pedir(estado, peticion).await
}

#[tokio::test]
async fn dentro_se_elige_usuario_sin_contrasena_ni_pin() {
    let (contexto, demo, _) = con_demostracion().await;
    let juan = "SELECT id FROM usuario WHERE rol = 'operario'";
    let juan = contar(&demo, juan).await;
    let estado = contexto.estado.clone().con_demostracion(demo);
    let en_demo = "alumframe_demostracion=si";
    for ruta in ["/entrar", "/entrar?usuario=1"] {
        let entrada = texto(pedir(&estado, con_cookies("GET", ruta, en_demo, "")).await).await;
        assert!(entrada.contains("Juan (demo)") && entrada.contains("/demostracion/entrar-como"));
        assert!(
            !entrada.contains("type=\"password\""),
            "{ruta} no pide clave"
        );
    }
    let respuesta = entrar_como(&estado, en_demo, juan).await;
    assert_eq!(destino(&respuesta), Some("/mi-dia"));
    let token = comun::token_de_la_cookie(&cookies_puestas(&respuesta)[0]);
    let dentro = format!("{en_demo}; alumframe_sesion={token}");
    let mi_dia = texto(pedir(&estado, con_cookies("GET", "/mi-dia", &dentro, "")).await).await;
    assert!(mi_dia.contains("Juan (demo)") && mi_dia.contains("Cambiar de usuario"));
}

#[tokio::test]
async fn en_los_datos_de_verdad_nunca_se_entra_sin_clave() {
    let (contexto, demo, _) = con_demostracion().await;
    let fuera = QuienEntra::Dueno;
    assert!(entrar_sin_clave(&contexto.estado, fuera).await.is_err());
    let estado = contexto.estado.clone().con_demostracion(demo);
    let respuesta = entrar_como(&estado, "", 1).await;
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
    assert!(cookies_puestas(&respuesta).is_empty(), "sin sesión");
    let entrada =
        texto(pedir(&estado, con_cookies("GET", "/entrar?usuario=1", "", "")).await).await;
    assert!(
        entrada.contains("type=\"password\""),
        "la de verdad pide contraseña"
    );
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
    let dentro = entrar_como(&estado, en_demo, 1).await;
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
