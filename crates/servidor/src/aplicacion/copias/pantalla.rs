//! Ajustes > Copias: ver las copias y los discos, copiar ahora, preparar un disco y restaurar.
//! Restaurar reutiliza la importación (ADR-0016): se revisa antes y se guarda lo de ahora.

use std::path::{Path, PathBuf};

use dominio::acceso::Accion;
use dominio::copias::{MomentoLocal, leer_nombre_de_copia};

use super::hacer::{carpeta_local, hacer_copia};
use super::registro::{RegistroDeCopias, leer_registro};
use crate::aplicacion::datos::recibir_importacion;
use crate::aplicacion::demostracion::exigir_datos_reales;
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{leer_utc, momento_de_madrid};

/// Sin una copia completa en este tiempo, el Inicio avisa aunque el último intento saliera bien.
const DIAS_SIN_COPIA_PARA_AVISAR: i64 = 3;

pub struct CopiaGuardada {
    pub nombre: String,
    pub momento: MomentoLocal,
    pub bytes: u64,
}

pub struct LugarDeCopias {
    /// Vacío para las de este equipo; la carpeta del disco en los demás.
    pub disco: String,
    pub copias: Vec<CopiaGuardada>,
}

pub struct EstadoDeLasCopias {
    pub registro: RegistroDeCopias,
    pub lugares: Vec<LugarDeCopias>,
    pub sin_preparar: Vec<String>,
}

pub fn consultar_copias(estado: &EstadoApp, actor: &Actor) -> Result<EstadoDeLasCopias, ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    exigir_datos_reales(estado)?;
    let local = carpeta_local(estado);
    let discos = estado.discos_de_copia();
    let mut lugares = vec![LugarDeCopias {
        disco: String::new(),
        copias: copias_en(&local),
    }];
    lugares.extend(discos.destinos().into_iter().map(|d| LugarDeCopias {
        copias: copias_en(&d),
        disco: d.display().to_string(),
    }));
    Ok(EstadoDeLasCopias {
        registro: leer_registro(&local),
        lugares,
        sin_preparar: discos
            .sin_preparar()
            .iter()
            .map(|r| r.display().to_string())
            .collect(),
    })
}

fn copias_en(carpeta: &Path) -> Vec<CopiaGuardada> {
    let mut copias: Vec<CopiaGuardada> = std::fs::read_dir(carpeta)
        .map(|entradas| entradas.flatten().filter_map(|e| leer_copia(&e)).collect())
        .unwrap_or_default();
    copias.sort_by_key(|c| std::cmp::Reverse(c.momento));
    copias
}

fn leer_copia(entrada: &std::fs::DirEntry) -> Option<CopiaGuardada> {
    let nombre = entrada.file_name().to_string_lossy().into_owned();
    let momento = leer_nombre_de_copia(&nombre)?;
    let bytes = entrada.metadata().ok()?.len();
    Some(CopiaGuardada {
        nombre,
        momento,
        bytes,
    })
}

pub async fn copiar_ahora(estado: &EstadoApp, actor: &Actor) -> Result<RegistroDeCopias, ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    exigir_datos_reales(estado)?;
    hacer_copia(estado).await
}

pub fn preparar_disco(estado: &EstadoApp, actor: &Actor, raiz: &str) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    exigir_datos_reales(estado)?;
    estado
        .discos_de_copia()
        .preparar(Path::new(raiz))
        .map(|_| ())
        .map_err(|error| ErrorApp::Validacion(format!("No se pudo preparar el disco: {error}.")))
}

/// Deja la copia elegida lista para revisar; devuelve la clave de la importación.
pub async fn restaurar_copia(
    estado: &EstadoApp,
    actor: &Actor,
    (disco, nombre): (&str, &str),
) -> Result<String, ErrorApp> {
    autorizar(actor, Accion::GestionarDatos)?;
    exigir_datos_reales(estado)?;
    let ruta = ruta_de_copia(estado, disco, nombre)?;
    let bytes = tokio::task::spawn_blocking(move || std::fs::read(ruta))
        .await
        .map_err(|e| ErrorApp::Interno(format!("tarea interrumpida: {e}")))?
        .map_err(|e| ErrorApp::Validacion(format!("No se pudo leer la copia: {e}.")))?;
    recibir_importacion(estado, actor, bytes).await
}

/// El disco y el nombre llegan de un formulario: solo valen los de la lista.
fn ruta_de_copia(estado: &EstadoApp, disco: &str, nombre: &str) -> Result<PathBuf, ErrorApp> {
    if leer_nombre_de_copia(nombre).is_none() {
        return Err(ErrorApp::NoEncontrado);
    }
    let carpeta = if disco.is_empty() {
        Some(carpeta_local(estado))
    } else {
        let destinos = estado.discos_de_copia().destinos();
        destinos
            .into_iter()
            .find(|d| d.display().to_string() == disco)
    };
    carpeta
        .map(|c| c.join(nombre))
        .filter(|ruta| ruta.is_file())
        .ok_or(ErrorApp::NoEncontrado)
}

/// El aviso del Inicio: el fallo del último intento o, si no, una copia completa demasiado vieja.
pub fn aviso_de_copias(estado: &EstadoApp) -> Option<String> {
    if estado.es_demostracion() {
        return None;
    }
    let registro = leer_registro(&carpeta_local(estado));
    registro.ultimo_intento.as_ref()?;
    if let Some(aviso) = registro.aviso {
        return Some(aviso);
    }
    let ultima = registro.ultima_completa.as_deref().and_then(leer_utc)?;
    let dias = momento_de_madrid(estado.ahora()).dias - momento_de_madrid(ultima).dias;
    (dias >= DIAS_SIN_COPIA_PARA_AVISAR)
        .then(|| format!("La última copia en el disco es de hace {dias} días."))
}
