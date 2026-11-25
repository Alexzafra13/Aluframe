#![forbid(unsafe_code)]
// Sin consola en Windows: es una aplicación con ventana (ADR-0015).
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Context;
use servidor::arranque::{ErrorArranque, escuchar, preparar_estado, servir_hasta};
use servidor::autenticacion::generar_token;
use servidor::config::{Configuracion, Modo};
use servidor::escritorio::latido::la_ventana_sigue_abierta;
use servidor::escritorio::preferencias::{ARCHIVO_DE_PREFERENCIAS, Preferencias};
use servidor::escritorio::recuperacion::Recuperacion;
use servidor::escritorio::ventana::{abrir_ventana, mostrar_error};
use servidor::escritorio::{carpeta_de_la_aplicacion, responde_alumframe};
use servidor::fecha::formatear_utc;
use servidor::http::crear_router_de_recuperacion;

const ARCHIVO_DE_REGISTRO: &str = "alumframe.log";
/// Con más, el registro se empieza de nuevo al abrir: no crece sin límite en el PC.
const TAMANO_MAXIMO_DEL_REGISTRO: u64 = 5 * 1024 * 1024;

fn main() {
    let carpeta = carpeta_de_la_aplicacion();
    let _ = std::fs::create_dir_all(&carpeta);
    iniciar_registro(&carpeta);
    let elegir = std::env::args().any(|a| a == "--elegir");
    let resultado = tokio::runtime::Runtime::new()
        .context("no se pudo iniciar")
        .and_then(|tiempo| tiempo.block_on(ejecutar(&carpeta, elegir)));
    if let Err(error) = resultado {
        tracing::error!("{error:#}");
        mostrar_error(&format!("{error:#}"), &carpeta.join("ventana"));
    }
}

async fn ejecutar(carpeta: &Path, elegir: bool) -> anyhow::Result<()> {
    let archivo = carpeta.join(ARCHIVO_DE_PREFERENCIAS);
    let preferencias = Preferencias::cargar(&archivo);
    let perfil = carpeta.join("ventana");
    if !elegir && let Some(servidor) = preferencias.servidor_elegido() {
        // Contra el servidor no hace falta nada en marcha aquí: solo la ventana.
        abrir_ventana(&format!("{servidor}/"), &perfil)?;
        return Ok(());
    }
    let inicio = if elegir || preferencias.conexion.is_none() {
        "/escritorio/conexion"
    } else {
        "/"
    };
    let config = configuracion_local(carpeta)?;
    let url = format!("http://127.0.0.1:{}{inicio}", config.puerto);
    if responde_alumframe(config.puerto) {
        abrir_ventana(&url, &perfil)?;
        return Ok(());
    }
    servir_mientras_la_ventana_siga_abierta(&config, archivo, (&url, &perfil)).await
}

async fn servir_mientras_la_ventana_siga_abierta(
    config: &Configuracion,
    preferencias: PathBuf,
    (url, perfil): (&str, &Path),
) -> anyhow::Result<()> {
    let estado = match preparar_estado(config).await {
        Ok(estado) => estado.con_preferencias_escritorio(preferencias),
        Err(ErrorArranque::BaseNoUsable { error, carpeta }) => {
            let recuperacion = Recuperacion::desde(&error, &carpeta);
            return servir_recuperacion(config, recuperacion, perfil).await;
        }
        Err(otro) => return Err(otro.into()),
    };
    let escucha = escuchar(config)
        .await
        .with_context(|| format!("el puerto {} está ocupado por otro programa", config.puerto))?;
    let (avisar, apagado) = tokio::sync::oneshot::channel::<()>();
    let servidor = tokio::spawn(servir_hasta(escucha, estado, async {
        let _ = apagado.await;
    }));
    if esperar_a_que_se_cierre(abrir_ventana(url, perfil)?).await? {
        let _ = avisar.send(());
    }
    servidor.await??;
    Ok(())
}

/// La base de este equipo no se puede usar: en vez del programa, una pantalla que lo explica y
/// ofrece abrir la carpeta de datos o, si es una base antigua, empezar de cero.
async fn servir_recuperacion(
    config: &Configuracion,
    recuperacion: Recuperacion,
    perfil: &Path,
) -> anyhow::Result<()> {
    let token = generar_token().map_err(|e| anyhow::anyhow!("{e}"))?;
    let marca = formatear_utc(time::OffsetDateTime::now_utc()).replace(':', "-");
    let router = crear_router_de_recuperacion(recuperacion, (token, marca));
    let escucha = escuchar(config)
        .await
        .with_context(|| format!("el puerto {} está ocupado por otro programa", config.puerto))?;
    let (avisar, apagado) = tokio::sync::oneshot::channel::<()>();
    let servidor = tokio::spawn(async move {
        axum::serve(escucha, router)
            .with_graceful_shutdown(async {
                let _ = apagado.await;
            })
            .await
    });
    let inicio = format!("http://127.0.0.1:{}/", config.puerto);
    if esperar_a_que_se_cierre(abrir_ventana(&inicio, perfil)?).await? {
        let _ = avisar.send(());
    }
    servidor.await??;
    Ok(())
}

/// `true` cuando la ventana se ha cerrado. Sin Edge ni Chrome se abrió el navegador normal y no
/// se sabe cuándo se cierra: el programa sigue en marcha hasta apagar o cerrar la sesión.
/// El proceso de Edge puede terminar sin que se cierre la ventana (si Edge ya tenía abierto el
/// perfil, se la pasa a ese y termina): solo se apaga cuando además la página deja de latir.
async fn esperar_a_que_se_cierre(ventana: Option<std::process::Child>) -> anyhow::Result<bool> {
    let Some(mut ventana) = ventana else {
        anotar("sin navegador compatible: se usa el predeterminado");
        return Ok(false);
    };
    tokio::task::spawn_blocking(move || ventana.wait()).await??;
    anotar("el proceso de la ventana ha terminado; se sigue mientras la página dé señales");
    while la_ventana_sigue_abierta() {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
    anotar("la ventana ya no da señales: Aluframe se cierra");
    Ok(true)
}

fn anotar(texto: &str) {
    tracing::info!("{texto}");
}

/// Modo local: solo este equipo lo ve (127.0.0.1) y los datos van en la carpeta del usuario.
fn configuracion_local(carpeta: &Path) -> anyhow::Result<Configuracion> {
    let base = Configuracion {
        modo: Modo::Local,
        carpeta_datos: carpeta.join("datos"),
        abrir_navegador: Some(false),
        ..Configuracion::default()
    };
    Ok(base.aplicar_variables(|nombre| std::env::var(nombre).ok())?)
}

fn iniciar_registro(carpeta: &Path) {
    let ruta = carpeta.join(ARCHIVO_DE_REGISTRO);
    let grande = std::fs::metadata(&ruta).is_ok_and(|m| m.len() > TAMANO_MAXIMO_DEL_REGISTRO);
    let archivo = std::fs::OpenOptions::new()
        .create(true)
        .append(!grande)
        .write(true)
        .truncate(grande)
        .open(&ruta);
    if let Ok(archivo) = archivo {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_ansi(false)
            .with_writer(Mutex::new(archivo))
            .init();
    }
}
