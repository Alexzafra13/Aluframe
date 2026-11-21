use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::usuarios::{
    cambiar_activo, crear_operario, listar_usuarios, restablecer_pin,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{MENU, OpcionUsuario, PaginaUsuarios, renderizar};

#[derive(Deserialize)]
pub struct Aviso {
    hecho: Option<String>,
}

#[derive(Deserialize)]
pub struct FormularioOperario {
    nombre: String,
    pin: String,
}

#[derive(Deserialize)]
pub struct FormularioPin {
    pin: String,
}

pub async fn mostrar_usuarios(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(aviso): Query<Aviso>,
) -> Result<Response, ErrorApp> {
    let confirmacion = aviso.hecho.as_deref().map(texto_de_confirmacion);
    pagina_de_usuarios(&estado, &actor, None, confirmacion).await
}

pub async fn alta_de_operario(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(formulario): Form<FormularioOperario>,
) -> Result<Response, ErrorApp> {
    let resultado = crear_operario(&estado, &actor, &formulario.nombre, formulario.pin).await;
    volver(&estado, &actor, resultado.map(|_| "alta")).await
}

pub async fn cambio_de_pin(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(formulario): Form<FormularioPin>,
) -> Result<Response, ErrorApp> {
    let resultado = restablecer_pin(&estado, &actor, id, formulario.pin).await;
    volver(&estado, &actor, resultado.map(|()| "pin")).await
}

pub async fn desactivacion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let resultado = cambiar_activo(&estado, &actor, id, false).await;
    volver(&estado, &actor, resultado.map(|()| "desactivado")).await
}

pub async fn activacion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let resultado = cambiar_activo(&estado, &actor, id, true).await;
    volver(&estado, &actor, resultado.map(|()| "activado")).await
}

async fn volver(
    estado: &EstadoApp,
    actor: &Actor,
    resultado: Result<&str, ErrorApp>,
) -> Result<Response, ErrorApp> {
    match resultado {
        Ok(hecho) => Ok(Redirect::to(&format!("/ajustes/usuarios?hecho={hecho}")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            let pagina = pagina_de_usuarios(estado, actor, Some(aviso), None).await?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, pagina).into_response())
        }
        Err(otro) => Err(otro),
    }
}

async fn pagina_de_usuarios(
    estado: &EstadoApp,
    actor: &Actor,
    aviso: Option<String>,
    confirmacion: Option<String>,
) -> Result<Response, ErrorApp> {
    let usuarios = listar_usuarios(estado, actor).await?;
    let pagina = PaginaUsuarios {
        usuario: actor.into(),
        menu: MENU,
        seccion: "ajustes",
        usuarios: usuarios.into_iter().map(OpcionUsuario::from).collect(),
        aviso,
        confirmacion,
    };
    Ok(renderizar(&pagina)?.into_response())
}

fn texto_de_confirmacion(hecho: &str) -> String {
    match hecho {
        "alta" => "Operario dado de alta.",
        "pin" => "PIN cambiado. El operario tendrá que volver a entrar.",
        "desactivado" => "Usuario desactivado.",
        "activado" => "Usuario activado.",
        _ => "Hecho.",
    }
    .to_owned()
}
