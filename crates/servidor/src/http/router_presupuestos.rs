use axum::Router;
use axum::routing::{get, post};

use super::rutas::{editor_linea, presupuesto_acciones, presupuesto_impreso, presupuestos};
use crate::estado::EstadoApp;

pub fn rutas_de_presupuestos() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/presupuestos",
            get(presupuestos::mostrar_lista).post(presupuestos::crear),
        )
        .route("/presupuestos/{id}", get(presupuestos::mostrar))
        .route(
            "/presupuestos/{id}/imprimir",
            get(presupuesto_impreso::mostrar),
        )
        .route(
            "/presupuestos/{id}/datos",
            post(presupuesto_acciones::guardar_datos),
        )
        .route(
            "/presupuestos/{id}/estado/{nuevo}",
            post(presupuesto_acciones::cambiar),
        )
        .route(
            "/presupuestos/{id}/version",
            post(presupuesto_acciones::version),
        )
        .route(
            "/presupuestos/{id}/aceptar",
            post(presupuesto_acciones::aceptar),
        )
        .route(
            "/presupuestos/{id}/recalcular",
            post(presupuesto_acciones::recalcular),
        )
        .route(
            "/presupuestos/{id}/renovar",
            post(presupuesto_acciones::renovar),
        )
        .merge(rutas_de_lineas())
}

fn rutas_de_lineas() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/presupuestos/{id}/lineas/libre",
            post(presupuesto_acciones::libre),
        )
        .route(
            "/presupuestos/{id}/lineas/calcular",
            post(editor_linea::calcular),
        )
        .route(
            "/presupuestos/{id}/lineas/nueva",
            get(editor_linea::mostrar_nueva).post(editor_linea::guardar_nueva),
        )
        .route(
            "/presupuestos/{id}/lineas/{orden}",
            get(editor_linea::mostrar_linea).post(editor_linea::guardar_existente),
        )
        .route(
            "/presupuestos/{id}/lineas/{orden}/quitar",
            post(presupuesto_acciones::quitar),
        )
}
