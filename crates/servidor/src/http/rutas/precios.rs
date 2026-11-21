//! Edición de precios en la tabla de artículos y ajuste por porcentaje.

use axum::Form;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::unidades::Porcentaje;

use super::catalogo_articulos::pagina_de_articulos;
use crate::aplicacion::tarifas::{aplicar_ajuste, guardar_precios};
use crate::bd::consultas::articulos::FiltroArticulos;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;

fn proveedor(pares: &[(String, String)]) -> Result<i64, ErrorApp> {
    pares
        .iter()
        .find(|(clave, _)| clave == "proveedor")
        .and_then(|(_, valor)| valor.parse().ok())
        .ok_or(ErrorApp::NoEncontrado)
}

/// Campos `prefijo{id}` del formulario como pares (id, valor).
fn por_articulo(pares: &[(String, String)], prefijo: &str) -> Vec<(i64, String)> {
    pares
        .iter()
        .filter_map(|(clave, valor)| {
            Some((clave.strip_prefix(prefijo)?.parse().ok()?, valor.clone()))
        })
        .collect()
}

fn volver(proveedor: i64, resultado: &str) -> Response {
    Redirect::to(&format!(
        "/catalogo/articulos?proveedor={proveedor}&guardado={resultado}"
    ))
    .into_response()
}

/// Los errores se muestran en la misma lista, sin guardar nada y sin perder el filtro.
async fn con_aviso(
    estado: &EstadoApp,
    actor: &crate::aplicacion::Actor,
    proveedor: i64,
    aviso: String,
) -> Result<Response, ErrorApp> {
    let consulta = FiltroArticulos {
        proveedor_id: Some(proveedor),
        texto: String::new(),
        archivados: false,
    };
    pagina_de_articulos(estado, actor, consulta, (Some(aviso), None)).await
}

pub async fn guardar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let proveedor = proveedor(&pares)?;
    match guardar_precios(&estado, &actor, proveedor, por_articulo(&pares, "precio_")).await {
        Ok(0) => Ok(volver(proveedor, "sin-cambios")),
        Ok(_) => Ok(volver(proveedor, "precios")),
        Err(ErrorApp::Validacion(aviso)) => con_aviso(&estado, &actor, proveedor, aviso).await,
        Err(otro) => Err(otro),
    }
}

pub async fn ajustar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let proveedor = proveedor(&pares)?;
    let texto = pares
        .iter()
        .find(|(c, _)| c == "porcentaje")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    let porcentaje = match Porcentaje::desde_texto_es(texto) {
        Ok(p) => p,
        Err(e) => return con_aviso(&estado, &actor, proveedor, format!("Porcentaje: {e}")).await,
    };
    let marcados = por_articulo(&pares, "sel_")
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    match aplicar_ajuste(&estado, &actor, (proveedor, marcados), porcentaje).await {
        Ok(_) => Ok(volver(proveedor, "ajuste")),
        Err(ErrorApp::Validacion(aviso)) => con_aviso(&estado, &actor, proveedor, aviso).await,
        Err(otro) => Err(otro),
    }
}
