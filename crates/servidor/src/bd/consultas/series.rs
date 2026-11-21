use std::collections::BTreeMap;

use dominio::series::{CONFIRMAR, Referencia};
use rusqlite::{Connection, OptionalExtension, params};

use crate::errores::ErrorApp;

pub struct VersionGuardada {
    pub id: i64,
    pub codigo: String,
    pub fabricante: String,
    pub numero: i64,
    pub estado: String,
    pub origen_yaml: String,
    pub sha256: String,
    pub avisos: i64,
    pub creada_en: String,
}

pub struct NuevaVersion<'a> {
    pub codigo: &'a str,
    pub fabricante: &'a str,
    pub estado: &'a str,
    pub origen_yaml: &'a str,
    pub sha256: &'a str,
    pub avisos: i64,
    pub importada_por: i64,
    pub creada_en: &'a str,
}

const ULTIMAS: &str = "SELECT v.id, s.codigo, s.fabricante, v.numero, v.estado, v.origen_yaml, \
     v.sha256, v.avisos, v.creada_en FROM serie_version v JOIN serie s ON s.id = v.serie_id \
     WHERE v.numero = (SELECT max(numero) FROM serie_version WHERE serie_id = s.id)";

fn leer_version(fila: &rusqlite::Row) -> rusqlite::Result<VersionGuardada> {
    Ok(VersionGuardada {
        id: fila.get(0)?,
        codigo: fila.get(1)?,
        fabricante: fila.get(2)?,
        numero: fila.get(3)?,
        estado: fila.get(4)?,
        origen_yaml: fila.get(5)?,
        sha256: fila.get(6)?,
        avisos: fila.get(7)?,
        creada_en: fila.get(8)?,
    })
}

pub fn listar_ultimas(con: &Connection) -> Result<Vec<VersionGuardada>, ErrorApp> {
    let mut consulta = con.prepare(&format!("{ULTIMAS} ORDER BY s.codigo"))?;
    let filas = consulta.query_map([], leer_version)?;
    Ok(filas.collect::<Result<_, _>>()?)
}

pub fn buscar_ultima(con: &Connection, codigo: &str) -> Result<Option<VersionGuardada>, ErrorApp> {
    Ok(con
        .query_row(
            &format!("{ULTIMAS} AND s.codigo = ?1"),
            [codigo],
            leer_version,
        )
        .optional()?)
}

pub fn insertar_version(con: &Connection, nueva: &NuevaVersion) -> Result<(i64, i64), ErrorApp> {
    con.execute(
        "INSERT INTO serie (codigo, fabricante, creada_en) VALUES (?1, ?2, ?3) \
         ON CONFLICT (codigo) DO NOTHING",
        params![nueva.codigo, nueva.fabricante, nueva.creada_en],
    )?;
    let serie_id: i64 = con.query_row(
        "SELECT id FROM serie WHERE codigo = ?1",
        [nueva.codigo],
        |f| f.get(0),
    )?;
    let numero: i64 = con.query_row(
        "SELECT coalesce(max(numero), 0) + 1 FROM serie_version WHERE serie_id = ?1",
        [serie_id],
        |f| f.get(0),
    )?;
    con.execute(
        "INSERT INTO serie_version (serie_id, numero, estado, origen_yaml, sha256, avisos, \
         importada_por, creada_en) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            serie_id,
            numero,
            nueva.estado,
            nueva.origen_yaml,
            nueva.sha256,
            nueva.avisos,
            nueva.importada_por,
            nueva.creada_en
        ],
    )?;
    Ok((con.last_insert_rowid(), numero))
}

pub fn insertar_referencias(
    con: &Connection,
    version_id: i64,
    referencias: &BTreeMap<String, Referencia>,
) -> Result<(), ErrorApp> {
    let mut sentencia = con.prepare(
        "INSERT INTO serie_referencia (serie_version_id, codigo, referencia_texto, proveedor_texto) \
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    for (codigo, referencia) in referencias {
        let texto = referencia
            .referencia
            .valor()
            .map_or(CONFIRMAR, String::as_str);
        sentencia.execute(params![version_id, codigo, texto, referencia.proveedor])?;
    }
    Ok(())
}
