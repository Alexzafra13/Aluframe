use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::Response;
use axum::routing::{get, post};

use super::extractores::ActorActual;
use super::middleware::{
    anadir_cabeceras_de_seguridad, cargar_sesion, exigir_configuracion, proteger_csrf,
};
use super::recursos::servir_recurso;
use super::rutas::paginas::{SECCIONES_PENDIENTES, mostrar_pendiente};
use super::rutas::{
    acceso, ajustes, ajustes_presupuestos, datos, escritorio, paginas, primer_arranque, salud,
    taller,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Las importaciones CSV (hasta 2 MB de texto) llegan codificadas como formulario.
const LIMITE_DE_CUERPO: usize = 8 * 1024 * 1024;
/// Una copia completa de los datos llega como archivo binario (ADR-0016).
const LIMITE_DE_COPIA: usize = 512 * 1024 * 1024;

pub fn crear_router(estado: EstadoApp) -> Router {
    let mut router = rutas_de_acceso()
        .merge(rutas_de_ajustes())
        .merge(super::router_catalogo::rutas_de_catalogo())
        .merge(super::router_clientes::rutas_de_clientes())
        .merge(super::router_presupuestos::rutas_de_presupuestos())
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
        .route(
            "/escritorio/conexion",
            get(escritorio::mostrar_conexion).post(escritorio::elegir_conexion),
        )
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
            "/ajustes/presupuestos",
            get(ajustes_presupuestos::mostrar).post(ajustes_presupuestos::guardar),
        )
        .route(
            "/ajustes/taller",
            get(taller::mostrar_taller).post(taller::guardar_taller),
        )
        .route("/ajustes/datos", get(datos::mostrar_datos))
        .route("/ajustes/datos/exportar", get(datos::descargar_copia))
        .route(
            "/ajustes/datos/importar",
            post(datos::subir_copia).layer(DefaultBodyLimit::max(LIMITE_DE_COPIA)),
        )
        .route(
            "/ajustes/datos/importar/{clave}",
            get(datos::revisar_copia).post(datos::confirmar_copia),
        )
}
