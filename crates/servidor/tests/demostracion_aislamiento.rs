#![allow(clippy::unwrap_used, clippy::panic)]

//! La demostración nunca se mezcla con los datos de verdad (ADR-0028).

mod comun;

use std::path::PathBuf;

use servidor::aplicacion::Actor;
use servidor::aplicacion::demostracion::{
    QuienEntra, asegurar_rellena, empezar_de_nuevo, entrar_sin_clave,
};
use servidor::aplicacion::sesion::identificar;
use servidor::arranque::preparar_estado;
use servidor::bd::copia::{comprobar_copia, exportar};
use servidor::config::Configuracion;
use servidor::errores::ErrorApp;
use servidor::estado::EstadoApp;

fn carpeta_temporal(nombre: &str) -> PathBuf {
    let carpeta = std::env::temp_dir().join(format!(
        "alumframe-{nombre}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&carpeta).unwrap();
    carpeta
}

fn configuracion(carpeta: &std::path::Path) -> Configuracion {
    Configuracion {
        carpeta_datos: carpeta.to_path_buf(),
        ..Configuracion::default()
    }
}

/// Arranca como el programa de verdad (archivo en disco) y rellena su demostración.
async fn arrancada(nombre: &str) -> (PathBuf, EstadoApp) {
    let carpeta = carpeta_temporal(nombre);
    let estado = preparar_estado(&configuracion(&carpeta)).await.unwrap();
    asegurar_rellena(estado.demostracion().unwrap())
        .await
        .unwrap();
    (carpeta, estado)
}

async fn dueno_de(demo: &EstadoApp) -> Actor {
    let (token, _) = entrar_sin_clave(demo, QuienEntra::Dueno).await.unwrap();
    identificar(demo, &token).await.unwrap().unwrap()
}

#[tokio::test]
async fn al_arrancar_la_demostracion_va_en_su_subcarpeta_y_no_toca_la_real() {
    let (carpeta, estado) = arrancada("demo-carpeta").await;
    assert!(carpeta.join("demostracion").join("alumframe.db").exists());
    assert!(!estado.es_demostracion());
    assert!(estado.demostracion().unwrap().es_demostracion());
    assert!(!estado.hay_usuarios(), "la de verdad sigue vacía");
}

#[tokio::test]
async fn una_base_real_marcada_como_demostracion_no_arranca() {
    let (carpeta, _) = arrancada("demo-cambiada").await;
    let real = rusqlite::Connection::open(carpeta.join("alumframe.db")).unwrap();
    servidor::bd::consultas::configuracion::marcar_como_demostracion(&real, "2026-11-21T08:00:00Z")
        .unwrap();
    drop(real);
    let resultado = preparar_estado(&configuracion(&carpeta)).await;
    let Err(error) = resultado else {
        panic!("arrancó con la base de la demostración");
    };
    assert!(error.to_string().contains("demostración"), "{error}");
}

#[tokio::test]
async fn una_copia_de_la_demostracion_no_se_importa_en_la_de_verdad() {
    let (carpeta, estado) = arrancada("demo-copia").await;
    let archivo = carpeta.join("demo.alumframe");
    let destino = archivo.clone();
    estado
        .demostracion()
        .unwrap()
        .bd
        .con_escritor_entero(move |con| exportar(con, &destino))
        .await
        .unwrap();
    let Err(ErrorApp::Validacion(motivo)) = comprobar_copia(&archivo) else {
        panic!("se aceptó una copia de la demostración");
    };
    assert!(motivo.contains("demostración"), "{motivo}");
}

#[tokio::test]
async fn empezar_de_nuevo_la_deja_como_al_principio_y_solo_lo_hace_el_dueno() {
    let (_, estado) = arrancada("demo-de-nuevo").await;
    let demo = estado.demostracion().unwrap();
    let dueno = dueno_de(demo).await;
    let borrar = |tx: &rusqlite::Transaction| Ok(tx.execute("DELETE FROM apunte", [])?);
    demo.bd.escribir(borrar).await.unwrap();
    let mut operario = dueno.clone();
    operario.rol = dominio::acceso::Rol::Operario;
    assert!(empezar_de_nuevo(demo, &operario).await.is_err());
    assert!(
        empezar_de_nuevo(&estado, &dueno).await.is_err(),
        "nunca sobre la de verdad"
    );
    empezar_de_nuevo(demo, &dueno).await.unwrap();
    let apuntes = demo
        .bd
        .leer(|con| Ok(con.query_row("SELECT count(*) FROM apunte", [], |f| f.get::<_, i64>(0))?))
        .await
        .unwrap();
    assert_eq!(apuntes, 2, "vuelve como al principio");
    assert!(demo.hay_usuarios());
}
