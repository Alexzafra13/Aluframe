//! Ajustes > Copias (H7).

use axum::Form;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::copias::pantalla::{
    consultar_copias, copiar_ahora, preparar_disco, restaurar_copia,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{PaginaCopias, renderizar};

#[derive(Deserialize)]
pub struct Aviso {
    aviso: Option<String>,
}

/// Solo claves fijas en la URL: el texto se elige aquí.
fn texto_del_aviso(clave: &str) -> Option<String> {
    let texto = match clave {
        "hecha" => "Copia hecha y comprobada.",
        "preparado" => "Disco preparado: las copias se guardarán también en él.",
        _ => return None,
    };
    Some(texto.to_owned())
}

pub async fn mostrar_copias(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(aviso): Query<Aviso>,
) -> Result<Response, ErrorApp> {
    let confirmacion = aviso.aviso.as_deref().and_then(texto_del_aviso);
    pagina(&estado, &actor, (confirmacion, None))
}

fn pagina(
    estado: &EstadoApp,
    actor: &Actor,
    avisos: (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let copias = consultar_copias(estado, actor)?;
    Ok(renderizar(&PaginaCopias::nueva(actor.into(), &copias, avisos))?.into_response())
}

pub async fn copiar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    copiar_ahora(&estado, &actor).await?;
    Ok(Redirect::to("/ajustes/copias?aviso=hecha").into_response())
}

#[derive(Deserialize)]
pub struct FormularioDisco {
    raiz: String,
}

pub async fn preparar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(formulario): Form<FormularioDisco>,
) -> Result<Response, ErrorApp> {
    match preparar_disco(&estado, &actor, &formulario.raiz) {
        Ok(()) => Ok(Redirect::to("/ajustes/copias?aviso=preparado").into_response()),
        Err(ErrorApp::Validacion(texto)) => pagina(&estado, &actor, (None, Some(texto))),
        Err(error) => Err(error),
    }
}

#[derive(Deserialize)]
pub struct FormularioRestaurar {
    disco: String,
    nombre: String,
}

pub async fn restaurar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(formulario): Form<FormularioRestaurar>,
) -> Result<Response, ErrorApp> {
    let elegida = (formulario.disco.as_str(), formulario.nombre.as_str());
    match restaurar_copia(&estado, &actor, elegida).await {
        Ok(clave) => Ok(Redirect::to(&format!("/ajustes/datos/importar/{clave}")).into_response()),
        Err(ErrorApp::Validacion(texto)) => pagina(&estado, &actor, (None, Some(texto))),
        Err(error) => Err(error),
    }
}
