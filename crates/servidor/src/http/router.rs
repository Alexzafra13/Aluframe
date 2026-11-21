use axum::Router;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::{Redirect, Response};
use axum::routing::{get, post};

use super::extractores::ActorActual;
use super::middleware::{
    anadir_cabeceras_de_seguridad, cargar_sesion, exigir_configuracion, proteger_csrf,
};
use super::recursos::servir_recurso;
use super::rutas::paginas::{SECCIONES_PENDIENTES, mostrar_pendiente};
use super::rutas::{acceso, ajustes, paginas, primer_arranque, salud, series, taller};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub fn crear_router(estado: EstadoApp) -> Router {
    let mut router = rutas_de_acceso()
        .merge(rutas_de_ajustes())
        .merge(rutas_de_catalogo())
        .route("/", get(paginas::mostrar_inicio))
        .route("/mi-dia", get(paginas::mostrar_mi_dia));
    for seccion in &SECCIONES_PENDIENTES {
        let manejador = move |ActorActual(actor): ActorActual| async move {
            let respuesta: Result<Response, ErrorApp> = mostrar_pendiente(&actor, seccion);
            respuesta
        };
        router = router.route(seccion.ruta, get(manejador));
    }
    router
        .layer(from_fn(proteger_csrf))
        .layer(from_fn_with_state(estado.clone(), cargar_sesion))
        .layer(from_fn_with_state(estado.clone(), exigir_configuracion))
        .layer(from_fn(anadir_cabeceras_de_seguridad))
        .with_state(estado)
}

fn rutas_de_acceso() -> Router<EstadoApp> {
    Router::new()
        .route("/health", get(salud::comprobar_salud))
        .route("/recursos/{*ruta}", get(servir_recurso))
        .route(
            "/primer-arranque",
            get(primer_arranque::mostrar_primer_arranque)
                .post(primer_arranque::procesar_primer_arranque),
        )
        .route(
            "/entrar",
            get(acceso::mostrar_entrar).post(acceso::procesar_entrar),
        )
        .route("/salir", post(acceso::salir))
}

fn rutas_de_ajustes() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/ajustes/usuarios",
            get(ajustes::mostrar_usuarios).post(ajustes::alta_de_operario),
        )
        .route("/ajustes/usuarios/{id}/pin", post(ajustes::cambio_de_pin))
        .route(
            "/ajustes/usuarios/{id}/desactivar",
            post(ajustes::desactivacion),
        )
        .route("/ajustes/usuarios/{id}/activar", post(ajustes::activacion))
        .route(
            "/ajustes/taller",
            get(taller::mostrar_taller).post(taller::guardar_taller),
        )
}

fn rutas_de_catalogo() -> Router<EstadoApp> {
    Router::new()
        .route(
            "/catalogo",
            get(|| async { Redirect::to("/catalogo/series") }),
        )
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
