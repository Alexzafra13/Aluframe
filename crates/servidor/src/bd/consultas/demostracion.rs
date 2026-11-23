//! Lo que solo necesita la demostración al rellenarse (ADR-0028): fechas hacia atrás para que
//! haya presupuestos caducados y archivados, y buscar el trabajo de un presupuesto.

use rusqlite::{Connection, params};

use crate::errores::ErrorApp;

/// Un presupuesto sin aceptar como si se hubiera hecho antes (los aceptados no se tocan).
pub fn fechar_presupuesto(
    con: &Connection,
    id: i64,
    (creado_en, validez_hasta): (&str, &str),
) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE presupuesto SET creado_en = ?2, validez_hasta = ?3, version = version + 1 \
         WHERE id = ?1 AND estado <> 'aceptado'",
        params![id, creado_en, validez_hasta],
    )?;
    Ok(())
}

pub fn trabajo_del_presupuesto(con: &Connection, presupuesto: i64) -> Result<i64, ErrorApp> {
    Ok(con.query_row(
        "SELECT id FROM trabajo WHERE presupuesto_id = ?1",
        [presupuesto],
        |f| f.get(0),
    )?)
}

pub fn version_de_trabajo(con: &Connection, trabajo: i64) -> Result<i64, ErrorApp> {
    Ok(con.query_row(
        "SELECT version FROM trabajo WHERE id = ?1",
        [trabajo],
        |f| f.get(0),
    )?)
}

/// Pasos de un trabajo: identificador y versión, en orden de ventana y de paso.
pub fn pasos_del_trabajo(con: &Connection, trabajo: i64) -> Result<Vec<(i64, i64)>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT p.id, p.version FROM paso_ventana p JOIN ventana_trabajo v ON v.id = p.ventana_id \
         WHERE v.trabajo_id = ?1 ORDER BY v.orden, p.orden",
    )?;
    let filas = consulta.query_map([trabajo], |f| Ok((f.get(0)?, f.get(1)?)))?;
    Ok(filas.collect::<Result<_, _>>()?)
}
