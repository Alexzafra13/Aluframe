use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use dominio::acceso::Accion;
use dominio::clientes::EntradaDeDireccion;
use dominio::nif::Nif;
use serde::Deserialize;

use crate::aplicacion::autorizar;
use crate::errores::ErrorApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{
    BloqueDireccion, ErroresDeCampo, FragmentoDireccion, FragmentoNif, renderizar,
};

pub fn bloque_vacio(n: usize) -> BloqueDireccion {
    BloqueDireccion {
        n,
        valores: EntradaDeDireccion::default(),
        errores: ErroresDeCampo::default(),
    }
}

#[derive(Deserialize)]
pub struct NifEscrito {
    nif: Option<String>,
}

pub async fn comprobar_nif(
    ActorActual(actor): ActorActual,
    Query(escrito): Query<NifEscrito>,
) -> Result<Response, ErrorApp> {
    autorizar(&actor, Accion::GestionarClientes)?;
    let texto = escrito.nif.unwrap_or_default();
    let (mensaje, correcto) = if texto.trim().is_empty() {
        (None, false)
    } else {
        match Nif::desde_texto(&texto) {
            Ok(nif) => (Some(format!("Correcto: {nif}")), true),
            Err(error) => (Some(error.to_string()), false),
        }
    };
    Ok(renderizar(&FragmentoNif { mensaje, correcto })?.into_response())
}

#[derive(Deserialize)]
pub struct NumeroDeBloque {
    n: Option<usize>,
}

pub async fn direccion_en_blanco(
    ActorActual(actor): ActorActual,
    Query(numero): Query<NumeroDeBloque>,
) -> Result<Response, ErrorApp> {
    autorizar(&actor, Accion::GestionarClientes)?;
    let n = numero.n.unwrap_or(1).min(19);
    let direccion = bloque_vacio(n);
    Ok(renderizar(&FragmentoDireccion { direccion })?.into_response())
}
