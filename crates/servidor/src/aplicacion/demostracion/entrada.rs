//! En la demostración se entra sin contraseña ni PIN: no protegen nada real y solo estorban para
//! probar el programa (decisión del propietario). Fuera de ella no existe: la base de verdad
//! siempre los pide.

use dominio::acceso::Rol;

use crate::aplicacion::sesion::{DuracionDeSesion, crear_sesion};
use crate::bd::consultas::usuarios;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Quién entra: el dueño de la demostración o un usuario elegido en su pantalla de entrada.
#[derive(Debug, Clone, Copy)]
pub enum QuienEntra {
    Dueno,
    Usuario(i64),
}

/// Abre una sesión en la demostración sin pedir clave. Devuelve el token y el rol, para llevar
/// al operario a «Mi día».
pub async fn entrar_sin_clave(
    demo: &EstadoApp,
    quien: QuienEntra,
) -> Result<(String, Rol), ErrorApp> {
    if !demo.es_demostracion() {
        return Err(ErrorApp::NoEncontrado);
    }
    let ahora = demo.ahora();
    demo.bd
        .escribir(move |tx| {
            let usuario = match quien {
                QuienEntra::Usuario(id) => usuarios::buscar_por_id(tx, id)?,
                QuienEntra::Dueno => usuarios::listar(tx, true)?
                    .into_iter()
                    .find(|u| u.rol == Rol::Propietario),
            }
            .filter(|u| u.activo)
            .ok_or(ErrorApp::NoEncontrado)?;
            let token = crear_sesion(tx, usuario.id, ahora, DuracionDeSesion::Normal)?;
            Ok((token, usuario.rol))
        })
        .await
}
