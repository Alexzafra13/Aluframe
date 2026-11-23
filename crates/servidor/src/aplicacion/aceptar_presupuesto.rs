//! Aceptar congela el presupuesto: se guarda una copia con todo lo que contaba (cliente, líneas,
//! cálculos, precios, versiones de serie y de tarifa) y su SHA-256 (DATOS, apartado 4). En la
//! misma transacción nace el trabajo del taller (H4).

use dominio::presupuesto::Situacion;
use serde::Serialize;

use super::Actor;
use super::ajustes_presupuesto;
use super::lineas_de_presupuesto::a_json;
use super::presupuestos::{FichaDePresupuesto, cambiar, cargar};
use super::resumen_de_linea::ResumenDeVentana;
use crate::autenticacion::calcular_sha256;
use crate::bd::consultas::clientes::FilaCliente;
use crate::bd::consultas::presupuestos;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fechas_de_situacion;

#[derive(Serialize)]
struct CopiaDeLinea {
    orden: i64,
    tipo: String,
    descripcion: String,
    cantidad_mil: i64,
    serie_version_id: Option<i64>,
    ancho_dmm: Option<i64>,
    alto_dmm: Option<i64>,
    opciones_json: String,
    resultado: Option<ResumenDeVentana>,
    tarifas: Vec<i64>,
    coste_cent: i64,
    mano_de_obra_cent: i64,
    descuento_bp: i64,
    tipo_iva_bp: i64,
    precio_unitario_pu: Option<i64>,
    precio_cent: i64,
}

#[derive(Serialize)]
struct CopiaDePresupuesto<'a> {
    codigo: String,
    numero_version: i64,
    aceptado_en: &'a str,
    empresa: Vec<String>,
    cliente: CopiaDeCliente,
    direccion_montaje: Option<String>,
    validez_hasta: Option<String>,
    condiciones: Option<String>,
    descuento_bp: i64,
    lineas: Vec<CopiaDeLinea>,
    base_cent: i64,
    iva_cent: i64,
    total_cent: i64,
}

#[derive(Serialize)]
struct CopiaDeCliente {
    numero: i64,
    tipo: String,
    nombre: Option<String>,
    nif: Option<String>,
    email: Option<String>,
    telefono: Option<String>,
    direccion_fiscal: Vec<String>,
}

impl From<&FilaCliente> for CopiaDeCliente {
    fn from(c: &FilaCliente) -> Self {
        CopiaDeCliente {
            numero: c.id,
            tipo: c.tipo.clone(),
            nombre: c.nombre.clone(),
            nif: c.nif.clone(),
            email: c.email.clone(),
            telefono: c.telefono.clone(),
            direccion_fiscal: [
                &c.fiscal_calle,
                &c.fiscal_cp,
                &c.fiscal_localidad,
                &c.fiscal_provincia,
                &c.fiscal_pais,
            ]
            .into_iter()
            .flatten()
            .cloned()
            .collect(),
        }
    }
}

fn motivo_de_caducidad(ficha: &FichaDePresupuesto, fechas: &(String, String)) -> Option<String> {
    if !matches!(
        ficha.situacion(fechas),
        Situacion::Caducado | Situacion::Archivado
    ) {
        return None;
    }
    let hasta = ficha
        .presupuesto
        .validez_hasta
        .as_deref()
        .unwrap_or_default();
    let dia: Vec<&str> = hasta.split('-').rev().collect();
    Some(format!(
        "su validez terminó el {}: pulsa «Renovar» para poner los precios de hoy",
        dia.join("/")
    ))
}

/// Por qué no se puede aceptar todavía; vacío si se puede (el botón lo muestra desactivado).
/// `fechas` sale de `fecha::fechas_de_situacion`: uno caducado se renueva antes (ADR-0027).
pub fn motivos_para_no_aceptar(
    ficha: &FichaDePresupuesto,
    fechas: &(String, String),
) -> Vec<String> {
    let mut motivos = Vec::new();
    if !matches!(ficha.presupuesto.estado.as_str(), "borrador" | "enviado") {
        motivos.push(format!("el presupuesto está {}", ficha.presupuesto.estado));
    } else if let Some(caducado) = motivo_de_caducidad(ficha, fechas) {
        motivos.push(caducado);
    }
    if ficha.lineas.is_empty() {
        motivos.push("no tiene líneas".into());
    }
    for linea in &ficha.lineas {
        if linea.incompleta {
            let faltan: Vec<String> = serde_json::from_str(&linea.faltan_json).unwrap_or_default();
            let primero = faltan
                .first()
                .cloned()
                .unwrap_or_else(|| "está incompleta".into());
            motivos.push(format!("línea {}: {}", linea.orden, primero));
        }
        if linea.revisar_limites {
            motivos.push(format!(
                "línea {}: medidas fuera de los límites sin revisar",
                linea.orden
            ));
        }
    }
    if ficha.tarifas_cambiadas {
        motivos.push("la tarifa ha cambiado: recalcula los precios".into());
    }
    motivos
}

fn copia<'a>(
    ficha: &FichaDePresupuesto,
    empresa: Vec<String>,
    ahora: &'a str,
) -> CopiaDePresupuesto<'a> {
    let p = &ficha.presupuesto;
    let direccion = ficha
        .direcciones
        .iter()
        .find(|d| Some(d.id) == p.direccion_montaje_id)
        .map(|d| d.datos.resumen());
    CopiaDePresupuesto {
        codigo: p.codigo(),
        numero_version: p.numero_version,
        aceptado_en: ahora,
        empresa,
        cliente: CopiaDeCliente::from(&ficha.cliente),
        direccion_montaje: direccion,
        validez_hasta: p.validez_hasta.clone(),
        condiciones: p.condiciones.clone(),
        descuento_bp: p.descuento_bp,
        lineas: ficha.lineas.iter().map(copia_de_linea).collect(),
        base_cent: p.base_cent,
        iva_cent: p.iva_cent,
        total_cent: p.total_cent,
    }
}

fn copia_de_linea(l: &crate::bd::consultas::lineas_presupuesto::FilaLinea) -> CopiaDeLinea {
    CopiaDeLinea {
        orden: l.orden,
        tipo: l.tipo.clone(),
        descripcion: l.descripcion.clone(),
        cantidad_mil: l.cantidad_mil,
        serie_version_id: l.serie_version_id,
        ancho_dmm: l.ancho_dmm,
        alto_dmm: l.alto_dmm,
        opciones_json: l.opciones_json.clone(),
        resultado: l
            .resultado_json
            .as_deref()
            .map(|j| ResumenDeVentana::leer(Some(j))),
        tarifas: serde_json::from_str(&l.tarifas_json).unwrap_or_default(),
        coste_cent: l.coste_cent,
        mano_de_obra_cent: l.mano_de_obra_cent,
        descuento_bp: l.descuento_bp,
        tipo_iva_bp: l.tipo_iva_bp,
        precio_unitario_pu: l.precio_unitario_pu,
        precio_cent: l.precio_cent,
    }
}

pub async fn aceptar_presupuesto(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<(), ErrorApp> {
    let fechas = fechas_de_situacion(estado.ahora());
    cambiar(estado, actor, "aceptar_presupuesto", move |tx, ahora| {
        let ficha = cargar(tx, id)?;
        let motivos = motivos_para_no_aceptar(&ficha, &fechas);
        if !motivos.is_empty() {
            return Err(ErrorApp::Validacion(format!(
                "No se puede aceptar: {}.",
                motivos.join("; ")
            )));
        }
        let empresa = ajustes_presupuesto::leer_de_la_base(tx)?.empresa();
        let texto = a_json(&copia(&ficha, empresa, ahora))?;
        let sha = calcular_sha256(&texto);
        let tarifas: Vec<i64> = ficha
            .lineas
            .iter()
            .flat_map(|l| serde_json::from_str::<Vec<i64>>(&l.tarifas_json).unwrap_or_default())
            .collect();
        presupuestos::aceptar(tx, id, (&texto, &sha, ahora))?;
        presupuestos::anotar_tarifas(tx, id, &tarifas)?;
        super::trabajos::crear::crear_trabajo(tx, &ficha, ahora)?;
        Ok(((), id))
    })
    .await
}
