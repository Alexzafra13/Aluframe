use crate::comun;

use std::path::PathBuf;
use std::sync::Arc;

use comun::{Contexto, PIN};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::Actor;
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::datos::{
    confirmar_importacion, exportar_datos, previsualizar_importacion, recibir_importacion,
};
use servidor::aplicacion::primer_arranque::{DatosDelPropietario, crear_propietario_inicial};
use servidor::aplicacion::sesion::identificar;
use servidor::bd::{BaseDeDatos, preparar_esquema};
use servidor::errores::ErrorApp;
use servidor::estado::EstadoApp;
use servidor::reloj::RelojFijo;
use time::macros::datetime;

fn carpeta_temporal() -> PathBuf {
    let mut aleatorio = [0u8; 8];
    getrandom::fill(&mut aleatorio).unwrap();
    let sufijo: String = aleatorio.iter().map(|b| format!("{b:02x}")).collect();
    let carpeta = std::env::temp_dir().join(format!("alumframe-prueba-{sufijo}"));
    std::fs::create_dir_all(&carpeta).unwrap();
    carpeta
}

async fn con_copias() -> (Contexto, Actor, PathBuf) {
    let mut contexto = Contexto::nuevo().await;
    let carpeta = carpeta_temporal();
    contexto.estado = contexto.estado.clone().con_carpeta_copias(carpeta.clone());
    let (_, actor) = contexto.con_propietario().await;
    (contexto, actor, carpeta)
}

async fn nuevo_cliente(estado: &EstadoApp, actor: &Actor, nombre: &str) {
    let datos = validar_cliente(&EntradaDeCliente {
        nombre: nombre.into(),
        ..EntradaDeCliente::default()
    })
    .unwrap();
    crear_cliente(estado, actor, datos, vec![]).await.unwrap();
}

async fn clientes(estado: &EstadoApp) -> i64 {
    estado
        .bd
        .leer(|con| Ok(con.query_row("SELECT count(*) FROM cliente", [], |f| f.get(0))?))
        .await
        .unwrap()
}

#[tokio::test]
async fn importar_una_copia_devuelve_los_datos_al_momento_de_exportar() {
    let (contexto, actor, carpeta) = con_copias().await;
    nuevo_cliente(&contexto.estado, &actor, "Ana Ruiz").await;
    let copia = exportar_datos(&contexto.estado, &actor).await.unwrap();
    assert_eq!(copia.nombre, "alumframe-2026-11-21.alumframe");
    nuevo_cliente(&contexto.estado, &actor, "Luis Gil").await;
    assert_eq!(clientes(&contexto.estado).await, 2);

    let clave = recibir_importacion(&contexto.estado, &actor, copia.bytes)
        .await
        .unwrap();
    let vista = previsualizar_importacion(&contexto.estado, &actor, &clave)
        .await
        .unwrap();
    assert_eq!((vista.actual.clientes, vista.archivo.clientes), (2, 1));
    let previa = confirmar_importacion(&contexto.estado, &actor, &clave)
        .await
        .unwrap();

    assert_eq!(clientes(&contexto.estado).await, 1);
    assert!(carpeta.join(previa).is_file(), "copia de lo anterior");
    let pendientes = std::fs::read_dir(&carpeta)
        .unwrap()
        .filter(|e| {
            let nombre = e.as_ref().unwrap().file_name();
            nombre.to_string_lossy().starts_with("importacion-")
        })
        .count();
    assert_eq!(pendientes, 0);
}

#[tokio::test]
async fn rechaza_un_archivo_que_no_es_una_copia_y_no_guarda_nada() {
    let (contexto, actor, carpeta) = con_copias().await;
    let error = recibir_importacion(&contexto.estado, &actor, b"no soy una base".to_vec())
        .await
        .unwrap_err();
    assert!(matches!(error, ErrorApp::Validacion(m) if m.contains("no es una copia")));
    assert_eq!(std::fs::read_dir(&carpeta).unwrap().count(), 0);
}

#[tokio::test]
async fn rechaza_una_copia_de_otra_version_del_esquema() {
    let (contexto, actor, carpeta) = con_copias().await;
    let ajena = carpeta.join("ajena.db");
    let con = rusqlite::Connection::open(&ajena).unwrap();
    con.execute_batch(
        "CREATE TABLE esquema (huella TEXT, creado_en TEXT);
         INSERT INTO esquema VALUES ('otra', '2026-01-01T00:00:00Z');",
    )
    .unwrap();
    drop(con);
    let bytes = std::fs::read(&ajena).unwrap();
    let error = recibir_importacion(&contexto.estado, &actor, bytes)
        .await
        .unwrap_err();
    assert!(matches!(error, ErrorApp::Validacion(m) if m.contains("otra versión")));
}

#[tokio::test]
async fn rechaza_una_copia_de_una_version_mas_nueva() {
    let (contexto, actor, carpeta) = con_copias().await;
    let copia = exportar_datos(&contexto.estado, &actor).await.unwrap();
    let ruta = carpeta.join("nueva.db");
    std::fs::write(&ruta, copia.bytes).unwrap();
    let con = rusqlite::Connection::open(&ruta).unwrap();
    con.pragma_update(None, "user_version", 99).unwrap();
    drop(con);
    let bytes = std::fs::read(&ruta).unwrap();
    let error = recibir_importacion(&contexto.estado, &actor, bytes)
        .await
        .unwrap_err();
    assert!(matches!(error, ErrorApp::Validacion(m) if m.contains("más nueva")));
}

#[tokio::test]
async fn una_clave_inventada_no_encuentra_nada() {
    let (contexto, actor, _) = con_copias().await;
    for clave in ["../../etc/passwd", &"a".repeat(64)] {
        let error = previsualizar_importacion(&contexto.estado, &actor, clave)
            .await
            .err()
            .unwrap();
        assert!(matches!(error, ErrorApp::NoEncontrado));
    }
}

#[tokio::test]
async fn el_operario_no_puede_exportar_ni_importar() {
    let (contexto, propietario, _) = con_copias().await;
    contexto.con_operario(&propietario).await;
    let operario = servidor::aplicacion::entrar::entrar(
        &contexto.estado,
        servidor::aplicacion::entrar::SolicitudDeEntrada {
            usuario_id: 2,
            clave: PIN.into(),
            ip: "127.0.0.1".into(),
            duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
        },
    )
    .await
    .unwrap();
    let servidor::aplicacion::entrar::ResultadoDeEntrada::Correcta { token, .. } = operario else {
        panic!("el operario no pudo entrar");
    };
    let actor = identificar(&contexto.estado, &token)
        .await
        .unwrap()
        .unwrap();
    assert!(exportar_datos(&contexto.estado, &actor).await.is_err());
    assert!(
        recibir_importacion(&contexto.estado, &actor, vec![])
            .await
            .is_err()
    );
}

/// Como en la instalación real: base en archivo, modo WAL y lectores abiertos.
#[tokio::test]
async fn restaura_sobre_una_base_en_archivo_con_lectores_abiertos() {
    let carpeta = carpeta_temporal();
    let ruta = carpeta.join("alumframe.db");
    let mut con = rusqlite::Connection::open(&ruta).unwrap();
    preparar_esquema(&mut con, "2026-11-21T08:00:00Z").unwrap();
    drop(con);
    let reloj = Arc::new(RelojFijo::new(datetime!(2026-11-21 08:00 UTC)));
    let estado = EstadoApp::nuevo(BaseDeDatos::abrir(&ruta).unwrap(), reloj)
        .await
        .unwrap()
        .con_carpeta_copias(carpeta.join("copias"));
    let datos = DatosDelPropietario {
        nombre: "Manuel".into(),
        contrasena: comun::CONTRASENA.into(),
        confirmacion: comun::CONTRASENA.into(),
    };
    let token = crear_propietario_inicial(&estado, datos).await.unwrap();
    let actor = identificar(&estado, &token).await.unwrap().unwrap();
    let copia = exportar_datos(&estado, &actor).await.unwrap();
    nuevo_cliente(&estado, &actor, "Ana Ruiz").await;
    assert_eq!(clientes(&estado).await, 1);

    let clave = recibir_importacion(&estado, &actor, copia.bytes)
        .await
        .unwrap();
    confirmar_importacion(&estado, &actor, &clave)
        .await
        .unwrap();

    assert_eq!(
        clientes(&estado).await,
        0,
        "los lectores ven los datos nuevos"
    );
    nuevo_cliente(&estado, &actor, "Luis Gil").await;
    assert_eq!(clientes(&estado).await, 1, "se sigue escribiendo");
}
