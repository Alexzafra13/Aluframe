#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use std::path::PathBuf;
use std::sync::Arc;

use comun::{CONTRASENA, Contexto};
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};
use servidor::aplicacion::restablecer_contrasena::restablecer_contrasena;
use servidor::bd::{BaseDeDatos, migrar};
use servidor::estado::EstadoApp;
use servidor::reloj::RelojDelSistema;

fn carpeta_temporal(nombre: &str) -> PathBuf {
    let carpeta = std::env::temp_dir().join(format!("alumframe-{nombre}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&carpeta);
    std::fs::create_dir_all(&carpeta).unwrap();
    carpeta
}

#[tokio::test]
async fn la_base_en_archivo_usa_wal_y_los_lectores_ven_lo_escrito() {
    let ruta = carpeta_temporal("wal").join("alumframe.db");
    let mut conexion = rusqlite::Connection::open(&ruta).unwrap();
    migrar(&mut conexion, None, "2026-11-21T08:00:00Z").unwrap();
    drop(conexion);
    let bd = BaseDeDatos::abrir(&ruta).unwrap();
    let modo: String = bd
        .leer(|con| Ok(con.query_row("PRAGMA journal_mode", [], |f| f.get(0))?))
        .await
        .unwrap();
    assert_eq!(modo, "wal");
    let estado = EstadoApp::nuevo(bd, Arc::new(RelojDelSistema))
        .await
        .unwrap();
    assert!(!estado.hay_usuarios());
}

#[tokio::test]
async fn restablecer_la_contrasena_permite_entrar_con_la_nueva() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
    restablecer_contrasena(
        &contexto.estado,
        "Manuel".into(),
        "nueva-clave-larga".into(),
    )
    .await
    .unwrap();
    let con_vieja = SolicitudDeEntrada {
        usuario_id: propietario.id,
        clave: CONTRASENA.into(),
        ip: "127.0.0.1".into(),
        duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
    };
    let resultado = entrar(&contexto.estado, con_vieja).await.unwrap();
    assert!(matches!(resultado, ResultadoDeEntrada::Incorrecta));
    let con_nueva = SolicitudDeEntrada {
        usuario_id: propietario.id,
        clave: "nueva-clave-larga".into(),
        ip: "127.0.0.1".into(),
        duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
    };
    let resultado = entrar(&contexto.estado, con_nueva).await.unwrap();
    assert!(matches!(resultado, ResultadoDeEntrada::Correcta { .. }));
}
