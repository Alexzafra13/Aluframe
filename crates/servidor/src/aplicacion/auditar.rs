use rusqlite::Connection;
use time::OffsetDateTime;

use crate::bd::consultas::auditoria::{Apunte, registrar};
use crate::errores::ErrorApp;
use crate::fecha::formatear_utc;

pub struct Evento<'a> {
    pub ahora: OffsetDateTime,
    pub autor: Option<i64>,
    pub accion: &'a str,
    pub usuario: i64,
}

pub fn auditar(con: &Connection, evento: Evento) -> Result<(), ErrorApp> {
    registrar(
        con,
        &Apunte {
            cuando_en: &formatear_utc(evento.ahora),
            usuario_id: evento.autor,
            accion: evento.accion,
            entidad: "usuario",
            entidad_id: Some(evento.usuario),
        },
    )
}

/// Apunte de cualquier entidad del catálogo o de clientes.
pub struct Cambio<'a> {
    pub ahora: OffsetDateTime,
    pub autor: i64,
    pub accion: &'a str,
    pub entidad: &'a str,
    pub entidad_id: i64,
}

pub fn anotar(con: &Connection, cambio: &Cambio) -> Result<(), ErrorApp> {
    registrar(
        con,
        &Apunte {
            cuando_en: &formatear_utc(cambio.ahora),
            usuario_id: Some(cambio.autor),
            accion: cambio.accion,
            entidad: cambio.entidad,
            entidad_id: Some(cambio.entidad_id),
        },
    )
}
