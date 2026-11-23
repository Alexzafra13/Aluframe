//! Facturas y sus líneas (DATOS 7.9). Lo emitido lo protegen los disparadores del esquema.

use dominio::factura::ImportesDeFactura;
use dominio::unidades::Dinero;
use rusqlite::{Connection, OptionalExtension, Row, params};

pub use super::lineas_factura::{FilaLineaFactura, insertar_linea, lineas, quitar_linea};
use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaFactura {
    pub id: i64,
    pub serie: String,
    pub anio: i64,
    pub numero: Option<i64>,
    pub tipo: String,
    pub rectifica_factura_id: Option<i64>,
    pub motivo_rectificacion: Option<String>,
    pub cliente_id: i64,
    pub cliente_nombre: Option<String>,
    pub trabajo_id: Option<i64>,
    pub estado: String,
    pub fecha_emision: Option<String>,
    pub fecha_operacion: Option<String>,
    pub vencimiento: Option<String>,
    pub notas: Option<String>,
    pub descuento_bp: i64,
    pub base_cent: i64,
    pub iva_cent: i64,
    pub total_cent: i64,
    pub snapshot_json: Option<String>,
    pub creado_en: String,
    pub version: i64,
    pub cobrado_cent: i64,
    /// La suma de las rectificativas emitidas que la corrigen (negativa si le quitan importe).
    pub rectificado_cent: i64,
}

impl FilaFactura {
    pub fn codigo(&self) -> String {
        dominio::factura::codigo_de_factura(&self.serie, self.anio, self.numero)
    }

    pub fn importes(&self) -> ImportesDeFactura {
        ImportesDeFactura {
            total: Dinero::desde_centimos(self.total_cent),
            rectificado: Dinero::desde_centimos(self.rectificado_cent),
            cobrado: Dinero::desde_centimos(self.cobrado_cent),
        }
    }
}

const SELECCION: &str = "SELECT f.id, f.serie, f.anio, f.numero, f.tipo, f.rectifica_factura_id, \
     f.motivo_rectificacion, f.cliente_id, c.nombre_o_razon_social, f.trabajo_id, f.estado, \
     f.fecha_emision, f.fecha_operacion, f.vencimiento, f.notas, f.descuento_bp, f.base_cent, \
     f.iva_cent, f.total_cent, f.snapshot_json, f.creado_en, f.version, \
     (SELECT ifnull(sum(importe_cent), 0) FROM cobro WHERE factura_id = f.id AND anulado = 0) \
     AS cobrado_cent, ifnull(r.suma, 0) AS rectificado_cent \
     FROM factura f JOIN cliente c ON c.id = f.cliente_id \
     LEFT JOIN (SELECT rectifica_factura_id AS id, sum(total_cent) AS suma FROM factura \
     WHERE tipo = 'rectificativa' AND estado = 'emitida' GROUP BY rectifica_factura_id) r \
     ON r.id = f.id";

fn leer(f: &Row<'_>) -> rusqlite::Result<FilaFactura> {
    Ok(FilaFactura {
        id: f.get(0)?,
        serie: f.get(1)?,
        anio: f.get(2)?,
        numero: f.get(3)?,
        tipo: f.get(4)?,
        rectifica_factura_id: f.get(5)?,
        motivo_rectificacion: f.get(6)?,
        cliente_id: f.get(7)?,
        cliente_nombre: f.get(8)?,
        trabajo_id: f.get(9)?,
        estado: f.get(10)?,
        fecha_emision: f.get(11)?,
        fecha_operacion: f.get(12)?,
        vencimiento: f.get(13)?,
        notas: f.get(14)?,
        descuento_bp: f.get(15)?,
        base_cent: f.get(16)?,
        iva_cent: f.get(17)?,
        total_cent: f.get(18)?,
        snapshot_json: f.get(19)?,
        creado_en: f.get(20)?,
        version: f.get(21)?,
        cobrado_cent: f.get(22)?,
        rectificado_cent: f.get(23)?,
    })
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaFactura>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE f.id = ?1"), [id], leer)
        .optional()?)
}

pub fn listar(con: &Connection) -> Result<Vec<FilaFactura>, ErrorApp> {
    let sql = format!(
        "{SELECCION} ORDER BY f.estado = 'emitida', f.anio DESC, f.numero DESC, f.id DESC LIMIT 500"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Las emitidas que aún deben algo, todas (sin el límite de la lista): de aquí salen el filtro
/// «Pendientes» y los avisos de cobros vencidos.
pub fn por_cobrar(con: &Connection) -> Result<Vec<FilaFactura>, ErrorApp> {
    let sql = format!(
        "SELECT * FROM ({SELECCION} WHERE f.estado = 'emitida' AND f.tipo = 'ordinaria') \
         WHERE total_cent + rectificado_cent > cobrado_cent ORDER BY anio, numero"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Las rectificativas (emitidas o en borrador) que corrigen a una factura.
pub fn rectificativas_de(con: &Connection, original: i64) -> Result<Vec<FilaFactura>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE f.rectifica_factura_id = ?1 ORDER BY f.id"
    ))?;
    let filas = consulta.query_map([original], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn de_trabajo(con: &Connection, trabajo: i64) -> Result<Vec<FilaFactura>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE f.trabajo_id = ?1 ORDER BY f.id"
    ))?;
    let filas = consulta.query_map([trabajo], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub struct NuevaFactura<'a> {
    pub serie: &'a str,
    pub anio: i64,
    pub tipo: &'a str,
    pub rectifica: Option<i64>,
    pub motivo: Option<&'a str>,
    pub cliente: i64,
    pub trabajo: Option<i64>,
    pub descuento_bp: i64,
    pub creado_en: &'a str,
}

pub fn insertar(con: &Connection, f: &NuevaFactura<'_>) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO factura (serie, anio, tipo, rectifica_factura_id, motivo_rectificacion, \
         cliente_id, trabajo_id, descuento_bp, creado_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            f.serie,
            f.anio,
            f.tipo,
            f.rectifica,
            f.motivo,
            f.cliente,
            f.trabajo,
            f.descuento_bp,
            f.creado_en
        ],
    )?;
    Ok(con.last_insert_rowid())
}

/// Datos del borrador, con bloqueo optimista.
pub fn cambiar_datos(
    con: &Connection,
    (id, version): (i64, i64),
    (fecha_operacion, vencimiento, notas, descuento_bp): (
        Option<&str>,
        Option<&str>,
        Option<&str>,
        i64,
    ),
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE factura SET fecha_operacion = ?3, vencimiento = ?4, notas = ?5, descuento_bp = ?6, \
         version = version + 1 WHERE id = ?1 AND version = ?2 AND estado = 'borrador'",
        params![
            id,
            version,
            fecha_operacion,
            vencimiento,
            notas,
            descuento_bp
        ],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Conflicto);
    }
    Ok(())
}

pub fn guardar_totales(
    con: &Connection,
    id: i64,
    (base, iva, total): (i64, i64, i64),
) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE factura SET base_cent = ?2, iva_cent = ?3, total_cent = ?4 WHERE id = ?1 AND estado = 'borrador'",
        params![id, base, iva, total],
    )?;
    Ok(())
}

pub struct Emision<'a> {
    pub anio: i64,
    pub numero: i64,
    pub fecha: &'a str,
    pub fecha_operacion: &'a str,
    pub snapshot: &'a str,
    pub sha256: &'a str,
}

/// Número y fotografía en el mismo cambio que pasa a emitida (BACKEND 5.6).
pub fn emitir(con: &Connection, id: i64, e: &Emision<'_>) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE factura SET estado = 'emitida', numero = ?2, fecha_emision = ?3, \
         fecha_operacion = ifnull(fecha_operacion, ?4), snapshot_json = ?5, snapshot_sha256 = ?6, \
         anio = ?7, version = version + 1 WHERE id = ?1 AND estado = 'borrador'",
        params![
            id,
            e.numero,
            e.fecha,
            e.fecha_operacion,
            e.snapshot,
            e.sha256,
            e.anio
        ],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Validacion("La factura ya está emitida.".into()));
    }
    Ok(())
}

pub fn borrar_borrador(con: &Connection, id: i64) -> Result<(), ErrorApp> {
    con.execute(
        "DELETE FROM factura WHERE id = ?1 AND estado = 'borrador'",
        [id],
    )?;
    Ok(())
}
