use axum::Form;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Rol;
use serde::Deserialize;

use crate::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};
use crate::aplicacion::sesion::cerrar_sesion;
use crate::aplicacion::usuarios::listar_para_entrar;
use crate::autenticacion::{cookie_de_cierre, cookie_de_sesion};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::{ActorActual, IpCliente};
use crate::vistas::{OpcionUsuario, PaginaEntrar, renderizar};

#[derive(Deserialize)]
pub struct EleccionDeUsuario {
    usuario: Option<i64>,
}

#[derive(Deserialize)]
pub struct FormularioDeEntrada {
    usuario_id: i64,
    clave: String,
}

pub async fn mostrar_entrar(
    State(estado): State<EstadoApp>,
    Query(eleccion): Query<EleccionDeUsuario>,
) -> Result<Response, ErrorApp> {
    pagina_de_entrada(&estado, eleccion.usuario, None).await
}

pub async fn procesar_entrar(
    State(estado): State<EstadoApp>,
    IpCliente(ip): IpCliente,
    Form(formulario): Form<FormularioDeEntrada>,
) -> Result<Response, ErrorApp> {
    let usuario_id = formulario.usuario_id;
    let solicitud = SolicitudDeEntrada {
        usuario_id,
        clave: formulario.clave,
        ip,
    };
    let aviso = match entrar(&estado, solicitud).await? {
        ResultadoDeEntrada::Correcta { token, rol } => {
            let destino = if rol == Rol::Operario { "/mi-dia" } else { "/" };
            let cookie = [(SET_COOKIE, cookie_de_sesion(&token))];
            return Ok((cookie, Redirect::to(destino)).into_response());
        }
        ResultadoDeEntrada::Incorrecta => "El PIN o la contraseña no son correctos.".to_owned(),
        ResultadoDeEntrada::Bloqueada { minutos } => ErrorApp::Bloqueado { minutos }.to_string(),
    };
    let pagina = pagina_de_entrada(&estado, Some(usuario_id), Some(aviso)).await?;
    Ok((StatusCode::UNAUTHORIZED, pagina).into_response())
}

pub async fn salir(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    cerrar_sesion(&estado, &actor).await?;
    Ok(([(SET_COOKIE, cookie_de_cierre())], Redirect::to("/entrar")).into_response())
}

async fn pagina_de_entrada(
    estado: &EstadoApp,
    elegido: Option<i64>,
    aviso: Option<String>,
) -> Result<Response, ErrorApp> {
    let mut usuarios: Vec<OpcionUsuario> = listar_para_entrar(estado)
        .await?
        .into_iter()
        .map(OpcionUsuario::from)
        .collect();
    let elegido = elegido
        .and_then(|id| usuarios.iter().position(|u| u.id == id))
        .map(|indice| usuarios.remove(indice));
    let pagina = PaginaEntrar {
        usuarios,
        elegido,
        aviso,
    };
    Ok(renderizar(&pagina)?.into_response())
}
