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
