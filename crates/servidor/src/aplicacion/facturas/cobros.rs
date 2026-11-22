//! Cobros (PANTALLAS 11): en una factura emitida, o como señal en el trabajo antes de facturar
//! (decisión del propietario). Medio: efectivo, Bizum, tarjeta, transferencia u otro. Se admiten
//! cobros parciales; un cobro no se borra, se anula con motivo.

use dominio::unidades::{Decimal, leer_decimal_es};

use super::cambiar;
use crate::aplicacion::Actor;
use crate::bd::consultas::cobros::{self, NuevoCobro};
use crate::bd::consultas::{facturas, trabajos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub const MEDIOS: [(&str, &str); 5] = [
    ("efectivo", "Efectivo"),
    ("bizum", "Bizum"),
    ("tarjeta", "Tarjeta"),
    ("transferencia", "Transferencia"),
    ("otro", "Otro"),
];

#[derive(Debug, Clone, Copy)]
pub enum DestinoDelCobro {
    Factura(i64),
    /// Señal a cuenta: se cobra en el trabajo y cuenta al emitir su factura.
    Trabajo(i64),
}

#[derive(Debug, Clone, Default)]
pub struct DatosDeCobro {
    pub fecha: String,
    pub importe: String,
    pub medio: String,
    pub nota: String,
}

fn leer_importe(texto: &str) -> Result<i64, ErrorApp> {
    let importe = leer_decimal_es(texto.trim())
        .ok()
        .and_then(|d| d.multiplicar(Decimal::desde_entero(100)).ok())
        .and_then(Decimal::a_entero)
        .filter(|c| *c > 0);
    importe.ok_or_else(|| {
        ErrorApp::Validacion("Escribe el importe cobrado, por ejemplo 250 o 250,50.".into())
    })
}

fn comprobar_destino(tx: &rusqlite::Connection, destino: DestinoDelCobro) -> Result<i64, ErrorApp> {
    match destino {
        DestinoDelCobro::Factura(id) => {
            let f = facturas::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
            if f.estado != "emitida" || f.tipo != "ordinaria" {
                return Err(ErrorApp::Validacion(
                    "Los cobros se apuntan en una factura emitida.".into(),
                ));
            }
            Ok(id)
        }
        DestinoDelCobro::Trabajo(id) => {
            let t = trabajos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
            let emitida = facturas::de_trabajo(tx, id)?
                .iter()
                .any(|f| f.estado == "emitida" && f.tipo == "ordinaria");
            if t.estado == "anulado" || emitida {
                return Err(ErrorApp::Validacion(
                    "Este trabajo ya tiene factura: apunta el cobro en ella.".into(),
                ));
            }
            Ok(id)
        }
    }
}

pub async fn registrar_cobro(
    estado: &EstadoApp,
    actor: &Actor,
    destino: DestinoDelCobro,
    d: DatosDeCobro,
) -> Result<(), ErrorApp> {
    let importe = leer_importe(&d.importe)?;
    if !MEDIOS.iter().any(|(m, _)| *m == d.medio) {
        return Err(ErrorApp::Validacion("Elige cómo se ha cobrado.".into()));
    }
    let formato = time::macros::format_description!("[year]-[month]-[day]");
    time::Date::parse(&d.fecha, formato)
        .map_err(|_| ErrorApp::Validacion("Revisa la fecha del cobro.".into()))?;
    let usuario = actor.id;
    cambiar(
        estado,
        actor,
        ("registrar_cobro", "cobro"),
        move |tx, ahora| {
            let id = comprobar_destino(tx, destino)?;
            let (factura, trabajo) = match destino {
                DestinoDelCobro::Factura(_) => (Some(id), None),
                DestinoDelCobro::Trabajo(_) => (None, Some(id)),
            };
            let nota = Some(d.nota.trim()).filter(|n| !n.is_empty());
            let nuevo = NuevoCobro {
                factura,
                trabajo,
                fecha: &d.fecha,
                importe_cent: importe,
                medio: &d.medio,
                nota,
                usuario,
                creado_en: ahora,
            };
            let cobro = cobros::insertar(tx, &nuevo)?;
            Ok(((), cobro))
        },
    )
    .await
}

/// Devuelve dónde estaba el cobro (trabajo y factura) para volver a esa pantalla.
pub async fn anular_cobro(
    estado: &EstadoApp,
    actor: &Actor,
    cobro: i64,
    motivo: String,
) -> Result<(Option<i64>, Option<i64>), ErrorApp> {
    let motivo: String = motivo.trim().chars().take(200).collect();
    if motivo.is_empty() {
        return Err(ErrorApp::Validacion(
            "Escribe por qué se anula el cobro.".into(),
        ));
    }
    cambiar(estado, actor, ("anular_cobro", "cobro"), move |tx, _| {
        let donde = cobros::buscar_trabajo_y_factura(tx, cobro)?;
        cobros::anular(tx, cobro, &motivo)?;
        Ok((donde, cobro))
    })
    .await
}
