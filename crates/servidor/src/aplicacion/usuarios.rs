use dominio::acceso::{Accion, Rol, validar_nombre_usuario, validar_pin};

use super::auditar::{Evento, auditar};
use super::primer_arranque::cifrar_en_segundo_plano;
use super::{Actor, autorizar};
use crate::bd::consultas::sesiones;
use crate::bd::consultas::usuarios::{self, FilaUsuario, NuevoUsuario};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

#[derive(Debug, Clone)]
pub struct UsuarioVisible {
    pub id: i64,
    pub nombre: String,
    pub rol: Rol,
    pub activo: bool,
}

impl From<FilaUsuario> for UsuarioVisible {
    fn from(fila: FilaUsuario) -> Self {
        UsuarioVisible {
            id: fila.id,
            nombre: fila.nombre,
            rol: fila.rol,
            activo: fila.activo,
        }
    }
}

pub async fn listar_para_entrar(estado: &EstadoApp) -> Result<Vec<UsuarioVisible>, ErrorApp> {
    let filas = estado.bd.leer(|con| usuarios::listar(con, true)).await?;
    Ok(filas.into_iter().map(UsuarioVisible::from).collect())
}

pub async fn listar_usuarios(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<UsuarioVisible>, ErrorApp> {
    autorizar(actor, Accion::GestionarUsuarios)?;
    let filas = estado.bd.leer(|con| usuarios::listar(con, false)).await?;
    Ok(filas.into_iter().map(UsuarioVisible::from).collect())
}

pub async fn crear_operario(
    estado: &EstadoApp,
    actor: &Actor,
    nombre: &str,
    pin: String,
) -> Result<i64, ErrorApp> {
    autorizar(actor, Accion::GestionarUsuarios)?;
    let nombre = validar_nombre_usuario(nombre)?;
    validar_pin(&pin)?;
    let clave_hash = cifrar_en_segundo_plano(pin).await?;
    let (ahora, actor_id) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            if usuarios::existe_nombre(tx, &nombre)? {
                return Err(ErrorApp::Validacion(format!(
                    "Ya hay un usuario llamado «{nombre}»."
                )));
            }
            let nuevo = NuevoUsuario {
                nombre: &nombre,
                rol: Rol::Operario,
                clave_hash: &clave_hash,
                creado_en: &formatear_utc(ahora),
            };
            let id = usuarios::insertar(tx, &nuevo)?;
            auditar(
                tx,
                Evento {
                    ahora,
                    autor: Some(actor_id),
                    accion: "crear_operario",
                    usuario: id,
                },
            )?;
            Ok(id)
        })
        .await
}

pub async fn restablecer_pin(
    estado: &EstadoApp,
    actor: &Actor,
    operario_id: i64,
    pin: String,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarUsuarios)?;
    validar_pin(&pin)?;
    let clave_hash = cifrar_en_segundo_plano(pin).await?;
    let (ahora, actor_id) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            buscar_operario(tx, operario_id)?;
            usuarios::cambiar_clave(tx, operario_id, &clave_hash)?;
            sesiones::borrar_de_usuario(tx, operario_id)?;
            auditar(
                tx,
                Evento {
                    ahora,
                    autor: Some(actor_id),
                    accion: "restablecer_pin",
                    usuario: operario_id,
                },
            )
        })
        .await
}

pub async fn cambiar_activo(
    estado: &EstadoApp,
    actor: &Actor,
    operario_id: i64,
    activo: bool,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarUsuarios)?;
    let (ahora, actor_id) = (estado.ahora(), actor.id);
    let accion = if activo {
        "activar_usuario"
    } else {
        "desactivar_usuario"
    };
    estado
        .bd
        .escribir(move |tx| {
            buscar_operario(tx, operario_id)?;
            usuarios::cambiar_activo(tx, operario_id, activo)?;
            if !activo {
                sesiones::borrar_de_usuario(tx, operario_id)?;
            }
            auditar(
                tx,
                Evento {
                    ahora,
                    autor: Some(actor_id),
                    accion,
                    usuario: operario_id,
                },
            )
        })
        .await
}

fn buscar_operario(con: &rusqlite::Connection, id: i64) -> Result<FilaUsuario, ErrorApp> {
    usuarios::buscar_por_id(con, id)?
        .filter(|u| u.rol == Rol::Operario)
        .ok_or(ErrorApp::NoEncontrado)
}
