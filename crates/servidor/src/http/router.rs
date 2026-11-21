use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::{Redirect, Response};
use axum::routing::{get, post};

use super::extractores::ActorActual;
use super::middleware::{
    anadir_cabeceras_de_seguridad, cargar_sesion, exigir_configuracion, proteger_csrf,
};
use super::recursos::servir_recurso;
use super::rutas::paginas::{SECCIONES_PENDIENTES, mostrar_pendiente};
use super::rutas::{
    acceso, ajustes, clientes, clientes_ficha, clientes_fragmentos, paginas, primer_arranque,
    salud, series, taller,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Las importaciones CSV (hasta 2 MB de texto) llegan codificadas como formulario.
const LIMITE_DE_CUERPO: usize = 8 * 1024 * 1024;

pub fn crear_router(estado: EstadoApp) -> Router {
    let mut router = rutas_de_acceso()
        .merge(rutas_de_ajustes())
        .merge(rutas_de_catalogo())
        .merge(rutas_de_clientes())
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
        .layer(DefaultBodyLimit::max(LIMITE_DE_CUERPO))
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

fn rutas_de_clientes() -> Router<EstadoApp> {
    Router::new()
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
