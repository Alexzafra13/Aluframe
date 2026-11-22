use axum::Router;
use axum::routing::{get, post};

use super::rutas::{mi_dia, trabajos};
use crate::estado::EstadoApp;

pub fn rutas_de_trabajos() -> Router<EstadoApp> {
    Router::new()
        .route("/", get(trabajos::mostrar_inicio))
        .route("/mi-dia", get(mi_dia::mostrar_mi_dia))
        .route("/mi-dia/{id}", get(mi_dia::mostrar_ventana))
        .route("/mi-dia/{id}/falta", post(mi_dia::avisar_falta))
        .route("/mi-dia/pasos/{paso}", post(mi_dia::cambiar_el_paso))
        .route("/trabajos", get(trabajos::mostrar_trabajos))
        .route("/trabajos/{id}", get(trabajos::mostrar_trabajo))
        .route("/trabajos/{id}/datos", post(trabajos::guardar_datos))
        .route("/trabajos/{id}/orden", get(trabajos::mostrar_orden))
        .route("/trabajos/{id}/entregar", post(trabajos::entregar))
        .route("/trabajos/{id}/anular", post(trabajos::anular))
        .route(
            "/trabajos/{id}/ventanas/{ventana}/operario",
            post(trabajos::asignar_la_ventana),
        )
        .route(
            "/trabajos/{id}/ventanas/{ventana}/cristal",
            post(trabajos::anotar_cristal),
        )
        .route(
            "/trabajos/{id}/ventanas/{ventana}/falta",
            post(trabajos::quitar_falta),
        )
}
