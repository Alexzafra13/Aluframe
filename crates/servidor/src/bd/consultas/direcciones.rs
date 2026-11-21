use dominio::clientes::DatosDeDireccion;
use rusqlite::{Connection, Row, params};

use crate::errores::ErrorApp;

#[derive(Debug, Clone)]
pub struct FilaDireccion {
    pub id: i64,
    pub cliente_id: i64,
    pub datos: DatosDeDireccion,
}

fn leer_fila(fila: &Row) -> rusqlite::Result<FilaDireccion> {
    Ok(FilaDireccion {
        id: fila.get(0)?,
        cliente_id: fila.get(1)?,
        datos: DatosDeDireccion {
            etiqueta: fila.get(2)?,
            calle: fila.get(3)?,
            cp: fila.get(4)?,
            localidad: fila.get(5)?,
            provincia: fila.get(6)?,
            contacto_nombre: fila.get(7)?,
            contacto_telefono: fila.get(8)?,
            notas: fila.get(9)?,
        },
    })
}

const SELECCION: &str = "SELECT id, cliente_id, etiqueta, calle, cp, localidad, provincia, \
     contacto_nombre, contacto_telefono, notas FROM direccion_montaje";

pub fn listar(con: &Connection, cliente_id: i64) -> Result<Vec<FilaDireccion>, ErrorApp> {
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE cliente_id = ?1 AND archivado = 0 ORDER BY id"
    ))?;
    let filas = consulta.query_map([cliente_id], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn insertar(con: &Connection, cliente_id: i64, d: &DatosDeDireccion) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO direccion_montaje (cliente_id, etiqueta, calle, cp, localidad, provincia, \
         contacto_nombre, contacto_telefono, notas) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            cliente_id,
            d.etiqueta,
            d.calle,
            d.cp,
            d.localidad,
            d.provincia,
            d.contacto_nombre,
            d.contacto_telefono,
            d.notas
        ],
    )?;
    Ok(con.last_insert_rowid())
}

/// Quitar una dirección la archiva: más adelante la usarán presupuestos ya emitidos.
pub fn archivar(con: &Connection, cliente_id: i64, id: i64) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE direccion_montaje SET archivado = 1 WHERE id = ?1 AND cliente_id = ?2",
        params![id, cliente_id],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::NoEncontrado);
    }
    Ok(())
}

pub fn actualizar(
    con: &Connection,
    (cliente_id, id): (i64, i64),
    d: &DatosDeDireccion,
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE direccion_montaje SET etiqueta = ?1, calle = ?2, cp = ?3, localidad = ?4, \
         provincia = ?5, contacto_nombre = ?6, contacto_telefono = ?7, notas = ?8 \
         WHERE id = ?9 AND cliente_id = ?10 AND archivado = 0",
        params![
            d.etiqueta,
            d.calle,
            d.cp,
            d.localidad,
            d.provincia,
            d.contacto_nombre,
            d.contacto_telefono,
            d.notas,
            id,
            cliente_id
        ],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::NoEncontrado);
    }
    Ok(())
}
