use dominio::acceso::{Accion, Rol, puede_realizar};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct Actor {
    pub id: i64,
    pub nombre: String,
    pub rol: Rol,
    pub token_csrf: String,
    pub huella_sesion: String,
}

/// Lo que no está permitido se responde como no encontrado, para no revelar que existe.
pub fn autorizar(actor: &Actor, accion: Accion) -> Result<(), ErrorApp> {
    if puede_realizar(actor.rol, accion) {
        Ok(())
    } else {
        Err(ErrorApp::NoEncontrado)
    }
}
