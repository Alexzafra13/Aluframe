use axum::Router;
use axum::routing::{get, post};

use super::rutas::{facturas, facturas_acciones as acciones};
use crate::estado::EstadoApp;

pub fn rutas_de_facturas() -> Router<EstadoApp> {
    Router::new()
        .route("/facturas", get(facturas::mostrar_facturas))
        .route("/facturas/libre", post(acciones::nueva_libre))
        .route("/facturas/{id}", get(facturas::mostrar_factura))
        .route("/facturas/{id}/imprimir", get(facturas::imprimir_factura))
        .route("/facturas/{id}/datos", post(acciones::datos))
        .route("/facturas/{id}/lineas", post(acciones::anadir))
        .route(
            "/facturas/{id}/lineas/{linea}/quitar",
            post(acciones::quitar),
        )
        .route("/facturas/{id}/emitir", post(acciones::emitir))
        .route("/facturas/{id}/descartar", post(acciones::descartar))
        .route("/facturas/{id}/cobros", post(acciones::cobrar))
        .route("/facturas/{id}/rectificar", post(acciones::rectificar))
        .route("/cobros/{id}/anular", post(acciones::anular))
        .route("/trabajos/{id}/factura", post(acciones::facturar_trabajo))
        .route("/trabajos/{id}/senal", post(acciones::senal))
        .route("/trabajos/{id}/albaran", post(acciones::albaran))
        .route("/albaranes/{id}/imprimir", get(facturas::imprimir_albaran))
}
