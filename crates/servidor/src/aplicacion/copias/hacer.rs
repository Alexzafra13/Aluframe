//! Hacer una copia: `VACUUM INTO` en el equipo (nunca se copia el archivo en caliente), se
//! comprueba, se lleva a cada disco de copias, se vuelve a comprobar allí y se borran las que
//! sobran (7 diarias y 4 semanales).

use std::path::{Path, PathBuf};

use dominio::copias::{copias_que_sobran, nombre_de_copia};

use super::registro::{RegistroDeCopias, guardar_registro, leer_registro};
use crate::bd::copia;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{a_hora_de_madrid, formatear_utc};

const SIN_DISCO: &str = "No hay ningún disco de copias conectado: la copia solo está en este \
                         equipo. Conecta el pendrive o disco con la carpeta AlumFrame-copias.";

/// La carpeta de las copias automáticas en este equipo.
pub fn carpeta_local(estado: &EstadoApp) -> PathBuf {
    estado.carpeta_copias().join("automaticas")
}

/// Hace la copia y anota el resultado. Error solo si ni siquiera se pudo copiar en el equipo.
pub async fn hacer_copia(estado: &EstadoApp) -> Result<RegistroDeCopias, ErrorApp> {
    let _turno = estado.turno_de_copia().lock().await;
    let local = carpeta_local(estado);
    std::fs::create_dir_all(&local).map_err(error_de_archivo)?;
    let ahora = estado.ahora();
    let nombre = nombre_para(ahora);
    let temporal = local.join(format!("{nombre}.tmp"));
    let _ = std::fs::remove_file(&temporal);
    let destino = temporal.clone();
    let hecha = estado
        .bd
        .con_escritor_entero(move |con| copia::exportar(con, &destino))
        .await;
    let destinos = estado.discos_de_copia().destinos();
    let resultado = match hecha {
        Ok(()) => {
            let datos = (local.clone(), nombre, destinos);
            tokio::task::spawn_blocking(move || repartir(&temporal, &datos))
                .await
                .map_err(|e| ErrorApp::Interno(format!("tarea interrumpida: {e}")))?
        }
        Err(error) => Err(error),
    };
    anotar(&local, &formatear_utc(ahora), resultado)
}

fn nombre_para(instante: time::OffsetDateTime) -> String {
    let local = a_hora_de_madrid(instante);
    nombre_de_copia(
        (local.year(), u8::from(local.month()), local.day()),
        (local.hour(), local.minute()),
    )
}

/// Discos en que quedó la copia y avisos de los que fallaron.
type Reparto = (Vec<String>, Vec<String>);

fn repartir(
    temporal: &Path,
    (local, nombre, destinos): &(PathBuf, String, Vec<PathBuf>),
) -> Result<Reparto, ErrorApp> {
    copia::comprobar_copia(temporal)?;
    let en_equipo = local.join(nombre);
    std::fs::rename(temporal, &en_equipo).map_err(error_de_archivo)?;
    retirar_las_que_sobran(local);
    let mut discos = Vec::new();
    let mut avisos = Vec::new();
    for destino in destinos {
        match llevar_a(&en_equipo, destino, nombre) {
            Ok(()) => discos.push(destino.display().to_string()),
            Err(error) => avisos.push(format!("{}: {error}", destino.display())),
        }
    }
    Ok((discos, avisos))
}

fn llevar_a(origen: &Path, carpeta: &Path, nombre: &str) -> Result<(), ErrorApp> {
    let temporal = carpeta.join(format!("{nombre}.tmp"));
    std::fs::copy(origen, &temporal).map_err(error_de_archivo)?;
    std::fs::File::open(&temporal)
        .and_then(|archivo| archivo.sync_all())
        .map_err(error_de_archivo)?;
    let final_ = carpeta.join(nombre);
    std::fs::rename(&temporal, &final_).map_err(error_de_archivo)?;
    copia::comprobar_copia(&final_)?;
    retirar_las_que_sobran(carpeta);
    Ok(())
}

/// Solo borra copias automáticas por su nombre: nada más de la carpeta se toca.
fn retirar_las_que_sobran(carpeta: &Path) {
    let nombres: Vec<String> = std::fs::read_dir(carpeta)
        .map(|entradas| {
            entradas
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    for nombre in copias_que_sobran(&nombres) {
        let _ = std::fs::remove_file(carpeta.join(nombre));
    }
}

fn anotar(
    local: &Path,
    ahora: &str,
    resultado: Result<Reparto, ErrorApp>,
) -> Result<RegistroDeCopias, ErrorApp> {
    let mut registro = leer_registro(local);
    registro.ultimo_intento = Some(ahora.to_owned());
    let fallo = match resultado {
        Ok((discos, avisos)) => {
            if !discos.is_empty() {
                registro.ultima_completa = Some(ahora.to_owned());
            }
            registro.aviso = describir_avisos(discos.is_empty(), &avisos);
            registro.discos = discos;
            None
        }
        Err(error) => {
            registro.aviso = Some(format!("No se pudo hacer la copia: {error}"));
            Some(error)
        }
    };
    if let Err(error) = guardar_registro(local, &registro) {
        tracing::error!("no se pudo anotar el registro de copias: {error}");
    }
    match fallo {
        Some(error) => Err(error),
        None => Ok(registro),
    }
}

fn describir_avisos(sin_discos: bool, avisos: &[String]) -> Option<String> {
    match (sin_discos, avisos.is_empty()) {
        (true, true) => Some(SIN_DISCO.to_owned()),
        (_, false) => Some(format!("Falló la copia en: {}", avisos.join("; "))),
        (false, true) => None,
    }
}

fn error_de_archivo(error: std::io::Error) -> ErrorApp {
    ErrorApp::Interno(format!("archivo de copia: {error}"))
}
