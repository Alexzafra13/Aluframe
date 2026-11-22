use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::bd::{version_de_la_base, version_del_programa};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

#[derive(Serialize)]
pub struct Salud {
    estado: &'static str,
    version: &'static str,
    esquema: u32,
}

pub async fn comprobar_salud(State(estado): State<EstadoApp>) -> Result<Json<Salud>, ErrorApp> {
    let base = estado.bd.leer(|con| Ok(version_de_la_base(con)?)).await?;
    let correcta = base == Some(version_del_programa());
    Ok(Json(Salud {
        estado: if correcta { "ok" } else { "esquema_inesperado" },
        version: env!("CARGO_PKG_VERSION"),
        esquema: base.unwrap_or_default(),
    }))
}
