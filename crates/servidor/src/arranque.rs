use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

use crate::aplicacion::acabados_incluidos::cargar_acabados_incluidos;
use crate::aplicacion::copias::discos::DiscosDeCopia;
use crate::aplicacion::copias::programador::{copiar_al_cerrar, vigilar_copias};
use crate::aplicacion::series_incluidas::{CargaDeSerie, cargar_series_incluidas};
use crate::bd::consultas::configuracion;
use crate::bd::{BaseDeDatos, ErrorEsquema, migraciones_pendientes, preparar_esquema};
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
    preparar_base(config, &formatear_utc(reloj.ahora()))?;
    let bd = BaseDeDatos::abrir(&config.ruta_base_de_datos())?;
    let estado = EstadoApp::nuevo(bd, reloj)
        .await?
        .con_datos_demo(config.permitir_datos_demo)
        .con_carpeta_copias(config.carpeta_copias())
        .con_discos_de_copia(DiscosDeCopia {
            indicados: config.discos_de_copia.clone(),
            buscar_en_el_sistema: true,
        });
    if estado.bd.leer(configuracion::es_de_demostracion).await? {
        return Err(ErrorArranque::App(ErrorApp::Validacion(
            "Esta base de datos es la de la demostración (datos inventados): no se usa para \
             trabajar. Revisa la carpeta de datos."
                .into(),
        )));
    }
    anotar_series_incluidas(&cargar_series_incluidas(&estado).await?);
    cargar_acabados_incluidos(&estado).await?;
    Ok(match preparar_demostracion(config, estado.reloj()).await {
        Some(demo) => estado.con_demostracion(demo),
        None => estado,
    })
}

/// La demostración (ADR-0028), en su propia subcarpeta. Si algo falla, el programa sigue sin
/// ella: nunca impide trabajar con los datos de verdad.
async fn preparar_demostracion(config: &Configuracion, reloj: Arc<dyn Reloj>) -> Option<EstadoApp> {
    match abrir_demostracion(config, reloj).await {
        Ok(demo) => demo,
        Err(error) => {
            anotar_error(&format!("la demostración no está disponible: {error}"));
            None
        }
    }
}

async fn abrir_demostracion(
    config: &Configuracion,
    reloj: Arc<dyn Reloj>,
) -> Result<Option<EstadoApp>, ErrorArranque> {
    let carpeta = config.carpeta_demostracion();
    std::fs::create_dir_all(&carpeta).map_err(|causa| ErrorArranque::Carpeta {
        ruta: carpeta.clone(),
        causa,
    })?;
    let ruta = carpeta.join("alumframe.db");
    let mut conexion = rusqlite::Connection::open(&ruta)?;
    preparar_esquema(&mut conexion, &formatear_utc(reloj.ahora()))?;
    drop(conexion);
    let demo = EstadoApp::nuevo(BaseDeDatos::abrir(&ruta)?, reloj)
        .await?
        .como_demostracion()
        .con_carpeta_copias(carpeta.join("copias"));
    if demo.hay_usuarios() && !demo.bd.leer(configuracion::es_de_demostracion).await? {
        anotar_error("la carpeta de la demostración tiene una base sin su marca: no se toca");
        return Ok(None);
    }
    cargar_series_incluidas(&demo).await?;
    cargar_acabados_incluidos(&demo).await?;
    Ok(Some(demo))
}

/// Crea la base o la actualiza; antes de actualizar una con datos, guarda una copia
/// (`antes-de-actualizar-…`) en la carpeta de copias.
fn preparar_base(config: &Configuracion, ahora_utc: &str) -> Result<(), ErrorArranque> {
    let mut conexion = rusqlite::Connection::open(config.ruta_base_de_datos())?;
    let pendientes = migraciones_pendientes(&conexion)?;
    let tiene_datos = crate::bd::version_de_la_base(&conexion)?.is_some_and(|v| v > 0);
    if pendientes > 0 && tiene_datos {
        let carpeta = config.carpeta_copias();
        std::fs::create_dir_all(&carpeta).map_err(|causa| ErrorArranque::Carpeta {
            ruta: carpeta.clone(),
            causa,
        })?;
        let nombre = format!(
            "antes-de-actualizar-{}.alumframe",
            ahora_utc.replace(':', "-")
        );
        crate::bd::copia::exportar(&conexion, &carpeta.join(&nombre))?;
        anotar_informacion(&format!("copia previa a la actualización: {nombre}"));
    }
    preparar_esquema(&mut conexion, ahora_utc)?;
    Ok(())
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
    let escucha = escuchar(config).await?;
    let url = format!("http://localhost:{}", config.puerto);
    if config.debe_abrir_navegador() {
        abrir_navegador(&url);
    }
    servir_hasta(escucha, estado, esperar_orden_de_parar()).await
}

/// Ctrl+C en la consola o la parada del servicio (`systemctl stop` envía SIGTERM).
async fn esperar_orden_de_parar() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut terminar) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = terminar.recv() => {}
                }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

pub async fn escuchar(config: &Configuracion) -> std::io::Result<tokio::net::TcpListener> {
    let direccion = SocketAddr::new(config.direccion_de_escucha(), config.puerto);
    let escucha = tokio::net::TcpListener::bind(direccion).await?;
    let datos = config.carpeta_datos.display().to_string();
    tracing::info!(%datos, "AlumFrame escuchando en {direccion}");
    Ok(escucha)
}

/// Atiende hasta que `apagado` termina (Ctrl+C o parada del servicio en el servidor; cerrar la
/// ventana en el escritorio). Mientras, vigila las copias; al terminar hace la copia de cierre.
pub async fn servir_hasta(
    escucha: tokio::net::TcpListener,
    estado: EstadoApp,
    apagado: impl std::future::Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    let vigilante = tokio::spawn(vigilar_copias(estado.clone()));
    let aplicacion =
        crear_router(estado.clone()).into_make_service_with_connect_info::<SocketAddr>();
    let resultado = axum::serve(escucha, aplicacion)
        .with_graceful_shutdown(apagado)
        .await;
    vigilante.abort();
    copiar_al_cerrar(&estado).await;
    resultado
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
