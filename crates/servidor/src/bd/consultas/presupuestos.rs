use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::errores::ErrorApp;

/// Cabecera con la versión vigente y el nombre del cliente para listas y pantallas.
#[derive(Debug, Clone)]
pub struct FilaPresupuesto {
    pub id: i64,
    pub anio: i64,
    pub numero: i64,
    pub cliente_id: i64,
    pub cliente_nombre: Option<String>,
    pub direccion_montaje_id: Option<i64>,
    pub estado: String,
    pub validez_hasta: Option<String>,
    pub condiciones: Option<String>,
    pub descuento_bp: i64,
    pub creado_en: String,
    pub version: i64,
    pub version_id: i64,
    pub numero_version: i64,
    pub base_cent: i64,
    pub iva_cent: i64,
    pub total_cent: i64,
    pub completo: bool,
    pub copia_json: Option<String>,
}

impl FilaPresupuesto {
    pub fn codigo(&self) -> String {
        format!("P-{}-{:04}", self.anio, self.numero)
    }
}

const SELECCION: &str = "SELECT p.id, p.anio, p.numero, p.cliente_id, c.nombre_o_razon_social, \
     p.direccion_montaje_id, p.estado, p.validez_hasta, p.condiciones, p.descuento_bp, p.creado_en, \
     p.version, v.id, v.numero_version, v.base_cent, v.iva_cent, v.total_cent, v.completo, p.copia_json \
     FROM presupuesto p JOIN cliente c ON c.id = p.cliente_id \
     JOIN version_presupuesto v ON v.presupuesto_id = p.id AND v.vigente = 1";

fn leer(f: &Row) -> rusqlite::Result<FilaPresupuesto> {
    Ok(FilaPresupuesto {
        id: f.get(0)?,
        anio: f.get(1)?,
        numero: f.get(2)?,
        cliente_id: f.get(3)?,
        cliente_nombre: f.get(4)?,
        direccion_montaje_id: f.get(5)?,
        estado: f.get(6)?,
        validez_hasta: f.get(7)?,
        condiciones: f.get(8)?,
        descuento_bp: f.get(9)?,
        creado_en: f.get(10)?,
        version: f.get(11)?,
        version_id: f.get(12)?,
        numero_version: f.get(13)?,
        base_cent: f.get(14)?,
        iva_cent: f.get(15)?,
        total_cent: f.get(16)?,
        completo: f.get(17)?,
        copia_json: f.get(18)?,
    })
}

/// Lo justo para decidir la situación de cada presupuesto (dominio::presupuesto), del más nuevo
/// al más antiguo: identificador, estado y validez.
pub fn estados_y_validez(
    con: &Connection,
    cliente: Option<i64>,
) -> Result<Vec<(i64, String, Option<String>)>, ErrorApp> {
    let mut consulta = con.prepare(
        "SELECT id, estado, validez_hasta FROM presupuesto \
         WHERE (?1 IS NULL OR cliente_id = ?1) ORDER BY anio DESC, numero DESC",
    )?;
    let filas = consulta.query_map([cliente], |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)))?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Los presupuestos con esos identificadores, del más nuevo al más antiguo.
pub fn buscar_varios(con: &Connection, ids: &[i64]) -> Result<Vec<FilaPresupuesto>, ErrorApp> {
    let lista = serde_json::to_string(ids).map_err(|e| ErrorApp::Interno(e.to_string()))?;
    let mut consulta = con.prepare(&format!(
        "{SELECCION} WHERE p.id IN (SELECT value FROM json_each(?1)) \
         ORDER BY p.anio DESC, p.numero DESC"
    ))?;
    let filas = consulta.query_map([lista], leer)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

/// Renovar (ADR-0027): la validez nueva cuenta desde hoy.
pub fn cambiar_validez(con: &Connection, id: i64, validez: Option<&str>) -> Result<(), ErrorApp> {
    con.execute(
        "UPDATE presupuesto SET validez_hasta = ?2, version = version + 1 WHERE id = ?1",
        params![id, validez],
    )?;
    Ok(())
}

pub fn buscar(con: &Connection, id: i64) -> Result<Option<FilaPresupuesto>, ErrorApp> {
    Ok(con
        .query_row(&format!("{SELECCION} WHERE p.id = ?1"), [id], leer)
        .optional()?)
}

/// Presupuestos y trabajos se numeran al crearse y pueden tener huecos (BACKEND 5.6).
pub fn siguiente_numero(con: &Connection, serie: &str, anio: i64) -> Result<i64, ErrorApp> {
    Ok(con.query_row(
        "INSERT INTO secuencia_documentos (serie, anio, ultimo) VALUES (?1, ?2, 1) \
         ON CONFLICT (serie, anio) DO UPDATE SET ultimo = ultimo + 1 RETURNING ultimo",
        params![serie, anio],
        |f| f.get(0),
    )?)
}

pub struct NuevoPresupuesto<'a> {
    pub anio: i64,
    pub numero: i64,
    pub cliente_id: i64,
    pub direccion_montaje_id: Option<i64>,
    pub validez_hasta: Option<&'a str>,
    pub condiciones: Option<&'a str>,
    pub creado_por: i64,
    pub ahora: &'a str,
}

pub fn insertar(con: &Connection, p: &NuevoPresupuesto) -> Result<i64, ErrorApp> {
    con.execute(
        "INSERT INTO presupuesto (anio, numero, cliente_id, direccion_montaje_id, validez_hasta, \
         condiciones, creado_por, creado_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            p.anio,
            p.numero,
            p.cliente_id,
            p.direccion_montaje_id,
            p.validez_hasta,
            p.condiciones,
            p.creado_por,
            p.ahora
        ],
    )?;
    let id = con.last_insert_rowid();
    con.execute(
        "INSERT INTO version_presupuesto (presupuesto_id, numero_version, vigente, creada_en) VALUES (?1, 1, 1, ?2)",
        params![id, p.ahora],
    )?;
    Ok(id)
}

pub struct Cabecera<'a> {
    pub direccion_montaje_id: Option<i64>,
    pub validez_hasta: Option<&'a str>,
    pub condiciones: Option<&'a str>,
    pub descuento_bp: i64,
}

pub fn actualizar_cabecera(
    con: &Connection,
    (id, version): (i64, i64),
    c: &Cabecera,
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE presupuesto SET direccion_montaje_id = ?1, validez_hasta = ?2, condiciones = ?3, \
         descuento_bp = ?4, version = version + 1 WHERE id = ?5 AND version = ?6",
        params![
            c.direccion_montaje_id,
            c.validez_hasta,
            c.condiciones,
            c.descuento_bp,
            id,
            version
        ],
    )?;
    comprobar_presupuesto(con, cambiadas, id)
}

fn comprobar_presupuesto(con: &Connection, cambiadas: usize, id: i64) -> Result<(), ErrorApp> {
    if cambiadas == 1 {
        return Ok(());
    }
    let existe: bool = con.query_row(
        "SELECT EXISTS (SELECT 1 FROM presupuesto WHERE id = ?1)",
        [id],
        |f| f.get(0),
    )?;
    Err(if existe {
        ErrorApp::Conflicto
    } else {
        ErrorApp::NoEncontrado
    })
}

pub fn cambiar_estado(
    con: &Connection,
    id: i64,
    estado: &str,
    ahora: &str,
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE presupuesto SET estado = ?1, version = version + 1, \
         enviado_en = CASE WHEN ?1 = 'enviado' THEN ?2 ELSE enviado_en END WHERE id = ?3",
        params![estado, ahora, id],
    )?;
    comprobar_presupuesto(con, cambiadas, id)
}

pub fn aceptar(
    con: &Connection,
    id: i64,
    (copia, sha256, ahora): (&str, &str, &str),
) -> Result<(), ErrorApp> {
    let cambiadas = con.execute(
        "UPDATE presupuesto SET estado = 'aceptado', aceptado_en = ?1, copia_json = ?2, copia_sha256 = ?3, \
         version = version + 1 WHERE id = ?4 AND estado IN ('borrador', 'enviado')",
        params![ahora, copia, sha256, id],
    )?;
    if cambiadas == 0 {
        return Err(ErrorApp::Validacion(
            "Este presupuesto ya no se puede aceptar.".into(),
        ));
    }
    Ok(())
}

pub fn anotar_tarifas(
    con: &Connection,
    presupuesto_id: i64,
    tarifas: &[i64],
) -> Result<(), ErrorApp> {
    for tarifa in tarifas {
        con.execute(
            "INSERT OR IGNORE INTO presupuesto_tarifa (presupuesto_id, tarifa_id) VALUES (?1, ?2)",
            params![presupuesto_id, tarifa],
        )?;
    }
    Ok(())
}

pub fn tarifa_usada(con: &Connection, tarifa_id: i64) -> Result<bool, ErrorApp> {
    Ok(con.query_row(
        "SELECT EXISTS (SELECT 1 FROM presupuesto_tarifa WHERE tarifa_id = ?1)",
        [tarifa_id],
        |f| f.get(0),
    )?)
}

pub fn tiene_presupuestos(con: &Connection, cliente_id: i64) -> Result<bool, ErrorApp> {
    Ok(con.query_row(
        "SELECT EXISTS (SELECT 1 FROM presupuesto WHERE cliente_id = ?1)",
        [cliente_id],
        |f| f.get(0),
    )?)
}
