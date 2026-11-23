use std::collections::HashSet;

use dominio::clientes::DatosDeCliente;
use dominio::nif::Nif;
use rusqlite::{Connection, OptionalExtension, Row, params};

use super::escapar_like;
use crate::errores::ErrorApp;

/// Un cliente tal como está guardado; los textos ya vienen validados del dominio.
#[derive(Debug, Clone, Default)]
pub struct FilaCliente {
    pub id: i64,
    pub tipo: String,
    pub nombre: Option<String>,
    pub nif: Option<String>,
    pub email: Option<String>,
    pub telefono: Option<String>,
    pub fiscal_calle: Option<String>,
    pub fiscal_cp: Option<String>,
    pub fiscal_localidad: Option<String>,
    pub fiscal_provincia: Option<String>,
    pub fiscal_pais: Option<String>,
    pub forma_de_pago: Option<String>,
    pub notas: Option<String>,
    pub archivado: bool,
    pub creado_en: String,
    pub version: i64,
}

const COLUMNAS: &str = "id, tipo, nombre_o_razon_social, nif, email, telefono, fiscal_calle, \
     fiscal_cp, fiscal_localidad, fiscal_provincia, fiscal_pais, forma_de_pago, notas, archivado, \
     creado_en, version";

fn leer_fila(fila: &Row) -> rusqlite::Result<FilaCliente> {
    Ok(FilaCliente {
        id: fila.get(0)?,
        tipo: fila.get(1)?,
        nombre: fila.get(2)?,
        nif: fila.get(3)?,
        email: fila.get(4)?,
        telefono: fila.get(5)?,
        fiscal_calle: fila.get(6)?,
        fiscal_cp: fila.get(7)?,
        fiscal_localidad: fila.get(8)?,
        fiscal_provincia: fila.get(9)?,
        fiscal_pais: fila.get(10)?,
        forma_de_pago: fila.get(11)?,
        notas: fila.get(12)?,
        archivado: fila.get(13)?,
        creado_en: fila.get(14)?,
        version: fila.get(15)?,
    })
}

pub struct Pagina {
    pub filtro: String,
    pub archivados: bool,
    pub limite: i64,
    pub desplazamiento: i64,
}

/// El filtro ya viene normalizado (sin tildes ni mayúsculas), como la columna `busqueda`.
pub fn listar(con: &Connection, pagina: &Pagina) -> Result<(Vec<FilaCliente>, i64), ErrorApp> {
    let patron = format!("%{}%", escapar_like(&pagina.filtro));
    let condicion = "archivado = ?1 AND busqueda LIKE ?2 ESCAPE '\\'";
    let total: i64 = con.query_row(
        &format!("SELECT count(*) FROM cliente WHERE {condicion}"),
        params![pagina.archivados, patron],
        |f| f.get(0),
    )?;
    let mut consulta = con.prepare(&format!(
        "SELECT {COLUMNAS} FROM cliente WHERE {condicion} \
         ORDER BY coalesce(nombre_o_razon_social, '') = '', busqueda, id LIMIT ?3 OFFSET ?4"
    ))?;
    let filas = consulta.query_map(
        params![
            pagina.archivados,
            patron,
            pagina.limite,
            pagina.desplazamiento
        ],
        leer_fila,
    )?;
    Ok((filas.collect::<Result<_, _>>()?, total))
}

/// Todos los activos, con el mismo orden que la lista, para un desplegable.
pub fn para_elegir(con: &Connection) -> Result<Vec<(i64, Option<String>)>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT id, nombre_o_razon_social FROM cliente WHERE archivado = 0 \
         ORDER BY coalesce(nombre_o_razon_social, '') = '', busqueda, id",
    )?;
    let filas = consulta.query_map([], |f| Ok((f.get(0)?, f.get(1)?)))?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaCliente>, ErrorApp> {
    Ok(con
        .query_row(
            &format!("SELECT {COLUMNAS} FROM cliente WHERE id = ?1"),
            [id],
            leer_fila,
        )
        .optional()?)
}

/// Otro cliente con ese NIF, para no duplicar fichas.
pub fn buscar_por_nif(
    con: &Connection,
    nif: &str,
    excepto: Option<i64>,
) -> Result<Option<FilaCliente>, ErrorApp> {
    Ok(con
        .query_row(
            &format!("SELECT {COLUMNAS} FROM cliente WHERE nif = ?1 AND id IS NOT ?2 LIMIT 1"),
            params![nif, excepto],
            leer_fila,
        )
        .optional()?)
}

pub fn nifs_existentes(con: &Connection) -> Result<HashSet<String>, ErrorApp> {
    let mut consulta = con.prepare("SELECT nif FROM cliente WHERE nif IS NOT NULL")?;
    let filas = consulta.query_map([], |f| f.get(0))?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn insertar(
    con: &Connection,
    datos: &DatosDeCliente,
    creado_en: &str,
    importacion_id: Option<i64>,
) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO cliente (tipo, nombre_o_razon_social, nif, email, telefono, fiscal_calle, \
         fiscal_cp, fiscal_localidad, fiscal_provincia, fiscal_pais, forma_de_pago, notas, \
         busqueda, importacion_id, creado_en) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            datos.tipo.como_texto(),
            datos.nombre,
            datos.nif.as_ref().map(Nif::como_texto),
            datos.email,
            datos.telefono,
            datos.fiscal_calle,
            datos.fiscal_cp,
            datos.fiscal_localidad,
            datos.fiscal_provincia,
            datos.fiscal_pais,
            datos.forma_de_pago,
            datos.notas,
            datos.texto_de_busqueda(),
            importacion_id,
            creado_en,
        ],
    )?;
    Ok(con.last_insert_rowid())
}

/// Bloqueo optimista: si otra persona guardó antes, la versión ya no coincide.
pub fn actualizar(
    con: &Connection,
    id: i64,
    version: i64,
    datos: &DatosDeCliente,
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE cliente SET tipo = ?1, nombre_o_razon_social = ?2, nif = ?3, email = ?4, \
         telefono = ?5, fiscal_calle = ?6, fiscal_cp = ?7, fiscal_localidad = ?8, \
         fiscal_provincia = ?9, fiscal_pais = ?10, forma_de_pago = ?11, notas = ?12, \
         busqueda = ?13, version = version + 1 WHERE id = ?14 AND version = ?15",
        params![
            datos.tipo.como_texto(),
            datos.nombre,
            datos.nif.as_ref().map(Nif::como_texto),
            datos.email,
            datos.telefono,
            datos.fiscal_calle,
            datos.fiscal_cp,
            datos.fiscal_localidad,
            datos.fiscal_provincia,
            datos.fiscal_pais,
            datos.forma_de_pago,
            datos.notas,
            datos.texto_de_busqueda(),
            id,
            version,
        ],
    )?;
    comprobar_version(con, cambiadas, id)
}

fn comprobar_version(con: &Connection, cambiadas: usize, id: i64) -> Result<(), ErrorApp> {
    if cambiadas == 1 {
        return Ok(());
    }
    match buscar(con, id)? {
        Some(_) => Err(ErrorApp::Conflicto),
        None => Err(ErrorApp::NoEncontrado),
    }
}

pub fn cambiar_archivado(con: &Connection, id: i64, archivado: bool) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE cliente SET archivado = ?1, version = version + 1 WHERE id = ?2",
        params![archivado, id],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::NoEncontrado);
    }
    Ok(())
}

/// Borrado definitivo (RGPD): primero sus direcciones de montaje.
pub fn borrar(con: &Connection, id: i64) -> Result<(), ErrorApp> {
    con.execute("DELETE FROM direccion_montaje WHERE cliente_id = ?1", [id])?;
    if con.execute("DELETE FROM cliente WHERE id = ?1", [id])? == 0 {
        return Err(ErrorApp::NoEncontrado);
    }
    Ok(())
}

/// Quita los clientes de una importación que nadie ha tocado; devuelve (quitados, conservados).
pub fn quitar_importados_sin_cambios(
    con: &Connection,
    importacion_id: i64,
) -> Result<(usize, usize), ErrorApp> {
    let quitados = con.execute(
        "DELETE FROM cliente WHERE importacion_id = ?1 AND version = 1 \
         AND NOT EXISTS (SELECT 1 FROM direccion_montaje d WHERE d.cliente_id = cliente.id)",
        [importacion_id],
    )?;
    let conservados = con.execute(
        "UPDATE cliente SET importacion_id = NULL WHERE importacion_id = ?1",
        [importacion_id],
    )?;
    Ok((quitados, conservados))
}
