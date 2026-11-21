use axum::Router;
use axum::response::Redirect;
use axum::routing::{get, post};

use super::rutas::{catalogo_articulos, catalogo_fichas, precios, series, tarifas};
use crate::estado::EstadoApp;

pub fn rutas_de_catalogo() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/catalogo",
            get(|| async { Redirect::to("/catalogo/articulos") }),
        )
        .merge(rutas_de_fichas())
        .merge(rutas_de_articulos())
        .merge(rutas_de_tarifas())
        .merge(rutas_de_series())
}

fn rutas_de_fichas() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/catalogo/proveedores",
            get(catalogo_fichas::mostrar_proveedores).post(catalogo_fichas::alta_de_proveedor),
        )
        .route(
            "/catalogo/proveedores/{id}",
            get(catalogo_fichas::mostrar_proveedor).post(catalogo_fichas::guardar_proveedor),
        )
        .route(
            "/catalogo/acabados",
            get(catalogo_fichas::mostrar_acabados).post(catalogo_fichas::alta_de_acabado),
        )
        .route(
            "/catalogo/acabados/{id}",
            get(catalogo_fichas::mostrar_acabado).post(catalogo_fichas::guardar_acabado),
        )
        .route(
            "/catalogo/{tipo}/{id}/{accion}",
            post(catalogo_fichas::cambiar_archivo),
        )
}

fn rutas_de_articulos() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/catalogo/articulos",
            get(catalogo_articulos::mostrar_articulos),
        )
        .route(
            "/catalogo/articulos/nuevo",
            get(catalogo_articulos::mostrar_alta).post(catalogo_articulos::procesar_alta),
        )
        .route("/catalogo/articulos/precios", post(precios::guardar))
        .route("/catalogo/articulos/ajuste", post(precios::ajustar))
        .route(
            "/catalogo/articulos/{id}",
            get(catalogo_articulos::mostrar_articulo).post(catalogo_articulos::guardar_articulo),
        )
}

fn rutas_de_tarifas() -> Router<EstadoApp> {
    Router::new()
        .route("/catalogo/tarifas", get(tarifas::mostrar_tarifas))
        .route(
            "/catalogo/tarifas/importar",
            get(tarifas::mostrar_importacion).post(tarifas::confirmar),
        )
        .route("/catalogo/tarifas/importar/revisar", post(tarifas::revisar))
        .route("/catalogo/tarifas/{id}", get(tarifas::mostrar_tarifa))
        .route("/catalogo/tarifas/{id}/deshacer", post(tarifas::deshacer))
}

fn rutas_de_series() -> Router<EstadoApp> {
    Router::new()
        .route("/catalogo/series", get(series::mostrar_series))
        .route(
            "/catalogo/series/importar",
            get(series::mostrar_importacion).post(series::confirmar_importacion),
        )
        .route(
            "/catalogo/series/importar/revisar",
            post(series::revisar_importacion),
        )
        .route(
            "/catalogo/series/{codigo}/probar",
            get(series::mostrar_prueba),
        )
}
