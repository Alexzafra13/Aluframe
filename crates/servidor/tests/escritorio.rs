#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::panic)]

//! La aplicación de escritorio con un «navegador» falso: un guion que descarga la página que se
//! le pide y termina, como quien abre la ventana y la cierra (ADR-0015).

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const NAVEGADOR_FALSO: &str = r#"#!/bin/sh
for a in "$@"; do case "$a" in --app=*) url="${a#--app=}";; esac; done
echo "$url" > "$CARPETA/url-abierta"
case "$url" in http://127.0.0.1*) curl -sL "$url" > "$CARPETA/pagina.html";; esac
"#;

fn preparar(nombre: &str, puerto: u16) -> (PathBuf, Command) {
    let carpeta = std::env::temp_dir().join(format!("alumframe-escritorio-{nombre}-{puerto}"));
    let _ = std::fs::remove_dir_all(&carpeta);
    std::fs::create_dir_all(&carpeta).unwrap();
    let navegador = carpeta.join("navegador.sh");
    std::fs::write(&navegador, NAVEGADOR_FALSO).unwrap();
    std::fs::set_permissions(&navegador, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut orden = Command::new(env!("CARGO_BIN_EXE_alumframe-escritorio"));
    orden
        .env("ALUMFRAME_CARPETA_APP", &carpeta)
        .env("ALUMFRAME_NAVEGADOR", &navegador)
        .env("ALUMFRAME_PUERTO", puerto.to_string())
        .env("ALUMFRAME_SILENCIO_PARA_CERRAR", "2")
        .env("CARPETA", &carpeta);
    (carpeta, orden)
}

/// Al cerrarse la ventana el programa también termina: no queda nada en marcha.
fn ejecutar_hasta_que_termine(mut orden: Command) {
    let mut proceso = orden.spawn().unwrap();
    let limite = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(estado) = proceso.try_wait().unwrap() {
            assert!(estado.success(), "terminó con {estado}");
            return;
        }
        if Instant::now() > limite {
            let _ = proceso.kill();
            panic!("la aplicación no se cerró al cerrar la ventana");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn leer(carpeta: &Path, archivo: &str) -> String {
    std::fs::read_to_string(carpeta.join(archivo)).unwrap_or_default()
}

#[test]
fn la_primera_vez_pregunta_donde_trabajar_y_se_cierra_con_la_ventana() {
    let (carpeta, orden) = preparar("primera", 19301);
    ejecutar_hasta_que_termine(orden);
    assert!(leer(&carpeta, "pagina.html").contains("¿Dónde vas a trabajar?"));
    assert!(carpeta.join("datos/alumframe.db").is_file());
}

#[test]
fn en_este_equipo_abre_directamente_el_programa() {
    let (carpeta, orden) = preparar("local", 19302);
    std::fs::write(
        carpeta.join("escritorio.toml"),
        "conexion = \"este_equipo\"\n",
    )
    .unwrap();
    ejecutar_hasta_que_termine(orden);
    assert!(leer(&carpeta, "url-abierta").ends_with(":19302/\n"));
    assert!(leer(&carpeta, "pagina.html").contains("Crea la cuenta del propietario"));
}

#[test]
fn contra_el_servidor_solo_abre_su_direccion_sin_datos_locales() {
    let (carpeta, orden) = preparar("servidor", 19303);
    let preferencias = "conexion = \"servidor\"\nservidor = \"http://192.168.1.100:9090\"\n";
    std::fs::write(carpeta.join("escritorio.toml"), preferencias).unwrap();
    ejecutar_hasta_que_termine(orden);
    assert_eq!(
        leer(&carpeta, "url-abierta"),
        "http://192.168.1.100:9090/\n"
    );
    assert!(!carpeta.join("datos").exists());
}

#[test]
fn cambiar_conexion_vuelve_a_preguntar_aunque_ya_se_eligiera() {
    let (carpeta, mut orden) = preparar("elegir", 19304);
    let preferencias = "conexion = \"servidor\"\nservidor = \"http://192.168.1.100:9090\"\n";
    std::fs::write(carpeta.join("escritorio.toml"), preferencias).unwrap();
    orden.arg("--elegir");
    ejecutar_hasta_que_termine(orden);
    let pagina = leer(&carpeta, "pagina.html");
    assert!(pagina.contains("¿Dónde vas a trabajar?"));
    assert!(pagina.contains("value=\"http://192.168.1.100:9090\""));
}

/// Edge con el perfil ya abierto: el proceso lanzado pasa la ventana al que ya había y termina
/// al momento, pero la ventana sigue abierta y late. El programa no debe apagarse mientras lata.
const NAVEGADOR_QUE_PASA_LA_VENTANA: &str = r#"#!/bin/sh
for a in "$@"; do case "$a" in --app=*) url="${a#--app=}";; esac; done
base="${url%/*}"
( for i in 1 2 3 4 5 6; do sleep 1; curl -s "$base/escritorio/latido" >/dev/null && echo "$i" >> "$CARPETA/latidos"; done ) &
"#;

#[test]
fn sigue_en_marcha_mientras_la_ventana_late_aunque_el_navegador_termine() {
    let (carpeta, orden) = preparar("traspaso", 19305);
    std::fs::write(carpeta.join("navegador.sh"), NAVEGADOR_QUE_PASA_LA_VENTANA).unwrap();
    let preferencias = "conexion = \"este_equipo\"\n";
    std::fs::write(carpeta.join("escritorio.toml"), preferencias).unwrap();
    ejecutar_hasta_que_termine(orden);
    let atendidos = leer(&carpeta, "latidos").lines().count();
    assert_eq!(
        atendidos, 6,
        "el programa se apagó con la ventana aún abierta"
    );
}
