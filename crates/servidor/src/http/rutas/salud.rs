use axum::Json;
use axum::extract::State;
use serde::Serialize;

use crate::bd::ULTIMA_VERSION;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

#[derive(Serialize)]
pub struct Salud {
    estado: &'static str,
    version: &'static str,
    esquema: i64,
}

pub async fn comprobar_salud(State(estado): State<EstadoApp>) -> Result<Json<Salud>, ErrorApp> {
    let esquema = estado
        .bd
        .leer(|con| {
            Ok(
                con.query_row("SELECT max(version) FROM version_esquema", [], |fila| {
                    fila.get::<_, i64>(0)
                })?,
            )
        })
        .await?;
    let correcta = esquema == ULTIMA_VERSION;
    Ok(Json(Salud {
        estado: if correcta { "ok" } else { "esquema_inesperado" },
        version: env!("CARGO_PKG_VERSION"),
        esquema,
    }))
}
