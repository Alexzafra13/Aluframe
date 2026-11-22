//! Cobros: de una factura o, como señal, de un trabajo antes de facturar (DATOS 7.9).

use rusqlite::{Connection, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaCobro {
    pub id: i64,
    pub factura_id: Option<i64>,
    pub trabajo_id: Option<i64>,
    pub fecha: String,
    pub importe_cent: i64,
    pub medio: String,
    pub nota: Option<String>,
    pub anulado: bool,
    pub motivo_anulacion: Option<String>,
    pub usuario: String,
}

const SELECCION: &str = "SELECT c.id, c.factura_id, c.trabajo_id, c.fecha, c.importe_cent, \
     c.medio, c.nota, c.anulado, c.motivo_anulacion, u.nombre FROM cobro c \
     JOIN usuario u ON u.id = c.usuario_id";

fn consultar(con: &Connection, filtro: &str, id: i64) -> Result<Vec<FilaCobro>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE {filtro} ORDER BY c.fecha, c.id"
    ))?;
    let filas = consulta.query_map([id], |f| {
        Ok(FilaCobro {
            id: f.get(0)?,
            factura_id: f.get(1)?,
            trabajo_id: f.get(2)?,
            fecha: f.get(3)?,
            importe_cent: f.get(4)?,
            medio: f.get(5)?,
            nota: f.get(6)?,
            anulado: f.get(7)?,
            motivo_anulacion: f.get(8)?,
            usuario: f.get(9)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn de_factura(con: &Connection, factura: i64) -> Result<Vec<FilaCobro>, ErrorApp> {
    consultar(con, "c.factura_id = ?1", factura)
}

pub fn de_trabajo(con: &Connection, trabajo: i64) -> Result<Vec<FilaCobro>, ErrorApp> {
    consultar(con, "c.trabajo_id = ?1", trabajo)
}

pub struct NuevoCobro<'a> {
    pub factura: Option<i64>,
    pub trabajo: Option<i64>,
    pub fecha: &'a str,
    pub importe_cent: i64,
    pub medio: &'a str,
    pub nota: Option<&'a str>,
    pub usuario: i64,
    pub creado_en: &'a str,
}

pub fn insertar(con: &Connection, c: &NuevoCobro<'_>) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO cobro (factura_id, trabajo_id, fecha, importe_cent, medio, nota, usuario_id, \
         creado_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            c.factura,
            c.trabajo,
            c.fecha,
            c.importe_cent,
            c.medio,
            c.nota,
            c.usuario,
            c.creado_en
        ],
    )?;
    Ok(con.last_insert_rowid())
}

/// Un cobro no se borra: se anula con motivo y deja de contar.
pub fn anular(con: &Connection, id: i64, motivo: &str) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE cobro SET anulado = 1, motivo_anulacion = ?2 WHERE id = ?1 AND anulado = 0",
        params![id, motivo],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Validacion("Ese cobro ya estaba anulado.".into()));
    }
    Ok(())
}

/// Al emitir la factura de un trabajo, sus señales pasan a contar en ella como pagado a cuenta.
pub fn llevar_senales_a_factura(
    con: &Connection,
    trabajo: i64,
    factura: i64,
) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE cobro SET factura_id = ?2 WHERE trabajo_id = ?1 AND factura_id IS NULL",
        params![trabajo, factura],
    )?;
    Ok(())
}

pub fn buscar_trabajo_y_factura(
    con: &Connection,
    id: i64,
) -> Result<(Option<i64>, Option<i64>), ErrorApp> {
    Ok(con.query_row(
        "SELECT trabajo_id, factura_id FROM cobro WHERE id = ?1",
        [id],
        |f| Ok((f.get(0)?, f.get(1)?)),
    )?)
}
