//! Pantalla de recuperación del escritorio: la base de este equipo no se puede usar.

use crate::comun;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use comun::texto;
use servidor::bd::ErrorEsquema;
use servidor::escritorio::recuperacion::Recuperacion;
use servidor::http::crear_router_de_recuperacion;
use tower::ServiceExt;

const TOKEN: &str = "abc123";

fn carpeta_con_base(nombre: &str) -> std::path::PathBuf {
    let carpeta = std::env::temp_dir().join(format!("alumframe-{nombre}-{}", std::process::id()));
    std::fs::create_dir_all(&carpeta).unwrap();
    std::fs::write(carpeta.join("alumframe.db"), b"antigua").unwrap();
    carpeta
}

fn pulsar(ruta: &str, token: &str) -> Request<Body> {
    Request::post(ruta)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("token={token}")))
        .unwrap()
}

#[tokio::test]
async fn con_una_base_antigua_dice_la_carpeta_y_la_aparta_sin_borrarla() {
    let carpeta = carpeta_con_base("recuperar-antigua");
    let recuperacion = Recuperacion::desde(&ErrorEsquema::Desconocida, &carpeta);
    let router = crear_router_de_recuperacion(recuperacion, (TOKEN.into(), "marca".into()));
    let inicio = router
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await;
    let pagina = texto(inicio.unwrap()).await;
    assert!(pagina.contains(&carpeta.display().to_string()));
    assert!(pagina.contains("Abrir la carpeta de datos") && pagina.contains("Empezar de cero"));
    let sin_token = router
        .clone()
        .oneshot(pulsar("/empezar-de-cero", "otro"))
        .await;
    assert_eq!(sin_token.unwrap().status(), StatusCode::FORBIDDEN);
    assert!(
        carpeta.join("alumframe.db").exists(),
        "sin el token no se toca"
    );
    let listo = router
        .oneshot(pulsar("/empezar-de-cero", TOKEN))
        .await
        .unwrap();
    assert!(texto(listo).await.contains("vuelve a abrir Aluframe"));
    assert!(!carpeta.join("alumframe.db").exists());
    assert!(
        carpeta.join("apartadas/alumframe-marca.db").exists(),
        "apartada, no borrada"
    );
    let _ = std::fs::remove_dir_all(&carpeta);
}

#[tokio::test]
async fn con_una_base_mas_nueva_no_ofrece_empezar_de_cero() {
    let carpeta = carpeta_con_base("recuperar-nueva");
    let nueva = ErrorEsquema::MasNueva {
        base: 9,
        programa: 4,
    };
    let router = crear_router_de_recuperacion(
        Recuperacion::desde(&nueva, &carpeta),
        (TOKEN.into(), "marca".into()),
    );
    let inicio = router
        .clone()
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await;
    let pagina = texto(inicio.unwrap()).await;
    assert!(pagina.contains("más reciente") && !pagina.contains("Empezar de cero"));
    let forzado = router
        .oneshot(pulsar("/empezar-de-cero", TOKEN))
        .await
        .unwrap();
    assert_eq!(forzado.status(), StatusCode::FORBIDDEN);
    assert!(carpeta.join("alumframe.db").exists());
    let _ = std::fs::remove_dir_all(&carpeta);
}
