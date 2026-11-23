//! El Inicio del propietario: tablero de trabajos y avisos (PANTALLAS 3, design/02-inicio).

use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::{Accion, Rol};

use super::avisos::reunir_avisos;
use crate::aplicacion::autorizar;
use crate::aplicacion::trabajos::consultar::listar_trabajos;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{MENU, PaginaInicio, renderizar, tablero};

pub async fn mostrar_inicio(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    if actor.rol == Rol::Operario {
        return Ok(Redirect::to("/mi-dia").into_response());
    }
    autorizar(&actor, Accion::VerPanelDelPropietario)?;
    let fichas = listar_trabajos(&estado, &actor, false).await?;
    let (columnas, falta_algo) = tablero(&fichas);
    let avisos = reunir_avisos(&estado, &actor, falta_algo).await?;
    let pagina = PaginaInicio {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "inicio",
        columnas,
        avisos: avisos.falta_algo,
        apuntes: avisos.apuntes,
        stock_bajo: avisos.stock_bajo,
        cobros_vencidos: avisos.cobros_vencidos,
        sin_respuesta: avisos.sin_respuesta,
        aviso_de_copias: avisos.copias,
    };
    Ok(renderizar(&pagina)?.into_response())
}
