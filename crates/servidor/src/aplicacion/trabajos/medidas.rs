//! «Modificar medidas» de una ventana del trabajo (PANTALLAS 8): solo el propietario, mientras
//! la orden no esté terminada. Recalcula con la misma versión de la serie con que se aceptó y
//! guarda una revisión con su motivo; el presupuesto aceptado no cambia, ni el importe.

use std::collections::BTreeMap;

use dominio::acceso::Accion;
use dominio::series::OpcionDeLinea;
use dominio::unidades::{Decimal, Dinero, Longitud, Porcentaje, Unidad, leer_decimal_es};
use rusqlite::Connection;

use super::acciones::actualizar_estado;
use super::congelado::VentanaCongelada;
use super::consultar::{VentanaDeTrabajo, ventana_completa};
use crate::aplicacion::auditar::{Cambio, anotar};
use crate::aplicacion::calculo_de_ventana::{EntradaDeVentana, calcular};
use crate::aplicacion::lineas_de_presupuesto::a_json;
use crate::aplicacion::resumen_de_linea::ResumenDeVentana;
use crate::aplicacion::series::reconstruir;
use crate::aplicacion::{Actor, ajustes_presupuesto, autorizar, taller};
use crate::bd::consultas::ventanas_trabajo::{self, Revision};
use crate::bd::consultas::{pasos, series, trabajos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

#[derive(Debug, Clone)]
pub struct NuevasMedidas {
    pub ancho: String,
    pub alto: String,
    pub opciones: BTreeMap<String, String>,
    pub motivo: String,
    pub reiniciar_pasos: bool,
}

pub struct Revisable {
    pub ventana: VentanaDeTrabajo,
    pub opciones: Vec<OpcionDeLinea>,
    pub pasos_hechos: bool,
}

pub fn cm_texto(dmm: i64) -> String {
    Longitud::desde_decimas_de_mm(dmm).formatear_cm()
}

fn comprobar_abierta(con: &Connection, trabajo: i64) -> Result<(), ErrorApp> {
    let t = trabajos::buscar(con, trabajo)?.ok_or(ErrorApp::NoEncontrado)?;
    if t.orden_terminada || matches!(t.estado.as_str(), "entregado" | "anulado") {
        return Err(ErrorApp::Validacion(
            "La orden ya está terminada: no se cambian medidas. Anula el trabajo o registra un ajuste de stock.".into(),
        ));
    }
    Ok(())
}

pub async fn preparar_revision(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<Revisable, ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado
        .bd
        .leer(move |con| {
            let fila = ventanas_trabajo::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
            comprobar_abierta(con, fila.trabajo_id)?;
            let ventana = ventana_completa(con, fila)?;
            let c = &ventana.congelado;
            let version =
                series::buscar_version(con, c.serie_version_id)?.ok_or(ErrorApp::NoEncontrado)?;
            let serie = reconstruir(version)?.serie;
            let opciones = serie
                .tipologia(&c.tipologia)
                .map(|t| t.opciones.clone())
                .unwrap_or_default();
            let pasos_hechos = ventana.pasos.iter().any(|p| p.estado != "pendiente");
            Ok(Revisable {
                ventana,
                opciones,
                pasos_hechos,
            })
        })
        .await
}

fn leer_medida(texto: &str, nombre: &str) -> Result<Decimal, ErrorApp> {
    leer_decimal_es(texto)
        .ok()
        .filter(|d| *d > Decimal::CERO)
        .ok_or_else(|| ErrorApp::Validacion(format!("Escribe el {nombre} en cm.")))
}

/// La ventana recalculada sin guardar: para enseñar antes y después y para la revisión.
pub fn recalcular(
    con: &Connection,
    anterior: &VentanaCongelada,
    medidas: &NuevasMedidas,
) -> Result<VentanaCongelada, ErrorApp> {
    let entrada = EntradaDeVentana {
        serie_version_id: anterior.serie_version_id,
        tipologia: anterior.tipologia.clone(),
        ancho: leer_medida(&medidas.ancho, "ancho")?,
        alto: leer_medida(&medidas.alto, "alto")?,
        opciones: medidas.opciones.clone(),
        acabado_id: anterior.acabado_id,
        vidrio_articulo_id: anterior.vidrio_articulo_id,
        cantidad: 1,
        mano_de_obra: Dinero::CERO,
        descuento: Porcentaje::CERO,
    };
    let ajustes = (
        taller::leer_de_la_base(con)?,
        ajustes_presupuesto::leer_de_la_base(con)?,
    );
    let calculo = calcular(con, &entrada, (&ajustes.0, &ajustes.1))?;
    let medida = |d: Decimal| Longitud::desde_decimal(d, Unidad::Cm).map(|l| l.decimas_de_mm());
    Ok(VentanaCongelada {
        ancho_dmm: medida(entrada.ancho).map_err(|e| ErrorApp::Validacion(e.to_string()))?,
        alto_dmm: medida(entrada.alto).map_err(|e| ErrorApp::Validacion(e.to_string()))?,
        opciones: entrada.opciones,
        descripcion: calculo.descripcion.clone(),
        resumen: ResumenDeVentana::from(&calculo),
        ..anterior.clone()
    })
}

pub async fn previsualizar(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    medidas: NuevasMedidas,
) -> Result<(VentanaCongelada, VentanaCongelada), ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado
        .bd
        .leer(move |con| {
            let fila = ventanas_trabajo::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
            let anterior = VentanaCongelada::leer(&fila.congelado_json);
            let nueva = recalcular(con, &anterior, &medidas)?;
            Ok((anterior, nueva))
        })
        .await
}

/// Guarda la revisión dentro de la transacción; devuelve el trabajo.
fn guardar_revision(
    tx: &Connection,
    (id, version, autor, cuando): (i64, i64, i64, &str),
    (medidas, motivo): (&NuevasMedidas, &str),
) -> Result<i64, ErrorApp> {
    let fila = ventanas_trabajo::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
    comprobar_abierta(tx, fila.trabajo_id)?;
    let anterior = VentanaCongelada::leer(&fila.congelado_json);
    let recalculada = recalcular(tx, &anterior, medidas)?;
    let nueva = a_json(&recalculada)?;
    let descripcion = recalculada.descripcion.clone();
    let datos = (descripcion.as_str(), nueva.as_str(), cuando);
    let revision = ventanas_trabajo::revisar(tx, (id, version), datos)?;
    let apunte = Revision {
        ventana: id,
        revision,
        anterior: &fila.congelado_json,
        nuevo: &nueva,
        motivo,
        pasos_reiniciados: medidas.reiniciar_pasos,
        usuario: autor,
        cuando,
    };
    ventanas_trabajo::anotar_revision(tx, &apunte)?;
    super::materiales::rehacer_materiales(tx, id, &recalculada)?;
    if medidas.reiniciar_pasos {
        pasos::reiniciar_de_ventana(tx, id)?;
    }
    Ok(fila.trabajo_id)
}

pub async fn modificar_medidas(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
    medidas: NuevasMedidas,
) -> Result<i64, ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    let motivo: String = medidas.motivo.trim().chars().take(200).collect();
    if motivo.is_empty() {
        return Err(ErrorApp::Validacion(
            "Escribe el motivo del cambio (una línea).".into(),
        ));
    }
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let cuando = formatear_utc(ahora);
            let trabajo = guardar_revision(tx, (id, version, autor, &cuando), (&medidas, &motivo))?;
            let fila = trabajos::buscar(tx, trabajo)?.ok_or(ErrorApp::NoEncontrado)?;
            actualizar_estado(tx, trabajo, &fila.estado)?;
            let cambio = Cambio {
                ahora,
                autor,
                accion: "modificar_medidas",
                entidad: "ventana_trabajo",
                entidad_id: id,
            };
            anotar(tx, &cambio)?;
            Ok(trabajo)
        })
        .await
}
