use dominio::catalogo::DatosDeArticulo;
use rusqlite::{Connection, OptionalExtension, Row, params};

use super::escapar_like;
use super::proveedores::comprobar;
use crate::errores::ErrorApp;

/// Un artículo con su precio en la tarifa vigente de su proveedor, si lo tiene.
#[derive(Debug, Clone)]
pub struct FilaArticulo {
    pub id: i64,
    pub proveedor_id: i64,
    pub proveedor: String,
    pub referencia: String,
    pub descripcion: String,
    pub tipo: String,
    pub unidad: String,
    pub longitud_barra_dmm: Option<i64>,
    pub unidades_por_caja: Option<i64>,
    pub con_acabado: bool,
    pub stock_minimo_mil: i64,
    pub archivado: bool,
    pub version: i64,
    pub precio_pu: Option<i64>,
}

const SELECCION: &str = "SELECT a.id, a.proveedor_id, p.nombre, a.referencia, a.descripcion, a.tipo, \
     a.unidad, a.longitud_barra_dmm, a.unidades_por_caja, a.con_acabado, a.stock_minimo_mil, \
     a.archivado, a.version, tp.precio_pu \
     FROM articulo a JOIN proveedor p ON p.id = a.proveedor_id \
     LEFT JOIN tarifa t ON t.proveedor_id = a.proveedor_id AND t.estado = 'vigente' \
     LEFT JOIN tarifa_precio tp ON tp.tarifa_id = t.id AND tp.articulo_id = a.id";

fn leer_fila(f: &Row) -> rusqlite::Result<FilaArticulo> {
    Ok(FilaArticulo {
        id: f.get(0)?,
        proveedor_id: f.get(1)?,
        proveedor: f.get(2)?,
        referencia: f.get(3)?,
        descripcion: f.get(4)?,
        tipo: f.get(5)?,
        unidad: f.get(6)?,
        longitud_barra_dmm: f.get(7)?,
        unidades_por_caja: f.get(8)?,
        con_acabado: f.get(9)?,
        stock_minimo_mil: f.get(10)?,
        archivado: f.get(11)?,
        version: f.get(12)?,
        precio_pu: f.get(13)?,
    })
}

pub struct FiltroArticulos {
    pub proveedor_id: Option<i64>,
    pub texto: String,
    pub archivados: bool,
}

pub fn listar(con: &Connection, filtro: &FiltroArticulos) -> Result<Vec<FilaArticulo>, ErrorApp> {
    let patron = format!("%{}%", escapar_like(filtro.texto.trim()));
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE a.archivado = ?1 AND (?2 IS NULL OR a.proveedor_id = ?2) \
         AND (a.referencia LIKE ?3 ESCAPE '\\' OR a.descripcion LIKE ?3 ESCAPE '\\') ORDER BY p.nombre, a.referencia LIMIT 1000"
    ))?;
    let filas = consulta.query_map(
        params![filtro.archivados, filtro.proveedor_id, patron],
        leer_fila,
    )?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaArticulo>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE a.id = ?1"), [id], leer_fila)
        .optional()?)
}

pub fn de_proveedor(con: &Connection, proveedor_id: i64) -> Result<Vec<FilaArticulo>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE a.proveedor_id = ?1 AND a.archivado = 0 ORDER BY a.referencia"
    ))?;
    let filas = consulta.query_map([proveedor_id], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn existe_referencia(
    con: &Connection,
    (proveedor_id, referencia): (i64, &str),
    excepto: Option<i64>,
) -> Result<bool, ErrorApp> {
    Ok(con.query_row(
        "SELECT EXISTS (SELECT 1 FROM articulo WHERE proveedor_id = ?1 AND referencia = ?2 AND id IS NOT ?3)",
        params![proveedor_id, referencia, excepto],
        |f| f.get(0),
    )?)
}

pub fn insertar(
    con: &Connection,
    proveedor_id: i64,
    d: &DatosDeArticulo,
    (creado_en, importacion_id): (&str, Option<i64>),
) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO articulo (proveedor_id, referencia, descripcion, tipo, unidad, longitud_barra_dmm, \
         unidades_por_caja, con_acabado, stock_minimo_mil, importacion_id, creado_en) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            proveedor_id,
            d.referencia,
            d.descripcion,
            d.tipo.como_texto(),
            d.unidad.como_texto(),
            d.longitud_barra.map(|l| l.decimas_de_mm()),
            d.unidades_por_caja,
            d.con_acabado,
            d.stock_minimo_mil,
            importacion_id,
            creado_en
        ],
    )?;
    Ok(con.last_insert_rowid())
}

pub fn actualizar(
    con: &Connection,
    (id, version): (i64, i64),
    d: &DatosDeArticulo,
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE articulo SET referencia = ?1, descripcion = ?2, tipo = ?3, unidad = ?4, \
         longitud_barra_dmm = ?5, unidades_por_caja = ?6, con_acabado = ?7, stock_minimo_mil = ?8, \
         version = version + 1 WHERE id = ?9 AND version = ?10",
        params![
            d.referencia,
            d.descripcion,
            d.tipo.como_texto(),
            d.unidad.como_texto(),
            d.longitud_barra.map(|l| l.decimas_de_mm()),
            d.unidades_por_caja,
            d.con_acabado,
            d.stock_minimo_mil,
            id,
            version
        ],
    )?;
    comprobar(con, "articulo", cambiadas, id)
}

/// La tarifa manda en descripción, unidad y longitud. Sube la versión para que quien tenga la
/// ficha abierta reciba el aviso de cambio en lugar de pisar los datos nuevos.
pub fn actualizar_desde_tarifa(
    con: &Connection,
    id: i64,
    d: &DatosDeArticulo,
) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE articulo SET descripcion = ?1, tipo = ?2, unidad = ?3, longitud_barra_dmm = ?4, \
         unidades_por_caja = ?5, version = version + 1 WHERE id = ?6",
        params![
            d.descripcion,
            d.tipo.como_texto(),
            d.unidad.como_texto(),
            d.longitud_barra.map(|l| l.decimas_de_mm()),
            d.unidades_por_caja,
            id
        ],
    )?;
    Ok(())
}
