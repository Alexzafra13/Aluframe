use axum::Router;
use axum::routing::{get, post};

use super::rutas::{
    avisos, inicio, materiales, medidas_trabajo, mi_dia, stock, terminar_orden, trabajos,
};
use crate::estado::EstadoApp;

pub fn rutas_de_trabajos() -> Router<EstadoApp> {
    Router::new()
        .route("/", get(inicio::mostrar_inicio))
        .route("/avisos/campana", get(avisos::mostrar_campana))
        .route("/mi-dia", get(mi_dia::mostrar_mi_dia))
        .route("/mi-dia/{id}", get(mi_dia::mostrar_ventana))
        .route("/mi-dia/{id}/falta", post(mi_dia::avisar_falta))
        .route("/mi-dia/{id}/repetir", post(mi_dia::repetir))
        .route("/mi-dia/pasos/{paso}", post(mi_dia::cambiar_el_paso))
        .route("/materiales/{id}", post(materiales::marcar))
        .route("/trabajos", get(trabajos::mostrar_trabajos))
        .route("/stock", get(stock::mostrar_stock))
        .route("/stock/entrada", post(stock::anotar_entrada))
        .route("/stock/ajuste", post(stock::anotar_ajuste))
        .route("/stock/compra", get(stock::mostrar_compra))
        .route("/stock/{id}", get(stock::mostrar_articulo))
        .route("/stock/{id}/minimo", post(stock::guardar_minimo))
        .route("/trabajos/{id}", get(trabajos::mostrar_trabajo))
        .route("/trabajos/{id}/datos", post(trabajos::guardar_datos))
        .route("/trabajos/{id}/orden", get(trabajos::mostrar_orden))
        .route(
            "/trabajos/{id}/terminar",
            get(terminar_orden::mostrar).post(terminar_orden::confirmar),
        )
        .route("/trabajos/{id}/entregar", post(trabajos::entregar))
        .route("/trabajos/{id}/anular", post(trabajos::anular))
        .route(
            "/trabajos/{id}/ventanas/{ventana}/operario",
            post(trabajos::asignar_la_ventana),
        )
        .route(
            "/trabajos/{id}/ventanas/{ventana}/medidas",
            get(medidas_trabajo::mostrar_medidas).post(medidas_trabajo::guardar_medidas),
        )
        .route(
            "/trabajos/{id}/ventanas/{ventana}/falta",
            post(trabajos::quitar_falta),
        )
}
