use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone, Default)]
pub struct FilaLinea {
    pub id: i64,
    pub version_id: i64,
    pub orden: i64,
    pub tipo: String,
    pub descripcion: String,
    pub cantidad_mil: i64,
    pub serie_version_id: Option<i64>,
    pub tipologia: Option<String>,
    pub ancho_dmm: Option<i64>,
    pub alto_dmm: Option<i64>,
    pub acabado_id: Option<i64>,
    pub vidrio_articulo_id: Option<i64>,
    pub opciones_json: String,
    pub resultado_json: Option<String>,
    pub faltan_json: String,
    pub incompleta: bool,
    pub revisar_limites: bool,
    pub tarifas_json: String,
    pub coste_cent: i64,
    pub mano_de_obra_cent: i64,
    pub descuento_bp: i64,
    pub tipo_iva_bp: i64,
    pub precio_unitario_pu: Option<i64>,
    pub precio_cent: i64,
}

const COLUMNAS: &str = "id, version_id, orden, tipo, descripcion, cantidad_mil, serie_version_id, \
     tipologia, ancho_dmm, alto_dmm, acabado_id, vidrio_articulo_id, opciones_json, resultado_json, \
     faltan_json, incompleta, revisar_limites, tarifas_json, coste_cent, mano_de_obra_cent, descuento_bp, \
     tipo_iva_bp, precio_unitario_pu, precio_cent";

fn leer(f: &Row) -> rusqlite::Result<FilaLinea> {
    Ok(FilaLinea {
        id: f.get(0)?,
        version_id: f.get(1)?,
        orden: f.get(2)?,
        tipo: f.get(3)?,
        descripcion: f.get(4)?,
        cantidad_mil: f.get(5)?,
        serie_version_id: f.get(6)?,
        tipologia: f.get(7)?,
        ancho_dmm: f.get(8)?,
        alto_dmm: f.get(9)?,
        acabado_id: f.get(10)?,
        vidrio_articulo_id: f.get(11)?,
        opciones_json: f.get(12)?,
        resultado_json: f.get(13)?,
        faltan_json: f.get(14)?,
        incompleta: f.get(15)?,
        revisar_limites: f.get(16)?,
        tarifas_json: f.get(17)?,
        coste_cent: f.get(18)?,
        mano_de_obra_cent: f.get(19)?,
        descuento_bp: f.get(20)?,
        tipo_iva_bp: f.get(21)?,
        precio_unitario_pu: f.get(22)?,
        precio_cent: f.get(23)?,
    })
}

pub fn listar(con: &Connection, version_id: i64) -> Result<Vec<FilaLinea>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "SELECT {COLUMNAS} FROM linea_presupuesto WHERE version_id = ?1 ORDER BY orden"
    ))?;
    let filas = consulta.query_map([version_id], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(
    con: &Connection,
    version_id: i64,
    orden: i64,
) -> Result<Option<FilaLinea>, ErrorApp> {
    Ok(con
        .query_row(
            &format!(
                "SELECT {COLUMNAS} FROM linea_presupuesto WHERE version_id = ?1 AND orden = ?2"
            ),
            params![version_id, orden],
            leer,
        )
        .optional()?)
}

/// Inserta (orden nuevo al final) o reemplaza la línea de ese orden.
pub fn guardar(con: &Connection, l: &FilaLinea) -> Result<i64, ErrorApp> {
    let orden = if l.orden > 0 {
        l.orden
    } else {
        con.query_row(
            "SELECT coalesce(max(orden), 0) + 1 FROM linea_presupuesto WHERE version_id = ?1",
            [l.version_id],
            |f| f.get(0),
        )?
    };
    con.execute(
        "DELETE FROM linea_presupuesto WHERE version_id = ?1 AND orden = ?2",
        params![l.version_id, orden],
    )?;
    con.execute(
        &format!("INSERT INTO linea_presupuesto ({}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)", COLUMNAS.trim_start_matches("id, ")),
        params![
            l.version_id, orden, l.tipo, l.descripcion, l.cantidad_mil, l.serie_version_id, l.tipologia,
            l.ancho_dmm, l.alto_dmm, l.acabado_id, l.vidrio_articulo_id, l.opciones_json, l.resultado_json,
            l.faltan_json, l.incompleta, l.revisar_limites, l.tarifas_json, l.coste_cent, l.mano_de_obra_cent,
            l.descuento_bp, l.tipo_iva_bp, l.precio_unitario_pu, l.precio_cent
        ],
    )?;
    Ok(orden)
}

/// Quita una línea y cierra el hueco para que los números sigan seguidos.
pub fn borrar(con: &Connection, version_id: i64, orden: i64) -> Result<(), ErrorApp> {
    if con.execute(
        "DELETE FROM linea_presupuesto WHERE version_id = ?1 AND orden = ?2",
        params![version_id, orden],
    )? == 0
    {
        return Err(ErrorApp::NoEncontrado);
    }
    let mut siguientes = con.prepare(
        "SELECT orden FROM linea_presupuesto WHERE version_id = ?1 AND orden > ?2 ORDER BY orden",
    )?;
    let ordenes: Vec<i64> = siguientes
        .query_map(params![version_id, orden], |f| f.get(0))?
        .collect::<Result<_, _>>()?;
    for viejo in ordenes {
        con.execute(
            "UPDATE linea_presupuesto SET orden = ?1 WHERE version_id = ?2 AND orden = ?3",
            params![viejo - 1, version_id, viejo],
        )?;
    }
    Ok(())
}
