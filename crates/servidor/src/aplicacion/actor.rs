use dominio::acceso::{Accion, Permisos, Rol, puede_realizar};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct Actor {
    pub id: i64,
    pub nombre: String,
    pub rol: Rol,
    /// Los extra que el dueño le ha dado (ADR-0024); al dueño no le hacen falta.
    pub permisos: Permisos,
    pub token_csrf: String,
    pub huella_sesion: String,
    /// Está en la demostración (ADR-0028): las pantallas lo avisan.
    pub en_demostracion: bool,
}

/// Lo que no está permitido se responde como no encontrado, para no revelar que existe.
pub fn autorizar(actor: &Actor, accion: Accion) -> Result<(), ErrorApp> {
    if actor.puede(accion) {
        Ok(())
    } else {
        Err(ErrorApp::NoEncontrado)
    }
}

impl Actor {
    /// Para decidir qué se enseña (menú, botones); lo que se puede hacer lo decide `autorizar`.
    pub fn puede(&self, accion: Accion) -> bool {
        puede_realizar(self.rol, &self.permisos, accion)
    }
}
