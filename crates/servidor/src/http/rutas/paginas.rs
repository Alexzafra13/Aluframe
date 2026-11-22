use axum::response::{IntoResponse, Response};
use dominio::acceso::Accion;

use crate::aplicacion::autorizar;
use crate::errores::ErrorApp;
use crate::vistas::{MENU, PaginaPendiente, renderizar};

pub struct SeccionPendiente {
    pub ruta: &'static str,
    pub seccion: &'static str,
    pub titulo: &'static str,
    pub hito: &'static str,
}

pub const SECCIONES_PENDIENTES: [SeccionPendiente; 2] = [
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
