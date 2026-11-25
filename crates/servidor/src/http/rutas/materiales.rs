//! Marcar el material de una ventana como pedido (o recibido, o volver atrás), desde la ficha del
//! trabajo o desde «Mi día» (ADR-0035).

use axum::Form;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use dominio::taller::EstadoDeMaterial;

use crate::aplicacion::trabajos::materiales::marcar_material;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;

pub async fn marcar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let nuevo = EstadoDeMaterial::desde_texto(&campos.texto("estado"))
        .ok_or_else(|| ErrorApp::Validacion("Estado desconocido.".into()))?;
    let version = campos.entero("version").unwrap_or(0);
    let nota = Some(campos.texto("nota"));
    let (trabajo, ventana) = marcar_material(&estado, &actor, (id, version), (nuevo, nota)).await?;
    let destino = if campos.texto("volver") == "mi-dia" {
        format!("/mi-dia/{ventana}#material-{ventana}")
    } else {
        format!("/trabajos/{trabajo}#material-{ventana}")
    };
    Ok(Redirect::to(&destino).into_response())
}
