//! Facturas, cobros y albaranes (H6). Una factura es borrador (editable, sin número) hasta que se
//! emite: recibe su número sin huecos, se congela con su huella y ya no cambia; una corrección es
//! una rectificativa. Los cobros van aparte y el estado de pago se calcula con ellos.

pub mod albaranes;
pub mod cobros;
pub mod crear;
pub mod editar;
pub mod emitir;

use dominio::acceso::Accion;
use rusqlite::{Connection, Transaction};

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::clientes::{self, FilaCliente};
use crate::bd::consultas::cobros::{FilaCobro, de_factura as cobros_de_factura};
use crate::bd::consultas::facturas::{self, FilaFactura, FilaLineaFactura};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

pub struct FichaDeFactura {
    pub factura: FilaFactura,
    pub cliente: FilaCliente,
    pub lineas: Vec<FilaLineaFactura>,
    pub cobros: Vec<FilaCobro>,
    /// La factura que corrige, si es una rectificativa.
    pub rectifica: Option<FilaFactura>,
}

pub fn cargar(con: &Connection, id: i64) -> Result<FichaDeFactura, ErrorApp> {
    let factura = facturas::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
    let rectifica = match factura.rectifica_factura_id {
        Some(r) => facturas::buscar(con, r)?,
        None => None,
    };
    Ok(FichaDeFactura {
        cliente: clientes::buscar(con, factura.cliente_id)?.ok_or(ErrorApp::NoEncontrado)?,
        lineas: facturas::lineas(con, id)?,
        cobros: cobros_de_factura(con, id)?,
        rectifica,
        factura,
    })
}

pub async fn cargar_factura(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FichaDeFactura, ErrorApp> {
    autorizar(actor, Accion::GestionarFacturas)?;
    estado.bd.leer(move |con| cargar(con, id)).await
}

pub async fn listar_facturas(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<FilaFactura>, ErrorApp> {
    autorizar(actor, Accion::GestionarFacturas)?;
    estado.bd.leer(facturas::listar).await
}

/// Ejecuta `operacion` en una transacción y anota el cambio sobre la entidad que devuelve.
pub(crate) async fn cambiar<T: Send + 'static>(
    estado: &EstadoApp,
    actor: &Actor,
    (accion, entidad): (&'static str, &'static str),
    operacion: impl FnOnce(&Transaction, &str) -> Result<(T, i64), ErrorApp> + Send + 'static,
) -> Result<T, ErrorApp> {
    autorizar(actor, Accion::GestionarFacturas)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let (resultado, entidad_id) = operacion(tx, &formatear_utc(ahora))?;
            let cambio = Cambio {
                ahora,
                autor,
                accion,
                entidad,
                entidad_id,
            };
            anotar(tx, &cambio)?;
            Ok(resultado)
        })
        .await
}

pub(crate) fn exigir_borrador(factura: &FilaFactura) -> Result<(), ErrorApp> {
    if factura.estado == "borrador" {
        Ok(())
    } else {
        Err(ErrorApp::Validacion(
            "Una factura emitida no se modifica: si hay que corregirla, crea una rectificativa."
                .into(),
        ))
    }
}
