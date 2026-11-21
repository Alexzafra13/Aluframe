use dominio::acceso::Accion;
use dominio::series::{OpcionDeLinea, Serie};
use rusqlite::Connection;

use super::ajustes_presupuesto::{self, AjustesDePresupuesto};
use super::series::reconstruir;
use super::{Actor, autorizar};
use crate::bd::consultas::{acabados, articulos_con_precio, series};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Las listas del editor de línea: (valor, texto) de cada desplegable.
pub struct DatosDelEditor {
    pub series: Vec<(String, String)>,
    pub acabados: Vec<(String, String)>,
    pub vidrios: Vec<(String, String)>,
    pub ajustes: AjustesDePresupuesto,
    /// Opciones de la tipología elegida (lado de la cremona…).
    pub opciones: Vec<OpcionDeLinea>,
}

/// `«id de versión»|«tipología»`: así un solo desplegable elige serie y tipología.
pub fn valor_de_seleccion(version_id: i64, tipologia: &str) -> String {
    format!("{version_id}|{tipologia}")
}

pub fn leer_seleccion(texto: &str) -> Option<(i64, String)> {
    let (version, tipologia) = texto.split_once('|')?;
    Some((version.parse().ok()?, tipologia.to_owned()))
}

fn series_elegibles(con: &Connection, extra: Option<i64>) -> Result<Vec<(i64, Serie)>, ErrorApp> {
    let mut versiones = series::listar_ultimas(con)?;
    if let Some(id) = extra.filter(|id| !versiones.iter().any(|v| v.id == *id)) {
        versiones.extend(series::buscar_version(con, id)?);
    }
    versiones
        .into_iter()
        .map(|v| Ok((v.id, reconstruir(v)?.serie)))
        .collect()
}

pub async fn cargar_datos_del_editor(
    estado: &EstadoApp,
    actor: &Actor,
    seleccion: Option<(i64, String)>,
) -> Result<DatosDelEditor, ErrorApp> {
    autorizar(actor, Accion::GestionarClientes)?;
    estado.bd.leer(move |con| leer_datos(con, seleccion)).await
}

fn leer_datos(
    con: &Connection,
    seleccion: Option<(i64, String)>,
) -> Result<DatosDelEditor, ErrorApp> {
    let elegibles = series_elegibles(con, seleccion.as_ref().map(|(id, _)| *id))?;
    let mut series = Vec::new();
    let mut opciones = Vec::new();
    for (id, serie) in &elegibles {
        for t in &serie.tipologias {
            series.push((
                valor_de_seleccion(*id, &t.codigo),
                format!("{} · {}", serie.codigo, t.nombre),
            ));
            if seleccion.as_ref() == Some(&(*id, t.codigo.clone())) {
                opciones.clone_from(&t.opciones);
            }
        }
    }
    let primera = elegibles.first().and_then(|(_, s)| s.tipologias.first());
    if let (None, Some(t)) = (&seleccion, primera) {
        opciones.clone_from(&t.opciones);
    }
    let acabados = acabados::listar(con)?
        .into_iter()
        .filter(|a| !a.archivado)
        .map(|a| (a.id.to_string(), a.nombre))
        .collect();
    let vidrios = articulos_con_precio::vidrios(con)?
        .into_iter()
        .map(|v| {
            (
                v.id.to_string(),
                format!("{} · {}", v.referencia, v.descripcion),
            )
        })
        .collect();
    Ok(DatosDelEditor {
        series,
        acabados,
        vidrios,
        ajustes: ajustes_presupuesto::leer_de_la_base(con)?,
        opciones,
    })
}
