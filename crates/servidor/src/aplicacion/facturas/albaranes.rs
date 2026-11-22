//! Albarán de entrega de un trabajo (DATOS 3.8): numerado (A-AAAA-NNNN), con la fotografía de lo
//! entregado y su huella; no se modifica. Al crearlo, el trabajo queda como entregado.

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use super::cambiar;
use crate::aplicacion::Actor;
use crate::aplicacion::trabajos::consultar::cargar as cargar_trabajo;
use crate::autenticacion::calcular_sha256;
use crate::bd::consultas::{presupuestos, trabajos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{anio_de_madrid, fecha_de_madrid, leer_utc};

#[derive(Debug, Clone)]
pub struct FilaAlbaran {
    pub id: i64,
    pub anio: i64,
    pub numero: i64,
    pub trabajo_id: i64,
    pub fecha: String,
    pub snapshot_json: String,
}

impl FilaAlbaran {
    pub fn codigo(&self) -> String {
        format!("A-{}-{:04}", self.anio, self.numero)
    }
}

const SELECCION: &str = "SELECT id, anio, numero, trabajo_id, fecha, snapshot_json FROM albaran";

fn leer(f: &rusqlite::Row<'_>) -> rusqlite::Result<FilaAlbaran> {
    Ok(FilaAlbaran {
        id: f.get(0)?,
        anio: f.get(1)?,
        numero: f.get(2)?,
        trabajo_id: f.get(3)?,
        fecha: f.get(4)?,
        snapshot_json: f.get(5)?,
    })
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaAlbaran>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE id = ?1"), [id], leer)
        .optional()?)
}

pub fn de_trabajo(con: &Connection, trabajo: i64) -> Result<Vec<FilaAlbaran>, ErrorApp> {
    let mut consulta = con.prepare(&format!("{SELECCION} WHERE trabajo_id = ?1 ORDER BY id"))?;
    let filas = consulta.query_map([trabajo], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub async fn crear_albaran(
    estado: &EstadoApp,
    actor: &Actor,
    trabajo: i64,
) -> Result<i64, ErrorApp> {
    cambiar(estado, actor, ("crear_albaran", "albaran"), move |tx, ahora| {
        let ficha = cargar_trabajo(tx, trabajo)?;
        let t = &ficha.trabajo;
        if t.estado == "anulado" {
            return Err(ErrorApp::Validacion("El trabajo está anulado.".into()));
        }
        let instante = leer_utc(ahora).ok_or_else(|| ErrorApp::Interno("fecha no válida".into()))?;
        let (anio, fecha) = (anio_de_madrid(instante), fecha_de_madrid(instante, 0));
        let numero = presupuestos::siguiente_numero(tx, "A", anio)?;
        let ventanas: Vec<_> = ficha.ventanas.iter().map(|v| json!({ "orden": v.fila.orden, "descripcion": v.fila.descripcion })).collect();
        let foto = json!({
            "codigo": format!("A-{anio}-{numero:04}"),
            "fecha": fecha,
            "trabajo": t.codigo(),
            "cliente": t.cliente_nombre,
            "direccion_montaje": t.direccion_montaje,
            "ventanas": ventanas,
        })
        .to_string();
        tx.execute(
            "INSERT INTO albaran (anio, numero, trabajo_id, fecha, snapshot_json, snapshot_sha256) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![anio, numero, trabajo, fecha, foto, calcular_sha256(&foto)],
        )?;
        let id = tx.last_insert_rowid();
        if t.estado != "entregado" {
            trabajos::cambiar_estado(tx, trabajo, "entregado")?;
        }
        Ok((id, id))
    })
    .await
}

pub async fn buscar_albaran(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FilaAlbaran, ErrorApp> {
    crate::aplicacion::autorizar(actor, dominio::acceso::Accion::GestionarFacturas)?;
    estado
        .bd
        .leer(move |con| buscar(con, id)?.ok_or(ErrorApp::NoEncontrado))
        .await
}
