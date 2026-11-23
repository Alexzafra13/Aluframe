//! Apuntes (ADR-0025): notas y tareas sueltas.

use dominio::apuntes::DatosDeApunte;
use dominio::clientes::{DatosDeDireccion, nombre_visible};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

/// Lo que se enseña de un apunte, con los nombres ya resueltos.
#[derive(Debug, Clone)]
pub struct FilaApunte {
    pub id: i64,
    pub autor_id: i64,
    pub autor: String,
    pub para_id: Option<i64>,
    pub para: Option<String>,
    pub cliente_id: Option<i64>,
    pub cliente: Option<String>,
    pub direccion_id: Option<i64>,
    pub direccion: Option<String>,
    pub contacto: Option<String>,
    pub texto: String,
    pub medidas: Option<String>,
    pub estado: String,
    pub presupuesto_id: Option<i64>,
    pub creado_en: String,
    pub version: i64,
}

const SELECCION: &str = "SELECT a.id, a.autor_id, au.nombre, a.para_id, pa.nombre, a.cliente_id, \
     c.nombre_o_razon_social, a.direccion_id, d.etiqueta, d.calle, d.localidad, a.contacto, \
     a.texto, a.medidas, a.estado, a.presupuesto_id, a.creado_en, a.version \
     FROM apunte a JOIN usuario au ON au.id = a.autor_id \
     LEFT JOIN usuario pa ON pa.id = a.para_id \
     LEFT JOIN cliente c ON c.id = a.cliente_id \
     LEFT JOIN direccion_montaje d ON d.id = a.direccion_id";

fn leer_fila(fila: &Row) -> rusqlite::Result<FilaApunte> {
    let cliente_id: Option<i64> = fila.get(5)?;
    let nombre_cliente: Option<String> = fila.get(6)?;
    let direccion_id: Option<i64> = fila.get(7)?;
    let direccion = direccion_id.map(|_| -> rusqlite::Result<String> {
        let datos = DatosDeDireccion {
            etiqueta: fila.get(8)?,
            calle: fila.get(9)?,
            localidad: fila.get(10)?,
            ..DatosDeDireccion::default()
        };
        Ok(datos.resumen())
    });
    Ok(FilaApunte {
        id: fila.get(0)?,
        autor_id: fila.get(1)?,
        autor: fila.get(2)?,
        para_id: fila.get(3)?,
        para: fila.get(4)?,
        cliente: cliente_id.map(|id| nombre_visible(id, nombre_cliente.as_deref())),
        cliente_id,
        direccion: direccion.transpose()?,
        direccion_id,
        contacto: fila.get(11)?,
        texto: fila.get(12)?,
        medidas: fila.get(13)?,
        estado: fila.get(14)?,
        presupuesto_id: fila.get(15)?,
        creado_en: fila.get(16)?,
        version: fila.get(17)?,
    })
}

pub struct NuevoApunte<'a> {
    pub autor_id: i64,
    pub para_id: Option<i64>,
    pub cliente_id: Option<i64>,
    pub direccion_id: Option<i64>,
    pub datos: &'a DatosDeApunte,
    pub creado_en: &'a str,
}

pub fn insertar(con: &Connection, n: &NuevoApunte) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO apunte (autor_id, para_id, cliente_id, direccion_id, contacto, texto, \
         medidas, creado_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            n.autor_id,
            n.para_id,
            n.cliente_id,
            n.direccion_id,
            n.datos.contacto,
            n.datos.texto,
            n.datos.medidas,
            n.creado_en
        ],
    )?;
    Ok(con.last_insert_rowid())
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaApunte>, ErrorApp> {
    let sql = format!("{SELECCION} WHERE a.id = ?1");
    Ok(con.query_row(&sql, [id], leer_fila).optional()?)
}

/// Qué apuntes listar: abiertos (pendientes) o cerrados, de todos o de un usuario (los que
/// escribió y los que le mandan), y opcionalmente de un cliente.
pub struct FiltroDeApuntes {
    pub abiertos: bool,
    pub de_usuario: Option<i64>,
    pub cliente: Option<i64>,
}

/// Los 200 más recientes: los antiguos ya no hacen falta en pantalla y siguen en la base.
pub fn listar(con: &Connection, f: &FiltroDeApuntes) -> Result<Vec<FilaApunte>, ErrorApp> {
    let estado = if f.abiertos {
        "a.estado = 'pendiente'"
    } else {
        "a.estado <> 'pendiente'"
    };
    let sql = format!(
        "{SELECCION} WHERE {estado} \
         AND (?1 IS NULL OR a.autor_id = ?1 OR a.para_id = ?1) \
         AND (?2 IS NULL OR a.cliente_id = ?2) \
         ORDER BY a.creado_en DESC, a.id DESC LIMIT 200"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map(params![f.de_usuario, f.cliente], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Las tareas pendientes que el dueño ha mandado a un operario, para su «Mi día».
pub fn tareas_de(con: &Connection, usuario: i64) -> Result<Vec<FilaApunte>, ErrorApp> {
    let sql = format!(
        "{SELECCION} WHERE a.estado = 'pendiente' AND a.para_id = ?1 ORDER BY a.creado_en, a.id"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([usuario], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Los pendientes para el dueño (sin destinatario), para la campana y el Inicio.
pub fn pendientes_para_el_dueno(con: &Connection) -> Result<Vec<FilaApunte>, ErrorApp> {
    let sql = format!(
        "{SELECCION} WHERE a.estado = 'pendiente' AND a.para_id IS NULL ORDER BY a.creado_en, a.id"
    );
    let mut consulta = con.prepare(&sql)?;
    let filas = consulta.query_map([], leer_fila)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub struct Cierre<'a> {
    pub id: i64,
    pub version: i64,
    pub estado: &'a str,
    pub cuando: Option<&'a str>,
    pub quien: Option<i64>,
    pub presupuesto_id: Option<i64>,
}

/// `false` si otra persona lo cambió antes (versión distinta).
pub fn cambiar_estado(con: &Connection, c: &Cierre) -> Result<bool, ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE apunte SET estado = ?3, cerrado_en = ?4, cerrado_por = ?5, \
         presupuesto_id = coalesce(?6, presupuesto_id), version = version + 1 \
         WHERE id = ?1 AND version = ?2",
        params![
            c.id,
            c.version,
            c.estado,
            c.cuando,
            c.quien,
            c.presupuesto_id
        ],
    )?;
    Ok(cambiadas == 1)
}
