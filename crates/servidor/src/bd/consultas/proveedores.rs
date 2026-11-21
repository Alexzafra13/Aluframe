use dominio::catalogo::DatosDeProveedor;
use dominio::nif::Nif;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaProveedor {
    pub id: i64,
    pub nombre: String,
    pub nif: Option<String>,
    pub contacto: Option<String>,
    pub condiciones_pago: Option<String>,
    pub descuento_habitual_bp: i64,
    pub notas: Option<String>,
    pub archivado: bool,
    pub version: i64,
    pub articulos: i64,
}

const SELECCION: &str = "SELECT p.id, p.nombre, p.nif, p.contacto, p.condiciones_pago, \
     p.descuento_habitual_bp, p.notas, p.archivado, p.version, \
     (SELECT count(*) FROM articulo a WHERE a.proveedor_id = p.id AND a.archivado = 0) \
     FROM proveedor p";

fn leer_fila(f: &Row) -> rusqlite::Result<FilaProveedor> {
    Ok(FilaProveedor {
        id: f.get(0)?,
        nombre: f.get(1)?,
        nif: f.get(2)?,
        contacto: f.get(3)?,
        condiciones_pago: f.get(4)?,
        descuento_habitual_bp: f.get(5)?,
        notas: f.get(6)?,
        archivado: f.get(7)?,
        version: f.get(8)?,
        articulos: f.get(9)?,
    })
}

pub fn listar(con: &Connection, con_archivados: bool) -> Result<Vec<FilaProveedor>, ErrorApp> {
    let filtro = if con_archivados {
        ""
    } else {
        " WHERE p.archivado = 0"
    };
    let mut consulta = con.prepare(&format!(
        "{SELECCION}{filtro} ORDER BY p.archivado, p.nombre"
    ))?;
    let filas = consulta.query_map([], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaProveedor>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE p.id = ?1"), [id], leer_fila)
        .optional()?)
}

pub fn existe_nombre(
    con: &Connection,
    nombre: &str,
    excepto: Option<i64>,
) -> Result<bool, ErrorApp> {
    Ok(con.query_row(
        "SELECT EXISTS (SELECT 1 FROM proveedor WHERE nombre = ?1 AND id IS NOT ?2)",
        params![nombre, excepto],
        |f| f.get(0),
    )?)
}

pub fn insertar(con: &Connection, d: &DatosDeProveedor) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO proveedor (nombre, nif, contacto, condiciones_pago, descuento_habitual_bp, notas) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            d.nombre,
            d.nif.as_ref().map(Nif::como_texto),
            d.contacto,
            d.condiciones_pago,
            d.descuento_habitual.puntos_basicos(),
            d.notas
        ],
    )?;
    Ok(con.last_insert_rowid())
}

pub fn actualizar(
    con: &Connection,
    (id, version): (i64, i64),
    d: &DatosDeProveedor,
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE proveedor SET nombre = ?1, nif = ?2, contacto = ?3, condiciones_pago = ?4, \
         descuento_habitual_bp = ?5, notas = ?6, version = version + 1 WHERE id = ?7 AND version = ?8",
        params![
            d.nombre,
            d.nif.as_ref().map(Nif::como_texto),
            d.contacto,
            d.condiciones_pago,
            d.descuento_habitual.puntos_basicos(),
            d.notas,
            id,
            version
        ],
    )?;
    comprobar(con, "proveedor", cambiadas, id)
}

pub fn cambiar_archivado(
    con: &Connection,
    tabla: &str,
    id: i64,
    archivado: bool,
) -> Result<(), ErrorApp> {
    let sql = match tabla {
        "proveedor" => "UPDATE proveedor SET archivado = ?1, version = version + 1 WHERE id = ?2",
        "acabado" => "UPDATE acabado SET archivado = ?1, version = version + 1 WHERE id = ?2",
        "articulo" => "UPDATE articulo SET archivado = ?1, version = version + 1 WHERE id = ?2",
        _ => return Err(ErrorApp::Interno(format!("tabla sin archivo: {tabla}"))),
    };
    if con.execute(sql, params![archivado, id])? == 0 {
        return Err(ErrorApp::NoEncontrado);
    }
    Ok(())
}

/// Si no se cambió ninguna fila, distingue entre «no existe» y «otra persona la cambió».
pub fn comprobar(con: &Connection, tabla: &str, cambiadas: usize, id: i64) -> Result<(), ErrorApp> {
    if cambiadas == 1 {
        return Ok(());
    }
    let sql = match tabla {
        "proveedor" => "SELECT EXISTS (SELECT 1 FROM proveedor WHERE id = ?1)",
        "acabado" => "SELECT EXISTS (SELECT 1 FROM acabado WHERE id = ?1)",
        _ => "SELECT EXISTS (SELECT 1 FROM articulo WHERE id = ?1)",
    };
    let existe: bool = con.query_row(sql, [id], |f| f.get(0))?;
    Err(if existe {
        ErrorApp::Conflicto
    } else {
        ErrorApp::NoEncontrado
    })
}
