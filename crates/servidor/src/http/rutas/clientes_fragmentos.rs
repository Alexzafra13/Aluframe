use axum::Json;
use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use dominio::acceso::Accion;
use dominio::clientes::EntradaDeDireccion;
use dominio::nif::Nif;
use serde::Deserialize;

use crate::aplicacion::autorizar;
use crate::aplicacion::poblaciones::{
    Poblacion, SugerenciaDeLocalidad, buscar_por_codigo_postal, buscar_por_localidad,
    sugerir_localidades,
};
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

#[derive(Deserialize)]
pub struct ConsultaDePoblacion {
    cp: Option<String>,
    localidad: Option<String>,
}

/// Para los formularios con dirección: lo que se puede rellenar a partir de lo escrito.
pub async fn buscar_poblacion(
    ActorActual(_actor): ActorActual,
    Query(consulta): Query<ConsultaDePoblacion>,
) -> Json<Poblacion> {
    let cp = consulta.cp.unwrap_or_default();
    let poblacion = if cp.trim().is_empty() {
        buscar_por_localidad(consulta.localidad.as_deref().unwrap_or_default())
    } else {
        buscar_por_codigo_postal(&cp)
    };
    Json(poblacion)
}

#[derive(Deserialize)]
pub struct ConsultaDeSugerencias {
    texto: Option<String>,
}

/// Las localidades más parecidas a lo que se va escribiendo, para la lista del campo.
pub async fn sugerir_poblaciones(
    ActorActual(_actor): ActorActual,
    Query(consulta): Query<ConsultaDeSugerencias>,
) -> Json<Vec<SugerenciaDeLocalidad>> {
    Json(sugerir_localidades(
        consulta.texto.as_deref().unwrap_or_default(),
    ))
}
