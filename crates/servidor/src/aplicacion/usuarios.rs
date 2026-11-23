use dominio::acceso::{Accion, Permisos, Rol, validar_nombre_usuario, validar_pin};

use super::auditar::{Evento, auditar};
use super::primer_arranque::cifrar_en_segundo_plano;
use super::{Actor, autorizar};
use crate::bd::consultas::usuarios::{self, FilaUsuario, NuevoUsuario};
use crate::bd::consultas::{permisos, sesiones};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

#[derive(Debug, Clone)]
pub struct UsuarioVisible {
    pub id: i64,
    pub nombre: String,
    pub rol: Rol,
    pub activo: bool,
    /// Solo se rellenan en la lista de Ajustes > Usuarios.
    pub permisos: Permisos,
}

impl From<FilaUsuario> for UsuarioVisible {
    fn from(fila: FilaUsuario) -> Self {
        UsuarioVisible {
            id: fila.id,
            nombre: fila.nombre,
            rol: fila.rol,
            activo: fila.activo,
            permisos: Permisos::default(),
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
    estado
        .bd
        .leer(|con| {
            let mut lista = Vec::new();
            for fila in usuarios::listar(con, false)? {
                let permisos = permisos::leer(con, fila.id)?;
                lista.push(UsuarioVisible {
                    permisos,
                    ..UsuarioVisible::from(fila)
                });
            }
            Ok(lista)
        })
        .await
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

/// Lo que el dueño deja ver o hacer a un operario además de lo básico (ADR-0024). Vale desde
/// la siguiente página que abra: los permisos se leen en cada petición.
pub async fn guardar_permisos(
    estado: &EstadoApp,
    actor: &Actor,
    operario_id: i64,
    nuevos: Permisos,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarUsuarios)?;
    let (ahora, actor_id) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            buscar_operario(tx, operario_id)?;
            permisos::guardar(tx, operario_id, &nuevos)?;
            let evento = Evento {
                ahora,
                autor: Some(actor_id),
                accion: "cambiar_permisos",
                usuario: operario_id,
            };
            auditar(tx, evento)
        })
        .await
}

fn buscar_operario(con: &rusqlite::Connection, id: i64) -> Result<FilaUsuario, ErrorApp> {
    usuarios::buscar_por_id(con, id)?
        .filter(|u| u.rol == Rol::Operario)
        .ok_or(ErrorApp::NoEncontrado)
}
