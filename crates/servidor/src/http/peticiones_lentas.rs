//! Anota en el registro las peticiones lentas, para ver en el servidor del taller lo que en las
//! pruebas no se ve (la tablet, Tailscale, una base con años de datos). Hoy todo responde en
//! menos de 10 ms (docs/METRICAS.md). Se anota la ruta como patrón (`/clientes/{id}`) y nunca la
//! dirección completa, que puede llevar lo que se buscó (datos de clientes).

use std::time::{Duration, Instant};

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;

/// Muy por encima de lo normal y aún por debajo de lo que nota quien espera.
const UMBRAL: Duration = Duration::from_millis(200);

pub async fn anotar_si_tarda(peticion: Request, siguiente: Next) -> Response {
    let inicio = Instant::now();
    let metodo = peticion.method().clone();
    let ruta = peticion
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "(sin ruta)".to_owned(), |r| r.as_str().to_owned());
    let respuesta = siguiente.run(peticion).await;
    let duracion = inicio.elapsed();
    if es_lenta(duracion) {
        tracing::warn!(
            "petición lenta: {metodo} {ruta} tardó {} ms ({})",
            duracion.as_millis(),
            respuesta.status().as_u16()
        );
    }
    respuesta
}

fn es_lenta(duracion: Duration) -> bool {
    duracion >= UMBRAL
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn solo_es_lenta_a_partir_del_umbral() {
        assert!(!es_lenta(Duration::from_millis(10)));
        assert!(!es_lenta(Duration::from_millis(199)));
        assert!(es_lenta(Duration::from_millis(200)));
    }
}
