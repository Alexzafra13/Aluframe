//! Stock (H5): saldos, entradas y ajustes hechos a mano, mínimos y lista de la compra. Nunca se
//! edita un saldo: siempre se añade un movimiento con su motivo.

pub mod orden;
pub mod prevision;
pub mod terminar;

use dominio::acceso::Accion;
use dominio::stock::{TipoDeMovimiento, comprobar_movimiento, leer_cantidad_mil};
use dominio::unidades::{Longitud, Unidad, leer_decimal_es};

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::stock::{self, FilaMovimiento, FilaSaldo, NuevoMovimiento};
use crate::bd::consultas::{acabados, articulos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

pub async fn saldos(estado: &EstadoApp, actor: &Actor) -> Result<Vec<FilaSaldo>, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado.bd.leer(stock::saldos).await
}

pub async fn movimientos_de(
    estado: &EstadoApp,
    actor: &Actor,
    articulo: i64,
) -> Result<Vec<FilaMovimiento>, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado
        .bd
        .leer(move |con| stock::movimientos(con, articulo))
        .await
}

/// Lo que se escribe en el formulario de entrada o ajuste.
#[derive(Debug, Clone, Default)]
pub struct MovimientoManual {
    pub articulo: i64,
    pub acabado: Option<i64>,
    /// En cm; vacío para la barra entera o la unidad. Con largo, es un retal.
    pub largo_cm: String,
    pub cantidad: String,
    pub motivo: String,
}

fn validacion(e: impl ToString) -> ErrorApp {
    ErrorApp::Validacion(format!("No se ha guardado: {}.", e.to_string()))
}

fn leer_largo(texto: &str) -> Result<Option<i64>, ErrorApp> {
    if texto.trim().is_empty() {
        return Ok(None);
    }
    leer_decimal_es(texto.trim())
        .ok()
        .and_then(|d| Longitud::desde_decimal(d, Unidad::Cm).ok())
        .map(Longitud::decimas_de_mm)
        .filter(|l| *l > 0)
        .map(Some)
        .ok_or_else(|| validacion("el largo del retal va en cm, por ejemplo 120,5"))
}

struct Comprobado {
    tipo: TipoDeMovimiento,
    cantidad: i64,
    largo: Option<i64>,
    motivo: String,
}

fn guardar_movimiento(
    tx: &rusqlite::Transaction,
    m: &MovimientoManual,
    c: &Comprobado,
    (cuando, autor): (&str, i64),
) -> Result<(), ErrorApp> {
    articulos::buscar(tx, m.articulo)?.ok_or(ErrorApp::NoEncontrado)?;
    if let Some(id) = m.acabado {
        acabados::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
    }
    let actual = stock::saldo(tx, (m.articulo, m.acabado, c.largo))?;
    comprobar_movimiento(c.tipo, (c.cantidad, &c.motivo), actual).map_err(validacion)?;
    let tipo = match c.tipo {
        TipoDeMovimiento::Entrada => "entrada",
        TipoDeMovimiento::Ajuste => "ajuste",
    };
    stock::insertar(
        tx,
        &NuevoMovimiento {
            articulo: m.articulo,
            acabado: m.acabado,
            longitud_dmm: c.largo,
            tipo,
            cantidad_mil: c.cantidad,
            motivo: &c.motivo,
            orden_taller: None,
            usuario: autor,
            cuando,
        },
    )
}

pub async fn registrar(
    estado: &EstadoApp,
    actor: &Actor,
    tipo: TipoDeMovimiento,
    m: MovimientoManual,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let motivo = match (tipo, m.motivo.trim()) {
        (TipoDeMovimiento::Entrada, "") => "Entrada de material".to_owned(),
        (_, texto) => texto.chars().take(200).collect(),
    };
    let comprobado = Comprobado {
        tipo,
        cantidad: leer_cantidad_mil(&m.cantidad).map_err(validacion)?,
        largo: leer_largo(&m.largo_cm)?,
        motivo,
    };
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            guardar_movimiento(tx, &m, &comprobado, (&formatear_utc(ahora), autor))?;
            let cambio = Cambio {
                ahora,
                autor,
                accion: "movimiento_stock",
                entidad: "articulo",
                entidad_id: m.articulo,
            };
            anotar(tx, &cambio)
        })
        .await
}

pub async fn cambiar_minimo(
    estado: &EstadoApp,
    actor: &Actor,
    articulo: i64,
    texto: String,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let minimo = if texto.trim().is_empty() {
        0
    } else {
        leer_cantidad_mil(&texto).map_err(validacion)?
    };
    if minimo < 0 {
        return Err(validacion("el mínimo no puede ser negativo"));
    }
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            articulos::buscar(tx, articulo)?.ok_or(ErrorApp::NoEncontrado)?;
            stock::cambiar_minimo(tx, articulo, minimo)?;
            let cambio = Cambio {
                ahora,
                autor,
                accion: "minimo_stock",
                entidad: "articulo",
                entidad_id: articulo,
            };
            anotar(tx, &cambio)
        })
        .await
}
