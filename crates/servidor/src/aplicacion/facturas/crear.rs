//! Crear el borrador: desde un trabajo (copia las líneas del presupuesto aceptado, con su precio
//! congelado), libre para un cliente, o rectificativa de una emitida (las mismas líneas en negativo).

use serde_json::Value;

use super::cambiar;
use super::editar::recalcular_totales;
use crate::aplicacion::Actor;
use crate::bd::consultas::facturas::{self, FilaLineaFactura, NuevaFactura};
use crate::bd::consultas::{clientes, presupuestos, trabajos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{anio_de_madrid, leer_utc};

fn anio(ahora: &str) -> i64 {
    leer_utc(ahora).map_or(2026, anio_de_madrid)
}

/// Una línea de la copia congelada del presupuesto, tal cual se aceptó.
fn linea_de_la_copia(l: &Value) -> FilaLineaFactura {
    let entero = |c: &str| l[c].as_i64().unwrap_or_default();
    let cantidad = entero("cantidad_mil").max(1);
    let base = entero("precio_cent");
    // Las ventanas guardan el precio de la línea; su unitario se deduce de él.
    let unitario = l["precio_unitario_pu"]
        .as_i64()
        .unwrap_or_else(|| (i128::from(base) * 100_000 / i128::from(cantidad)) as i64);
    FilaLineaFactura {
        id: 0,
        orden: 0,
        descripcion: l["descripcion"].as_str().unwrap_or_default().to_owned(),
        cantidad_mil: cantidad,
        precio_unitario_pu: unitario,
        descuento_bp: entero("descuento_bp"),
        tipo_iva_bp: entero("tipo_iva_bp"),
        base_cent: base,
    }
}

/// La factura ordinaria del trabajo; si ya tiene una, se abre esa en lugar de crear otra.
pub async fn factura_de_trabajo(
    estado: &EstadoApp,
    actor: &Actor,
    trabajo: i64,
) -> Result<i64, ErrorApp> {
    cambiar(
        estado,
        actor,
        ("crear_factura", "factura"),
        move |tx, ahora| {
            if let Some(f) = facturas::de_trabajo(tx, trabajo)?
                .into_iter()
                .find(|f| f.tipo == "ordinaria")
            {
                return Ok((f.id, f.id));
            }
            let t = trabajos::buscar(tx, trabajo)?.ok_or(ErrorApp::NoEncontrado)?;
            if t.estado == "anulado" {
                return Err(ErrorApp::Validacion("El trabajo está anulado.".into()));
            }
            let p = presupuestos::buscar(tx, t.presupuesto_id)?.ok_or(ErrorApp::NoEncontrado)?;
            let copia: Value =
                serde_json::from_str(p.copia_json.as_deref().unwrap_or("{}")).unwrap_or_default();
            let nueva = NuevaFactura {
                serie: "F",
                anio: anio(ahora),
                tipo: "ordinaria",
                rectifica: None,
                motivo: None,
                cliente: t.cliente_id,
                trabajo: Some(trabajo),
                descuento_bp: copia["descuento_bp"].as_i64().unwrap_or_default(),
                creado_en: ahora,
            };
            let id = facturas::insertar(tx, &nueva)?;
            for linea in copia["lineas"].as_array().into_iter().flatten() {
                facturas::insertar_linea(tx, id, &linea_de_la_copia(linea))?;
            }
            recalcular_totales(tx, id)?;
            Ok((id, id))
        },
    )
    .await
}

pub async fn factura_libre(
    estado: &EstadoApp,
    actor: &Actor,
    cliente: i64,
) -> Result<i64, ErrorApp> {
    cambiar(
        estado,
        actor,
        ("crear_factura", "factura"),
        move |tx, ahora| {
            clientes::buscar(tx, cliente)?.ok_or(ErrorApp::NoEncontrado)?;
            let nueva = NuevaFactura {
                serie: "F",
                anio: anio(ahora),
                tipo: "ordinaria",
                rectifica: None,
                motivo: None,
                cliente,
                trabajo: None,
                descuento_bp: 0,
                creado_en: ahora,
            };
            let id = facturas::insertar(tx, &nueva)?;
            Ok((id, id))
        },
    )
    .await
}

/// Rectificativa: apunta a la original y empieza con sus líneas en negativo (para anularla
/// entera); se pueden cambiar antes de emitirla.
pub async fn rectificativa(
    estado: &EstadoApp,
    actor: &Actor,
    original: i64,
    motivo: String,
) -> Result<i64, ErrorApp> {
    let motivo: String = motivo.trim().chars().take(300).collect();
    if motivo.is_empty() {
        return Err(ErrorApp::Validacion(
            "Escribe el motivo de la rectificación.".into(),
        ));
    }
    let permiso = ("crear_rectificativa", "factura");
    cambiar(estado, actor, permiso, move |tx, ahora| {
        let id = crear_rectificativa(tx, original, &motivo, ahora)?;
        Ok((id, id))
    })
    .await
}

fn crear_rectificativa(
    tx: &rusqlite::Connection,
    original: i64,
    motivo: &str,
    ahora: &str,
) -> Result<i64, ErrorApp> {
    let f = facturas::buscar(tx, original)?.ok_or(ErrorApp::NoEncontrado)?;
    if f.estado != "emitida" || f.tipo != "ordinaria" {
        return Err(ErrorApp::Validacion(
            "Solo se rectifica una factura ordinaria emitida.".into(),
        ));
    }
    let nueva = NuevaFactura {
        serie: "R",
        anio: anio(ahora),
        tipo: "rectificativa",
        rectifica: Some(original),
        motivo: Some(motivo),
        cliente: f.cliente_id,
        trabajo: f.trabajo_id,
        descuento_bp: f.descuento_bp,
        creado_en: ahora,
    };
    let id = facturas::insertar(tx, &nueva)?;
    for l in facturas::lineas(tx, original)? {
        let negativa = FilaLineaFactura {
            cantidad_mil: -l.cantidad_mil,
            base_cent: -l.base_cent,
            ..l
        };
        facturas::insertar_linea(tx, id, &negativa)?;
    }
    recalcular_totales(tx, id)?;
    Ok(id)
}
