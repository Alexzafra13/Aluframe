//! Logotipo de la empresa, guardado en `configuracion` como dirección `data:`: así entra en
//! las copias de seguridad y en la exportación sin archivos sueltos.

use dominio::acceso::Accion;
use dominio::imagen::convertir_en_direccion_de_datos;
use rusqlite::Connection;

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::configuracion::{self, Valor};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

const CLAVE: &str = "empresa.logo";

pub fn leer_logo(con: &Connection) -> Result<Option<String>, ErrorApp> {
    Ok(configuracion::leer(con, CLAVE)?.filter(|v| !v.is_empty()))
}

pub async fn leer_logo_de_empresa(estado: &EstadoApp) -> Result<Option<String>, ErrorApp> {
    estado.bd.leer(leer_logo).await
}

/// Sin bytes, quita el logotipo y la hoja vuelve a llevar el nombre en grande.
pub async fn cambiar_logo(
    estado: &EstadoApp,
    actor: &Actor,
    bytes: Option<Vec<u8>>,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarUsuarios)?;
    let valor = match bytes {
        Some(b) => convertir_en_direccion_de_datos(&b)
            .map_err(|e| ErrorApp::Validacion(format!("No se ha guardado: {e}.")))?,
        None => String::new(),
    };
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let cuando = formatear_utc(ahora);
            let nuevo = Valor {
                clave: CLAVE,
                valor: &valor,
                ahora: &cuando,
                usuario_id: autor,
            };
            configuracion::guardar(tx, &nuevo)?;
            let cambio = Cambio {
                ahora,
                autor,
                accion: "logo_de_empresa",
                entidad: "configuracion",
                entidad_id: 0,
            };
            anotar(tx, &cambio)
        })
        .await
}
