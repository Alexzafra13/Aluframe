use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::{Accion, Rol};

use crate::aplicacion::autorizar;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{MENU, PaginaInicio, PaginaMiDia, PaginaPendiente, renderizar};

pub async fn mostrar_inicio(ActorActual(actor): ActorActual) -> Result<Response, ErrorApp> {
    if actor.rol == Rol::Operario {
        return Ok(Redirect::to("/mi-dia").into_response());
    }
    autorizar(&actor, Accion::VerPanelDelPropietario)?;
    let pagina = PaginaInicio {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "inicio",
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn mostrar_mi_dia(
    State(_): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    autorizar(&actor, Accion::VerMiDia)?;
    let pagina = PaginaMiDia {
        usuario: (&actor).into(),
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub struct SeccionPendiente {
    pub ruta: &'static str,
    pub seccion: &'static str,
    pub titulo: &'static str,
    pub hito: &'static str,
}

pub const SECCIONES_PENDIENTES: [SeccionPendiente; 5] = [
    pendiente("/clientes", "clientes", "Clientes", "H2"),
    pendiente("/presupuestos", "presupuestos", "Presupuestos", "H3"),
    pendiente("/trabajos", "trabajos", "Trabajos", "H4"),
    pendiente("/stock", "stock", "Stock", "H5"),
    pendiente("/facturas", "facturas", "Facturas", "H6"),
];

const fn pendiente(
    ruta: &'static str,
    seccion: &'static str,
    titulo: &'static str,
    hito: &'static str,
) -> SeccionPendiente {
    SeccionPendiente {
        ruta,
        seccion,
        titulo,
        hito,
    }
}

pub fn mostrar_pendiente(
    actor: &crate::aplicacion::Actor,
    seccion: &SeccionPendiente,
) -> Result<Response, ErrorApp> {
    autorizar(actor, Accion::VerPanelDelPropietario)?;
    let pagina = PaginaPendiente {
        usuario: actor.into(),
        menu: MENU,
        seccion: seccion.seccion,
        titulo: seccion.titulo,
        hito: seccion.hito,
    };
    Ok(renderizar(&pagina)?.into_response())
}
