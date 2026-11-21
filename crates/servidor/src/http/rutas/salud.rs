use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::bd::{huella_del_esquema, leer_huella_guardada};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

#[derive(Serialize)]
pub struct Salud {
    estado: &'static str,
    version: &'static str,
    esquema: String,
}

pub async fn comprobar_salud(State(estado): State<EstadoApp>) -> Result<Json<Salud>, ErrorApp> {
    let guardada = estado.bd.leer(|con| Ok(leer_huella_guardada(con)?)).await?;
    let correcta = guardada.as_deref() == Some(huella_del_esquema().as_str());
    Ok(Json(Salud {
        estado: if correcta { "ok" } else { "esquema_inesperado" },
        version: env!("CARGO_PKG_VERSION"),
        esquema: guardada.unwrap_or_default().chars().take(12).collect(),
    }))
}
