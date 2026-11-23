use axum::Router;
use axum::routing::{get, post};

use super::rutas::{clientes, clientes_ficha, clientes_fragmentos, importar_clientes};
use crate::estado::EstadoApp;

pub fn rutas_de_clientes() -> Router<EstadoApp> {
    Router::new()
        .merge(rutas_de_importacion())
        .merge(rutas_de_poblaciones())
        .route("/clientes", get(clientes::mostrar_clientes))
        .route(
            "/clientes/nuevo",
            get(clientes::mostrar_alta).post(clientes::procesar_alta),
        )
        .route(
            "/clientes/comprobar-nif",
            get(clientes_fragmentos::comprobar_nif),
        )
        .route(
            "/clientes/direccion-en-blanco",
            get(clientes_fragmentos::direccion_en_blanco),
        )
        .route("/clientes/{id}", get(clientes_ficha::mostrar_ficha))
        .route(
            "/clientes/{id}/editar",
            get(clientes::mostrar_edicion).post(clientes::procesar_edicion),
        )
        .route("/clientes/{id}/archivar", post(clientes_ficha::archivar))
        .route("/clientes/{id}/recuperar", post(clientes_ficha::recuperar))
        .route("/clientes/{id}/borrar", post(clientes_ficha::borrar))
        .route("/clientes/{id}/exportar", get(clientes_ficha::exportar))
        .route(
            "/clientes/{id}/direcciones",
            post(clientes_ficha::nueva_direccion),
        )
        .route(
            "/clientes/{id}/direcciones/{direccion}/editar",
            get(clientes_ficha::mostrar_direccion).post(clientes_ficha::guardar_direccion),
        )
        .route(
            "/clientes/{id}/direcciones/{direccion}/quitar",
            post(clientes_ficha::quitar),
        )
}

fn rutas_de_importacion() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/clientes/importar",
            get(importar_clientes::mostrar).post(importar_clientes::confirmar),
        )
        .route(
            "/clientes/importar/revisar",
            post(importar_clientes::revisar),
        )
        .route(
            "/clientes/importaciones/{id}/deshacer",
            post(importar_clientes::deshacer),
        )
}

/// Para rellenar código postal, localidad y provincia en los formularios con dirección (ADR-0020).
fn rutas_de_poblaciones() -> Router<EstadoApp> {
    Router::new()
        .route("/poblaciones", get(clientes_fragmentos::buscar_poblacion))
        .route(
            "/poblaciones/sugerencias",
            get(clientes_fragmentos::sugerir_poblaciones),
        )
}
