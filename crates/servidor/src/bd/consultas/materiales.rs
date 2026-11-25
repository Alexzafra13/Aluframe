//! Material a pedir de cada ventana del trabajo (migración 0005, ADR-0035): el cristal y lo que
//! la serie marca «se_pide», con su estado (pendiente, pedido o recibido).

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaMaterial {
    pub id: i64,
    pub ventana_id: i64,
    pub orden: i64,
    pub clase: String,
    pub codigo: String,
    pub nombre: Option<String>,
    pub modelo: Option<String>,
    pub cantidad: i64,
    pub ancho_dmm: Option<i64>,
    pub alto_dmm: Option<i64>,
    pub estado: String,
    pub nota: Option<String>,
    pub usuario_nombre: Option<String>,
    pub cambiado_en: Option<String>,
    pub version: i64,
}

/// Lo que se guarda al aceptar el presupuesto, o al cambiar las medidas.
pub struct NuevoMaterial<'a> {
    pub clase: &'a str,
    pub codigo: &'a str,
    pub nombre: Option<&'a str>,
    pub modelo: Option<&'a str>,
    pub cantidad: i64,
    pub ancho_dmm: Option<i64>,
    pub alto_dmm: Option<i64>,
    pub estado: &'a str,
    pub nota: Option<&'a str>,
}

fn leer_material(f: &Row<'_>) -> rusqlite::Result<FilaMaterial> {
    Ok(FilaMaterial {
        id: f.get(0)?,
        ventana_id: f.get(1)?,
        orden: f.get(2)?,
        clase: f.get(3)?,
        codigo: f.get(4)?,
        nombre: f.get(5)?,
        modelo: f.get(6)?,
        cantidad: f.get(7)?,
        ancho_dmm: f.get(8)?,
        alto_dmm: f.get(9)?,
        estado: f.get(10)?,
        nota: f.get(11)?,
        usuario_nombre: f.get(12)?,
        cambiado_en: f.get(13)?,
        version: f.get(14)?,
    })
}

const SELECCION: &str = "SELECT m.id, m.ventana_id, m.orden, m.clase, m.codigo, m.nombre, \
     m.modelo, m.cantidad, m.ancho_dmm, m.alto_dmm, m.estado, m.nota, u.nombre, m.cambiado_en, \
     m.version FROM material_a_pedir m LEFT JOIN usuario u ON u.id = m.usuario_id";

pub fn insertar(
    con: &Connection,
    (ventana, orden): (i64, i64),
    m: &NuevoMaterial,
) -> Result<(), ErrorApp> {
    con.execute(
        "INSERT INTO material_a_pedir (ventana_id, orden, clase, codigo, nombre, modelo, \
         cantidad, ancho_dmm, alto_dmm, estado, nota) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            ventana,
            orden,
            m.clase,
            m.codigo,
            m.nombre,
            m.modelo,
            m.cantidad,
            m.ancho_dmm,
            m.alto_dmm,
            m.estado,
            m.nota
        ],
    )?;
    Ok(())
}

pub fn de_ventana(con: &Connection, ventana: i64) -> Result<Vec<FilaMaterial>, ErrorApp> {
    let sql = format!("{SELECCION} WHERE m.ventana_id = ?1 ORDER BY m.orden");
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([ventana], leer_material)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaMaterial>, ErrorApp> {
    let sql = format!("{SELECCION} WHERE m.id = ?1");
    Ok(con.query_row(&sql, [id], leer_material).optional()?)
}

/// Si otra persona lo cambió antes (otra versión), no se toca: la segunda ve el aviso.
pub fn cambiar_estado(
    con: &Connection,
    (id, version): (i64, i64),
    (estado, nota): (&str, Option<&str>),
    (usuario, cuando): (i64, &str),
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE material_a_pedir SET estado = ?3, nota = ?4, usuario_id = ?5, cambiado_en = ?6, \
         version = version + 1 WHERE id = ?1 AND version = ?2",
        params![id, version, estado, nota, usuario, cuando],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Conflicto);
    }
    Ok(())
}

/// Al cambiar las medidas se vuelve a escribir la lista de la ventana.
pub fn borrar_de_ventana(con: &Connection, ventana: i64) -> Result<(), ErrorApp> {
    con.execute(
        "DELETE FROM material_a_pedir WHERE ventana_id = ?1",
        [ventana],
    )?;
    Ok(())
}
