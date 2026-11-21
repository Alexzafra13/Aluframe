use dominio::acceso::{Rol, validar_contrasena};

use super::auditar::{Evento, auditar};
use super::primer_arranque::cifrar_en_segundo_plano;
use crate::bd::consultas::{sesiones, usuarios};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Recuperación desde la consola del servidor: quien la usa ya tiene acceso a la máquina.
pub async fn restablecer_contrasena(
    estado: &EstadoApp,
    nombre: String,
    contrasena: String,
) -> Result<(), ErrorApp> {
    validar_contrasena(&contrasena)?;
    let clave_hash = cifrar_en_segundo_plano(contrasena).await?;
    let ahora = estado.ahora();
    estado
        .bd
        .escribir(move |tx| {
            let usuario = usuarios::buscar_por_nombre(tx, nombre.trim())?
                .filter(|u| u.rol == Rol::Propietario)
                .ok_or_else(|| {
                    ErrorApp::Validacion(format!("No hay ningún propietario llamado «{nombre}»."))
                })?;
            usuarios::cambiar_clave(tx, usuario.id, &clave_hash)?;
            usuarios::cambiar_activo(tx, usuario.id, true)?;
            sesiones::borrar_de_usuario(tx, usuario.id)?;
            auditar(
                tx,
                Evento {
                    ahora,
                    autor: None,
                    accion: "restablecer_contrasena",
                    usuario: usuario.id,
                },
            )
        })
        .await
}
