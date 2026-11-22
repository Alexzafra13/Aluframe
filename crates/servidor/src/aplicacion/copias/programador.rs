//! Cuándo se copia sin que nadie lo pida: a las 14:00, al arrancar si faltó la de las 14:00
//! (apagón, equipo apagado) y al cerrar el programa (decisión del propietario).

use std::time::Duration;

use dominio::copias::{MomentoLocal, toca_copia, toca_copia_al_cerrar};

use super::hacer::{carpeta_local, hacer_copia};
use super::registro::leer_registro;
use crate::estado::EstadoApp;
use crate::fecha::{leer_utc, momento_de_madrid};

const CADA: Duration = Duration::from_secs(60);

/// Mira cada minuto si toca copia. La primera vuelta es al arrancar.
pub async fn vigilar_copias(estado: EstadoApp) {
    let mut reloj = tokio::time::interval(CADA);
    loop {
        reloj.tick().await;
        let registro = leer_registro(&carpeta_local(&estado));
        let ahora = momento_de_madrid(estado.ahora());
        let toca = toca_copia(
            momento(registro.ultima_completa.as_deref()),
            momento(registro.ultimo_intento.as_deref()),
            ahora,
        );
        if toca && estado.hay_usuarios() {
            copiar_y_anotar(&estado, "programada").await;
        }
    }
}

pub async fn copiar_al_cerrar(estado: &EstadoApp) {
    let registro = leer_registro(&carpeta_local(estado));
    let ahora = momento_de_madrid(estado.ahora());
    if estado.hay_usuarios()
        && toca_copia_al_cerrar(momento(registro.ultima_completa.as_deref()), ahora)
    {
        copiar_y_anotar(estado, "al cerrar").await;
    }
}

async fn copiar_y_anotar(estado: &EstadoApp, cuando: &str) {
    match hacer_copia(estado).await {
        Ok(registro) => match registro.aviso {
            None => anotar_bien(&format!("copia {cuando} en {}", registro.discos.join(", "))),
            Some(aviso) => anotar_aviso(&format!("copia {cuando}: {aviso}")),
        },
        Err(error) => anotar_aviso(&format!("copia {cuando} fallida: {error}")),
    }
}

fn anotar_bien(texto: &str) {
    tracing::info!("{texto}");
}

fn anotar_aviso(texto: &str) {
    tracing::warn!("{texto}");
}

fn momento(utc: Option<&str>) -> Option<MomentoLocal> {
    utc.and_then(leer_utc).map(momento_de_madrid)
}
