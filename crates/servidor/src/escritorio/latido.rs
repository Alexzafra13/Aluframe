//! La ventana avisa cada poco de que sigue abierta. Hace falta porque el proceso de Edge que
//! lanzamos no siempre es el de la ventana: si Edge ya tenía abierto ese perfil, le pasa la
//! ventana y termina al momento, y el programa creía que se había cerrado y se apagaba.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// La página late cada 20 s; con la ventana minimizada el navegador puede espaciar sus
/// temporizadores hasta un minuto, así que se deja margen de sobra.
const SILENCIO_PARA_CERRAR: Duration = Duration::from_secs(150);

/// `ALUMFRAME_SILENCIO_PARA_CERRAR` (segundos) lo acorta en las pruebas.
fn silencio_para_cerrar() -> Duration {
    std::env::var("ALUMFRAME_SILENCIO_PARA_CERRAR")
        .ok()
        .and_then(|s| s.parse().ok())
        .map_or(SILENCIO_PARA_CERRAR, Duration::from_secs)
}

static ULTIMO_LATIDO: AtomicU64 = AtomicU64::new(0);

fn inicio() -> Instant {
    static INICIO: OnceLock<Instant> = OnceLock::new();
    *INICIO.get_or_init(Instant::now)
}

pub fn anotar_latido() {
    let segundos = inicio().elapsed().as_secs();
    ULTIMO_LATIDO.store(segundos, Ordering::Relaxed);
}

/// Tiempo sin latidos; sin ninguno todavía, el que lleva el programa en marcha.
pub fn tiempo_sin_latidos() -> Duration {
    let ahora = inicio().elapsed().as_secs();
    Duration::from_secs(ahora.saturating_sub(ULTIMO_LATIDO.load(Ordering::Relaxed)))
}

pub fn la_ventana_sigue_abierta() -> bool {
    tiempo_sin_latidos() < silencio_para_cerrar()
}

/// Cada cuánto se mira si la ventana sigue latiendo.
pub fn intervalo_de_comprobacion() -> Duration {
    intervalo_para(silencio_para_cerrar())
}

/// 10 s con el silencio normal; con uno corto (las pruebas) se mira antes, para no esperar de más.
fn intervalo_para(silencio: Duration) -> Duration {
    Duration::from_secs(10).min(silencio / 4)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn tras_un_latido_la_ventana_sigue_abierta() {
        anotar_latido();
        assert!(tiempo_sin_latidos() < Duration::from_secs(2));
        assert!(la_ventana_sigue_abierta());
    }

    #[test]
    fn se_comprueba_cada_diez_segundos_salvo_con_un_silencio_corto() {
        assert_eq!(
            intervalo_para(SILENCIO_PARA_CERRAR),
            Duration::from_secs(10)
        );
        assert_eq!(
            intervalo_para(Duration::from_secs(2)),
            Duration::from_millis(500)
        );
    }
}
