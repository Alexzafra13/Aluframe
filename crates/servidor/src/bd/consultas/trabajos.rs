//! Trabajos y su orden de taller (DATOS 7.7).

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaTrabajo {
    pub id: i64,
    pub anio: i64,
    pub numero: i64,
    pub presupuesto_id: i64,
    pub cliente_id: i64,
    pub cliente_nombre: String,
    pub direccion_montaje: Option<String>,
    pub estado: String,
    pub asignado_a: Option<i64>,
    pub asignado_nombre: Option<String>,
    pub fecha_prevista: Option<String>,
    pub importe_cent: i64,
    pub creado_en: String,
    pub version: i64,
    pub orden_terminada: bool,
}

impl FilaTrabajo {
    pub fn codigo(&self) -> String {
        format!("T-{}-{:04}", self.anio, self.numero)
    }
}

const COLUMNAS: &str = "t.id, t.anio, t.numero, t.presupuesto_id, t.cliente_id, t.cliente_nombre, \
     t.direccion_montaje, t.estado, t.asignado_a, u.nombre, t.fecha_prevista, t.importe_cent, \
     t.creado_en, t.version, ifnull(o.estado = 'terminada', 0)";

const DESDE: &str = "FROM trabajo t LEFT JOIN usuario u ON u.id = t.asignado_a \
     LEFT JOIN orden_taller o ON o.trabajo_id = t.id";

fn leer_fila(f: &Row<'_>) -> rusqlite::Result<FilaTrabajo> {
    Ok(FilaTrabajo {
        id: f.get(0)?,
        anio: f.get(1)?,
        numero: f.get(2)?,
        presupuesto_id: f.get(3)?,
        cliente_id: f.get(4)?,
        cliente_nombre: f.get(5)?,
        direccion_montaje: f.get(6)?,
        estado: f.get(7)?,
        asignado_a: f.get(8)?,
        asignado_nombre: f.get(9)?,
        fecha_prevista: f.get(10)?,
        importe_cent: f.get(11)?,
        creado_en: f.get(12)?,
        version: f.get(13)?,
        orden_terminada: f.get(14)?,
    })
}

pub struct NuevoTrabajo<'a> {
    pub anio: i64,
    pub numero: i64,
    pub presupuesto_id: i64,
    pub cliente_id: i64,
    pub cliente_nombre: &'a str,
    pub direccion_montaje: Option<&'a str>,
    pub importe_cent: i64,
    pub creado_en: &'a str,
}

pub fn insertar(con: &Connection, t: &NuevoTrabajo<'_>) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO trabajo (anio, numero, presupuesto_id, cliente_id, cliente_nombre, \
         direccion_montaje, importe_cent, creado_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            t.anio,
            t.numero,
            t.presupuesto_id,
            t.cliente_id,
            t.cliente_nombre,
            t.direccion_montaje,
            t.importe_cent,
            t.creado_en
        ],
    )?;
    let id = con.last_insert_rowid();
    con.execute("INSERT INTO orden_taller (trabajo_id) VALUES (?1)", [id])?;
    Ok(id)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaTrabajo>, ErrorApp> {
    let sql = format!("SELECT {COLUMNAS} {DESDE} WHERE t.id = ?1");
    Ok(con.query_row(&sql, [id], leer_fila).optional()?)
}

pub fn de_presupuesto(con: &Connection, presupuesto: i64) -> Result<Option<i64>, ErrorApp> {
    Ok(con
        .query_row(
            "SELECT id FROM trabajo WHERE presupuesto_id = ?1",
            [presupuesto],
            |f| f.get(0),
        )
        .optional()?)
}

/// Los que siguen en el taller primero, por fecha prevista; después lo entregado y anulado.
pub fn listar(con: &Connection, incluir_cerrados: bool) -> Result<Vec<FilaTrabajo>, ErrorApp> {
    let filtro = if incluir_cerrados {
        ""
    } else {
        "WHERE t.estado NOT IN ('entregado', 'anulado')"
    };
    let sql = format!(
        "SELECT {COLUMNAS} {DESDE} {filtro} ORDER BY t.estado IN ('entregado', 'anulado'), \
         t.fecha_prevista IS NULL, t.fecha_prevista, t.anio, t.numero"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn de_cliente(con: &Connection, cliente: i64) -> Result<Vec<FilaTrabajo>, ErrorApp> {
    let sql = format!("SELECT {COLUMNAS} {DESDE} WHERE t.cliente_id = ?1 ORDER BY t.id DESC");
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([cliente], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Con bloqueo optimista: si otra persona lo cambió antes, no se toca y se avisa.
pub fn cambiar_datos(
    con: &Connection,
    (id, version): (i64, i64),
    (asignado_a, fecha_prevista): (Option<i64>, Option<&str>),
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE trabajo SET asignado_a = ?3, fecha_prevista = ?4, version = version + 1 \
         WHERE id = ?1 AND version = ?2",
        params![id, version, asignado_a, fecha_prevista],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Conflicto);
    }
    Ok(())
}

pub fn cambiar_estado(con: &Connection, id: i64, estado: &str) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE trabajo SET estado = ?2, version = version + 1 WHERE id = ?1 AND estado <> ?2",
        params![id, estado],
    )?;
    Ok(())
}
