use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

use crate::aplicacion::series_incluidas::{CargaDeSerie, cargar_series_incluidas};
use crate::bd::{BaseDeDatos, ErrorEsquema, preparar_esquema};
use crate::config::{Configuracion, ErrorConfiguracion, NOMBRE_ARCHIVO_CONFIGURACION};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;
use crate::http::crear_router;
use crate::reloj::{Reloj, RelojDelSistema};

#[derive(Debug, Error)]
pub enum ErrorArranque {
    #[error(transparent)]
    Configuracion(#[from] ErrorConfiguracion),
    #[error(transparent)]
    Esquema(#[from] ErrorEsquema),
    #[error("No se pudo abrir la base de datos: {0}")]
    BaseDeDatos(#[from] rusqlite::Error),
    #[error("No se pudo crear la carpeta de datos {ruta}: {causa}")]
    Carpeta {
        ruta: PathBuf,
        causa: std::io::Error,
    },
    #[error("{0}")]
    App(#[from] ErrorApp),
}

/// Las rutas relativas cuentan desde la carpeta del ejecutable: un servicio no arranca en ella.
pub fn cargar_configuracion() -> Result<Configuracion, ErrorArranque> {
    let carpeta = carpeta_del_ejecutable();
    let archivo = std::env::var_os("ALUMFRAME_CONFIG")
        .map_or_else(|| carpeta.join(NOMBRE_ARCHIVO_CONFIGURACION), PathBuf::from);
    let mut config = Configuracion::cargar(&archivo, |nombre| std::env::var(nombre).ok())?;
    if config.carpeta_datos.is_relative() {
        config.carpeta_datos = carpeta.join(&config.carpeta_datos);
    }
    Ok(config)
}

fn carpeta_del_ejecutable() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|ruta| ruta.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub async fn preparar_estado(config: &Configuracion) -> Result<EstadoApp, ErrorArranque> {
    std::fs::create_dir_all(&config.carpeta_datos).map_err(|causa| ErrorArranque::Carpeta {
        ruta: config.carpeta_datos.clone(),
        causa,
    })?;
    let reloj: Arc<dyn Reloj> = Arc::new(RelojDelSistema);
    {
        let mut conexion = rusqlite::Connection::open(config.ruta_base_de_datos())?;
        preparar_esquema(&mut conexion, &formatear_utc(reloj.ahora()))?;
    }
    let bd = BaseDeDatos::abrir(&config.ruta_base_de_datos())?;
    let estado = EstadoApp::nuevo(bd, reloj)
        .await?
        .con_datos_demo(config.permitir_datos_demo);
    anotar_series_incluidas(&cargar_series_incluidas(&estado).await?);
    Ok(estado)
}

fn anotar_series_incluidas(cargas: &[CargaDeSerie]) {
    for (es_error, texto) in cargas.iter().filter_map(CargaDeSerie::describir) {
        if es_error {
            anotar_error(&texto);
        } else {
            anotar_informacion(&texto);
        }
    }
}

fn anotar_error(texto: &str) {
    tracing::error!("{texto}");
}

fn anotar_informacion(texto: &str) {
    tracing::info!("{texto}");
}

pub async fn servir_http(config: &Configuracion, estado: EstadoApp) -> std::io::Result<()> {
    let direccion = SocketAddr::new(config.direccion_de_escucha(), config.puerto);
    let escucha = tokio::net::TcpListener::bind(direccion).await?;
    let url = format!("http://localhost:{}", config.puerto);
    let datos = config.carpeta_datos.display().to_string();
    tracing::info!(%datos, %url, "AlumFrame escuchando en {direccion}");
    if config.debe_abrir_navegador() {
        abrir_navegador(&url);
    }
    let aplicacion = crear_router(estado).into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(escucha, aplicacion)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
}

fn abrir_navegador(url: &str) {
    let resultado = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(url).spawn()
    };
    if let Err(error) = resultado {
        tracing::warn!("no se pudo abrir el navegador: {error}. Abre {url} a mano.");
    }
}
