use crate::comun;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use comun::{Contexto, destino, formulario, get, texto};
use servidor::escritorio::preferencias::{Conexion, Preferencias};

fn elegir(cuerpo: &str) -> Request<Body> {
    Request::post("/escritorio/conexion")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(cuerpo.to_owned()))
        .unwrap()
}

async fn de_escritorio(nombre: &str) -> (Contexto, std::path::PathBuf) {
    let mut contexto = Contexto::nuevo().await;
    let archivo = std::env::temp_dir().join(format!(
        "alumframe-pref-{nombre}-{}.toml",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&archivo);
    contexto.estado = contexto
        .estado
        .clone()
        .con_preferencias_escritorio(archivo.clone());
    (contexto, archivo)
}

#[tokio::test]
async fn en_el_servidor_del_taller_no_existe_la_pantalla_de_conexion() {
    let contexto = Contexto::nuevo().await;
    contexto.con_propietario().await;
    let respuesta = contexto.pedir(get("/escritorio/conexion", None)).await;
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn elegir_este_equipo_lo_recuerda_y_abre_el_programa() {
    let (contexto, archivo) = de_escritorio("local").await;
    let pagina = texto(contexto.pedir(get("/escritorio/conexion", None)).await).await;
    assert!(
        pagina.contains("¿Dónde vas a trabajar?"),
        "sin usuarios también"
    );
    let cuerpo = formulario(&[("conexion", "este_equipo")]);
    let respuesta = contexto.pedir(elegir(&cuerpo)).await;
    assert_eq!(destino(&respuesta), Some("/"));
    assert_eq!(
        Preferencias::cargar(&archivo).conexion,
        Some(Conexion::EsteEquipo)
    );
}

#[tokio::test]
async fn elegir_el_servidor_completa_la_direccion_y_pasa_a_ella() {
    let (contexto, archivo) = de_escritorio("servidor").await;
    let cuerpo = formulario(&[("conexion", "servidor"), ("servidor", "192.168.1.100")]);
    let pagina = texto(contexto.pedir(elegir(&cuerpo)).await).await;
    assert!(pagina.contains("url=http://192.168.1.100:9090/"));
    assert_eq!(
        Preferencias::cargar(&archivo).servidor_elegido(),
        Some("http://192.168.1.100:9090")
    );
}

#[tokio::test]
async fn una_direccion_mala_avisa_sin_perder_lo_escrito_ni_guardar_nada() {
    let (contexto, archivo) = de_escritorio("mala").await;
    let cuerpo = formulario(&[("conexion", "servidor"), ("servidor", "ftp://taller")]);
    let respuesta = contexto.pedir(elegir(&cuerpo)).await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let pagina = texto(respuesta).await;
    assert!(pagina.contains("http:// o https://"));
    assert!(pagina.contains("value=\"ftp://taller\""));
    assert!(!archivo.exists());
}
