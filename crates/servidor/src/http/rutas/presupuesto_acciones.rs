use axum::Form;
use axum::extract::{Path, State};
use axum::response::Response;
use dominio::unidades::{Decimal, Porcentaje, PrecioUnitario, leer_decimal_es};
use serde::Deserialize;

use super::presupuestos::{a_la_ficha, pagina};
use crate::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use crate::aplicacion::lineas_de_presupuesto::{
    LineaLibre, borrar_linea, guardar_libre, recalcular_precios,
};
use crate::aplicacion::presupuestos::{
    DatosDeCabecera, cambiar_estado, guardar_cabecera, nueva_version,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;

/// Los errores de validación se muestran en la propia ficha, sin perder el contexto.
async fn o_en_la_ficha(
    estado: &EstadoApp,
    actor: &crate::aplicacion::Actor,
    id: i64,
    resultado: Result<&str, ErrorApp>,
) -> Result<Response, ErrorApp> {
    match resultado {
        Ok(aviso) => Ok(a_la_ficha(id, aviso)),
        Err(error @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
            pagina(estado, actor, id, (Some(error.to_string()), None)).await
        }
        Err(otro) => Err(otro),
    }
}

pub fn leer_porcentaje(texto: &str, nombre: &str) -> Result<Porcentaje, ErrorApp> {
    if texto.trim().is_empty() {
        return Ok(Porcentaje::CERO);
    }
    Porcentaje::desde_texto_es(texto)
        .ok()
        .filter(|p| (0..=10_000).contains(&p.puntos_basicos()))
        .ok_or_else(|| {
            ErrorApp::Validacion(format!("{nombre}: escribe un porcentaje entre 0 y 100."))
        })
}

#[derive(Deserialize)]
pub struct DatosFormulario {
    version: i64,
    direccion: Option<String>,
    validez: Option<String>,
    descuento: Option<String>,
    condiciones: Option<String>,
}

pub async fn guardar_datos(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(f): Form<DatosFormulario>,
) -> Result<Response, ErrorApp> {
    let resultado = async {
        let datos = DatosDeCabecera {
            direccion_montaje_id: f.direccion.and_then(|d| d.parse().ok()),
            validez_hasta: f.validez.filter(|v| v.len() == 10),
            condiciones: f
                .condiciones
                .map(|c| c.trim().chars().take(4_000).collect::<String>())
                .filter(|c| !c.is_empty()),
            descuento_bp: leer_porcentaje(f.descuento.as_deref().unwrap_or(""), "Descuento")?
                .puntos_basicos(),
        };
        guardar_cabecera(&estado, &actor, (id, f.version), datos)
            .await
            .map(|()| "datos")
    }
    .await;
    o_en_la_ficha(&estado, &actor, id, resultado).await
}

pub async fn cambiar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, nuevo)): Path<(i64, String)>,
) -> Result<Response, ErrorApp> {
    let (destino, aviso) = match nuevo.as_str() {
        "enviado" => ("enviado", "enviado"),
        "rechazado" => ("rechazado", "rechazado"),
        _ => return Err(ErrorApp::NoEncontrado),
    };
    let resultado = cambiar_estado(&estado, &actor, id, destino)
        .await
        .map(|()| aviso);
    o_en_la_ficha(&estado, &actor, id, resultado).await
}

#[derive(Deserialize)]
pub struct Motivo {
    motivo: Option<String>,
}

pub async fn version(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(m): Form<Motivo>,
) -> Result<Response, ErrorApp> {
    let motivo = m
        .motivo
        .map(|t| t.trim().chars().take(200).collect::<String>())
        .filter(|t| !t.is_empty());
    let resultado = nueva_version(&estado, &actor, id, motivo)
        .await
        .map(|()| "version");
    o_en_la_ficha(&estado, &actor, id, resultado).await
}

pub async fn aceptar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let resultado = aceptar_presupuesto(&estado, &actor, id)
        .await
        .map(|()| "aceptado");
    o_en_la_ficha(&estado, &actor, id, resultado).await
}

pub async fn recalcular(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let resultado = recalcular_precios(&estado, &actor, id)
        .await
        .map(|()| "recalculado");
    o_en_la_ficha(&estado, &actor, id, resultado).await
}

pub async fn quitar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, orden)): Path<(i64, i64)>,
) -> Result<Response, ErrorApp> {
    let resultado = borrar_linea(&estado, &actor, (id, orden))
        .await
        .map(|()| "quitada");
    o_en_la_ficha(&estado, &actor, id, resultado).await
}

#[derive(Deserialize)]
pub struct Libre {
    descripcion: String,
    cantidad: String,
    precio: String,
    descuento: Option<String>,
    iva: i64,
}

fn leer_libre(f: Libre) -> Result<LineaLibre, ErrorApp> {
    let descripcion: String = f.descripcion.trim().chars().take(300).collect();
    if descripcion.is_empty() {
        return Err(ErrorApp::Validacion(
            "Escribe la descripción de la línea libre.".into(),
        ));
    }
    let cantidad_mil = leer_decimal_es(&f.cantidad)
        .ok()
        .and_then(|c| c.multiplicar(Decimal::desde_entero(1_000)).ok())
        .and_then(Decimal::a_entero)
        .filter(|c| *c > 0)
        .ok_or_else(|| {
            ErrorApp::Validacion("La cantidad debe ser mayor que cero (hasta 3 decimales).".into())
        })?;
    let precio_unitario = PrecioUnitario::desde_texto_es(&f.precio)
        .ok()
        .filter(|p| p.diezmilesimas() >= 0)
        .ok_or_else(|| {
            ErrorApp::Validacion(
                "Escribe el precio unitario en euros, por ejemplo 80 o 12,50.".into(),
            )
        })?;
    let descuento = leer_porcentaje(f.descuento.as_deref().unwrap_or(""), "Descuento")?;
    let iva = Porcentaje::desde_puntos_basicos(f.iva.clamp(0, 10_000));
    Ok(LineaLibre {
        descripcion,
        cantidad_mil,
        precio_unitario,
        descuento,
        iva,
    })
}

pub async fn libre(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(f): Form<Libre>,
) -> Result<Response, ErrorApp> {
    let resultado = match leer_libre(f) {
        Ok(linea) => guardar_libre(&estado, &actor, (id, 0), linea)
            .await
            .map(|_| "linea"),
        Err(error) => Err(error),
    };
    o_en_la_ficha(&estado, &actor, id, resultado).await
}
