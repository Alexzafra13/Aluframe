use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Accion;
use serde::Deserialize;

use crate::aplicacion::autorizar;
use crate::aplicacion::datos::{
    confirmar_importacion, exportar_datos, previsualizar_importacion, recibir_importacion,
};
use crate::bd::copia::resumir;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{MENU, PaginaDatos, PaginaImportarDatos, ResumenEnPantalla, renderizar};

#[derive(Deserialize)]
pub struct Aviso {
    importado: Option<String>,
}

pub async fn mostrar_datos(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(aviso): Query<Aviso>,
) -> Result<Response, ErrorApp> {
    autorizar(&actor, Accion::GestionarDatos)?;
    crate::aplicacion::demostracion::exigir_datos_reales(&estado)?;
    let resumen = estado.bd.leer(resumir).await?;
    let pagina = PaginaDatos {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "ajustes",
        pestana: "datos",
        resumen: ResumenEnPantalla::from(&resumen),
        confirmacion: aviso.importado.map(|_| {
            "Datos importados. Lo que había se guardó antes en la carpeta de copias.".to_owned()
        }),
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn descargar_copia(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    let copia = exportar_datos(&estado, &actor).await?;
    let adjunto = format!("attachment; filename=\"{}\"", copia.nombre);
    let cabeceras = [
        (CONTENT_TYPE, "application/octet-stream".to_owned()),
        (CONTENT_DISPOSITION, adjunto),
    ];
    Ok((cabeceras, copia.bytes).into_response())
}

/// Llega el archivo tal cual (lo envía `subir-copia.js`); un error vuelve como texto para mostrarlo.
pub async fn subir_copia(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    cuerpo: Bytes,
) -> Response {
    match recibir_importacion(&estado, &actor, cuerpo.to_vec()).await {
        Ok(clave) => Redirect::to(&format!("/ajustes/datos/importar/{clave}")).into_response(),
        Err(ErrorApp::Validacion(mensaje)) => {
            (StatusCode::UNPROCESSABLE_ENTITY, mensaje).into_response()
        }
        Err(otro) => otro.into_response(),
    }
}

pub async fn revisar_copia(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(clave): Path<String>,
) -> Result<Response, ErrorApp> {
    let vista = previsualizar_importacion(&estado, &actor, &clave).await?;
    let pagina = PaginaImportarDatos {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "ajustes",
        pestana: "datos",
        clave,
        actual: ResumenEnPantalla::from(&vista.actual),
        archivo: ResumenEnPantalla::from(&vista.archivo),
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn confirmar_copia(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(clave): Path<String>,
) -> Result<Response, ErrorApp> {
    confirmar_importacion(&estado, &actor, &clave).await?;
    Ok(Redirect::to("/ajustes/datos?importado=si").into_response())
}
