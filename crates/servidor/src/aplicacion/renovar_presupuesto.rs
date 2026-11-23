//! Renovar un presupuesto caducado o rechazado (ADR-0027): precios de hoy y validez nueva.
//! Si ya se había enviado, lo enviado queda como versión anterior y se trabaja en una nueva.

use super::Actor;
use super::ajustes_presupuesto;
use super::lineas_de_presupuesto::recalcular_en;
use super::presupuestos::cambiar;
use crate::bd::consultas::{presupuestos, versiones_presupuesto};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fecha_de_madrid;

pub async fn renovar_presupuesto(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<(), ErrorApp> {
    let instante = estado.ahora();
    cambiar(estado, actor, "renovar_presupuesto", move |tx, ahora| {
        let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        if presupuesto.estado == "aceptado" {
            return Err(ErrorApp::Validacion(
                "Un presupuesto aceptado no se renueva.".into(),
            ));
        }
        if presupuesto.estado != "borrador" {
            let motivo = Some("Renovado con los precios de hoy");
            versiones_presupuesto::crear_copia(tx, id, (motivo, ahora))?;
            presupuestos::cambiar_estado(tx, id, "borrador", ahora)?;
        }
        recalcular_en(tx, id)?;
        let dias = ajustes_presupuesto::leer_de_la_base(tx)?.validez_dias;
        let validez = dias.map(|d| fecha_de_madrid(instante, d));
        presupuestos::cambiar_validez(tx, id, validez.as_deref())?;
        Ok(((), id))
    })
    .await
}
