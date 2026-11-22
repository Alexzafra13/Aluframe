//! Exportar e importar todos los datos (ADR-0016): para pasar del modo local al servidor y al
//! revés. Importar **sustituye** la base entera; antes se guarda una copia de lo que había.

use std::path::{Path, PathBuf};

use dominio::acceso::Accion;

use super::{Actor, autorizar};
use crate::autenticacion::generar_token;
use crate::bd::consultas::auditoria::{Apunte, registrar};
use crate::bd::copia::{self, ResumenDeDatos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

const PREFIJO_IMPORTACION: &str = "importacion-";
const EXTENSION: &str = ".alumframe";

pub struct CopiaExportada {
    pub nombre: String,
    pub bytes: Vec<u8>,
}

pub struct VistaDeImportacion {
    pub actual: ResumenDeDatos,
    pub archivo: ResumenDeDatos,
}

pub async fn exportar_datos(estado: &EstadoApp, actor: &Actor) -> Result<CopiaExportada, ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    let carpeta = preparar_carpeta(estado)?;
    let ruta = carpeta.join(format!("exportacion-{}{EXTENSION}", generar_token()?));
    let destino = ruta.clone();
    estado
        .bd
        .con_escritor_entero(move |con| copia::exportar(con, &destino))
        .await?;
    let bytes = std::fs::read(&ruta).map_err(error_de_archivo);
    let _ = std::fs::remove_file(&ruta);
    let ahora = formatear_utc(estado.ahora());
    anotar(estado, Some(actor.id), "exportar_datos").await?;
    Ok(CopiaExportada {
        nombre: format!(
            "alumframe-{}{EXTENSION}",
            ahora.get(..10).unwrap_or("copia")
        ),
        bytes: bytes?,
    })
}

/// Guarda el archivo subido mientras se revisa; devuelve la clave para confirmarlo después.
pub async fn recibir_importacion(
    estado: &EstadoApp,
    actor: &Actor,
    bytes: Vec<u8>,
) -> Result<String, ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    let carpeta = preparar_carpeta(estado)?;
    borrar_importaciones_pendientes(&carpeta);
    let clave = generar_token()?;
    let ruta = ruta_de_importacion(&carpeta, &clave);
    std::fs::write(&ruta, bytes).map_err(error_de_archivo)?;
    if let Err(error) = comprobar(&ruta).await {
        let _ = std::fs::remove_file(&ruta);
        return Err(error);
    }
    Ok(clave)
}

pub async fn previsualizar_importacion(
    estado: &EstadoApp,
    actor: &Actor,
    clave: &str,
) -> Result<VistaDeImportacion, ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    let archivo = comprobar(&importacion_existente(estado, clave)?).await?;
    let actual = estado.bd.leer(copia::resumir).await?;
    Ok(VistaDeImportacion { actual, archivo })
}

/// Sustituye los datos por los del archivo. Devuelve el nombre de la copia de lo anterior.
pub async fn confirmar_importacion(
    estado: &EstadoApp,
    actor: &Actor,
    clave: &str,
) -> Result<String, ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    let origen = importacion_existente(estado, clave)?;
    let resumen = comprobar(&origen).await?;
    let ahora = formatear_utc(estado.ahora()).replace(':', "-");
    let nombre_previa = format!(
        "antes-de-importar-{ahora}-{}{EXTENSION}",
        clave.get(..8).unwrap_or(clave)
    );
    let previa = estado.carpeta_copias().join(&nombre_previa);
    let desde = origen.clone();
    let ahora_utc = formatear_utc(estado.ahora());
    estado
        .bd
        .con_escritor_entero(move |con| {
            copia::exportar(con, &previa)?;
            copia::restaurar(con, &desde, &ahora_utc)
        })
        .await?;
    let _ = std::fs::remove_file(&origen);
    estado.fijar_hay_usuarios(resumen.usuarios > 0);
    // Quien importa puede no existir en los datos nuevos: el apunte va sin usuario.
    anotar(estado, None, "importar_datos").await?;
    Ok(nombre_previa)
}

async fn comprobar(ruta: &Path) -> Result<ResumenDeDatos, ErrorApp> {
    let ruta = ruta.to_path_buf();
    tokio::task::spawn_blocking(move || copia::comprobar_copia(&ruta))
        .await
        .map_err(|error| ErrorApp::Interno(format!("tarea interrumpida: {error}")))?
}

async fn anotar(
    estado: &EstadoApp,
    usuario_id: Option<i64>,
    accion: &'static str,
) -> Result<(), ErrorApp> {
    let ahora = formatear_utc(estado.ahora());
    estado
        .bd
        .escribir(move |tx| {
            let apunte = Apunte {
                cuando_en: &ahora,
                usuario_id,
                accion,
                entidad: "datos",
                entidad_id: None,
            };
            registrar(tx, &apunte)
        })
        .await
}

fn preparar_carpeta(estado: &EstadoApp) -> Result<PathBuf, ErrorApp> {
    let carpeta = estado.carpeta_copias().to_path_buf();
    std::fs::create_dir_all(&carpeta).map_err(error_de_archivo)?;
    Ok(carpeta)
}

/// La clave llega de la URL: solo se aceptan las que genera `generar_token` (64 cifras hex).
fn importacion_existente(estado: &EstadoApp, clave: &str) -> Result<PathBuf, ErrorApp> {
    let valida = clave.len() == 64 && clave.bytes().all(|b| b.is_ascii_hexdigit());
    let ruta = ruta_de_importacion(estado.carpeta_copias(), clave);
    if valida && ruta.is_file() {
        Ok(ruta)
    } else {
        Err(ErrorApp::NoEncontrado)
    }
}

fn ruta_de_importacion(carpeta: &Path, clave: &str) -> PathBuf {
    carpeta.join(format!("{PREFIJO_IMPORTACION}{clave}{EXTENSION}"))
}

fn borrar_importaciones_pendientes(carpeta: &Path) {
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        return;
    };
    for entrada in entradas.flatten() {
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        if nombre.starts_with(PREFIJO_IMPORTACION) && nombre.ends_with(EXTENSION) {
            let _ = std::fs::remove_file(entrada.path());
        }
    }
}

fn error_de_archivo(error: std::io::Error) -> ErrorApp {
    ErrorApp::Interno(format!("archivo de copia: {error}"))
}
