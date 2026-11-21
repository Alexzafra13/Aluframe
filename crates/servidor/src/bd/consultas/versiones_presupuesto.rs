use rusqlite::{Connection, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaVersion {
    pub id: i64,
    pub numero_version: i64,
    pub motivo: Option<String>,
    pub vigente: bool,
    pub total_cent: i64,
    pub creada_en: String,
}

pub fn listar(con: &Connection, presupuesto_id: i64) -> Result<Vec<FilaVersion>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT id, numero_version, motivo, vigente, total_cent, creada_en FROM version_presupuesto \
         WHERE presupuesto_id = ?1 ORDER BY numero_version DESC",
    )?;
    let filas = consulta.query_map([presupuesto_id], |f| {
        Ok(FilaVersion {
            id: f.get(0)?,
            numero_version: f.get(1)?,
            motivo: f.get(2)?,
            vigente: f.get(3)?,
            total_cent: f.get(4)?,
            creada_en: f.get(5)?,
        })
    })?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub struct Totales {
    pub base_cent: i64,
    pub iva_cent: i64,
    pub total_cent: i64,
    pub completo: bool,
}

pub fn guardar_totales(con: &Connection, version_id: i64, t: &Totales) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE version_presupuesto SET base_cent = ?1, iva_cent = ?2, total_cent = ?3, completo = ?4 WHERE id = ?5",
        params![t.base_cent, t.iva_cent, t.total_cent, t.completo, version_id],
    )?;
    Ok(())
}

/// La versión nueva copia las líneas; la anterior queda de solo lectura (PANTALLAS, apartado 6).
pub fn crear_copia(
    con: &Connection,
    presupuesto_id: i64,
    (motivo, ahora): (Option<&str>, &str),
) -> Result<i64, ErrorApp> {
    let (anterior, numero): (i64, i64) = con.query_row(
        "SELECT id, numero_version FROM version_presupuesto WHERE presupuesto_id = ?1 AND vigente = 1",
        [presupuesto_id],
        |f| Ok((f.get(0)?, f.get(1)?)),
    )?;
    con.execute(
        "UPDATE version_presupuesto SET vigente = 0 WHERE id = ?1",
        [anterior],
    )?;
    con.execute(
        "INSERT INTO version_presupuesto (presupuesto_id, numero_version, motivo, vigente, base_cent, \
         iva_cent, total_cent, completo, creada_en) SELECT presupuesto_id, ?1, ?2, 1, base_cent, iva_cent, \
         total_cent, completo, ?3 FROM version_presupuesto WHERE id = ?4",
        params![numero + 1, motivo, ahora, anterior],
    )?;
    let nueva = con.last_insert_rowid();
    con.execute(
        "INSERT INTO linea_presupuesto (version_id, orden, tipo, descripcion, cantidad_mil, serie_version_id, \
         tipologia, ancho_dmm, alto_dmm, acabado_id, vidrio_articulo_id, opciones_json, resultado_json, \
         faltan_json, incompleta, revisar_limites, tarifas_json, coste_cent, mano_de_obra_cent, descuento_bp, \
         tipo_iva_bp, precio_unitario_pu, precio_cent) SELECT ?1, orden, tipo, descripcion, cantidad_mil, \
         serie_version_id, tipologia, ancho_dmm, alto_dmm, acabado_id, vidrio_articulo_id, opciones_json, \
         resultado_json, faltan_json, incompleta, revisar_limites, tarifas_json, coste_cent, \
         mano_de_obra_cent, descuento_bp, tipo_iva_bp, precio_unitario_pu, precio_cent \
         FROM linea_presupuesto WHERE version_id = ?2",
        params![nueva, anterior],
    )?;
    Ok(nueva)
}
