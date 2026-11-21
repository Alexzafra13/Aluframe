use dominio::acceso::{Rol, validar_contrasena, validar_nombre_usuario};
use rusqlite::Connection;
use time::OffsetDateTime;

use super::auditar::{Evento, auditar};
use super::sesion::crear_sesion;
use crate::autenticacion::cifrar_clave;
use crate::bd::consultas::usuarios::{self, NuevoUsuario};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

pub struct DatosDelPropietario {
    pub nombre: String,
    pub contrasena: String,
    pub confirmacion: String,
}

/// Solo funciona con la base vacía: crea el propietario y devuelve el token de su sesión.
pub async fn crear_propietario_inicial(
    estado: &EstadoApp,
    datos: DatosDelPropietario,
) -> Result<String, ErrorApp> {
    let nombre = validar_nombre_usuario(&datos.nombre)?;
    validar_contrasena(&datos.contrasena)?;
    if datos.contrasena != datos.confirmacion {
        return Err(ErrorApp::Validacion(
            "Las dos contraseñas no coinciden.".into(),
        ));
    }
    let clave_hash = cifrar_en_segundo_plano(datos.contrasena).await?;
    let ahora = estado.ahora();
    let token = estado
        .bd
        .escribir(move |tx| insertar_propietario(tx, &nombre, &clave_hash, ahora))
        .await?;
    estado.anotar_que_hay_usuarios();
    Ok(token)
}

fn insertar_propietario(
    tx: &Connection,
    nombre: &str,
    clave_hash: &str,
    ahora: OffsetDateTime,
) -> Result<String, ErrorApp> {
    if usuarios::contar_usuarios(tx)? > 0 {
        return Err(ErrorApp::Validacion(
            "AlumFrame ya está configurado.".into(),
        ));
    }
    let nuevo = NuevoUsuario {
        nombre,
        rol: Rol::Propietario,
        clave_hash,
        creado_en: &formatear_utc(ahora),
    };
    let id = usuarios::insertar(tx, &nuevo)?;
    let evento = Evento {
        ahora,
        autor: Some(id),
        accion: "crear_propietario",
        usuario: id,
    };
    auditar(tx, evento)?;
    crear_sesion(tx, id, ahora)
}

pub async fn cifrar_en_segundo_plano(clave: String) -> Result<String, ErrorApp> {
    tokio::task::spawn_blocking(move || cifrar_clave(&clave))
        .await
        .map_err(|error| ErrorApp::Interno(format!("tarea interrumpida: {error}")))?
}
