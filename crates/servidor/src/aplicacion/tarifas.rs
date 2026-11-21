use std::collections::BTreeMap;

use dominio::acceso::Accion;
use dominio::tarifa::validar_ajuste;
use dominio::unidades::{Porcentaje, PrecioUnitario};
use rusqlite::{Connection, Transaction};
use time::OffsetDateTime;

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::tarifas::{self, FilaTarifa, NuevaTarifa, PrecioDeTarifa};
use crate::bd::consultas::{articulos, importaciones, proveedores};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{formatear_para_pantalla, formatear_utc, leer_utc};
use crate::vistas::{PrecioHistorico, precio};

/// Cabecera de una versión nueva de tarifa.
pub struct DatosDeVersion {
    pub proveedor_id: i64,
    pub nombre: String,
    pub fecha_tarifa: Option<String>,
    pub descuento_compra_bp: Option<i64>,
    pub formato_origen: &'static str,
    pub archivo_origen: Option<String>,
    pub importacion_id: Option<i64>,
}

/// Crea una versión completa: copia los precios vigentes y aplica los nuevos (ADR-0012).
pub fn crear_version(
    tx: &Connection,
    datos: &DatosDeVersion,
    nuevos: &BTreeMap<i64, i64>,
    (ahora, autor): (&str, i64),
) -> Result<i64, ErrorApp> {
    let anterior = tarifas::vigente(tx, datos.proveedor_id)?;
    let mut precios = match &anterior {
        Some(t) => tarifas::precios(tx, t.id)?,
        None => BTreeMap::new(),
    };
    precios.extend(nuevos.iter().map(|(a, p)| (*a, *p)));
    let descuento = match (datos.descuento_compra_bp, &anterior) {
        (Some(d), _) => d,
        (None, Some(t)) => t.descuento_compra_bp,
        (None, None) => {
            proveedores::buscar(tx, datos.proveedor_id)?.map_or(0, |p| p.descuento_habitual_bp)
        }
    };
    let nueva = NuevaTarifa {
        proveedor_id: datos.proveedor_id,
        nombre: &datos.nombre,
        fecha_tarifa: datos.fecha_tarifa.as_deref(),
        descuento_compra_bp: descuento,
        formato_origen: datos.formato_origen,
        archivo_origen: datos.archivo_origen.as_deref(),
        importacion_id: datos.importacion_id,
        ahora,
        usuario_id: autor,
    };
    tarifas::insertar(tx, &nueva, &precios)
}

pub async fn listar_tarifas(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<FilaTarifa>, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado.bd.leer(tarifas::listar).await
}

pub async fn cargar_tarifa(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<(FilaTarifa, Vec<PrecioDeTarifa>), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado
        .bd
        .leer(move |con| {
            let tarifa = tarifas::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
            Ok((tarifa, tarifas::precios_con_articulo(con, id)?))
        })
        .await
}

pub fn texto_de_origen(formato: &str) -> &'static str {
    match formato {
        "csv" => "importada (CSV)",
        "ajuste" => "ajuste por porcentaje",
        _ => "edición en la tabla",
    }
}

pub async fn historial_de_precios(
    estado: &EstadoApp,
    actor: &Actor,
    articulo_id: i64,
) -> Result<Vec<PrecioHistorico>, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let filas = estado
        .bd
        .leer(move |con| tarifas::historial(con, articulo_id))
        .await?;
    Ok(filas
        .iter()
        .map(|h| PrecioHistorico {
            tarifa: h.tarifa.clone(),
            fecha: leer_utc(&h.vigente_desde)
                .map(formatear_para_pantalla)
                .unwrap_or_default(),
            origen: texto_de_origen(&h.formato_origen),
            precio: precio(Some(h.precio_pu)),
            vigente: h.estado == "vigente",
        })
        .collect())
}

async fn escribir_version<T: Send + 'static>(
    estado: &EstadoApp,
    actor: &Actor,
    accion: &'static str,
    operacion: impl FnOnce(&Transaction, (&str, i64)) -> Result<(T, i64), ErrorApp> + Send + 'static,
) -> Result<T, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let (ahora, autor): (OffsetDateTime, i64) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let (resultado, tarifa_id) = operacion(tx, (&formatear_utc(ahora), autor))?;
            if tarifa_id > 0 {
                let cambio = Cambio {
                    ahora,
                    autor,
                    accion,
                    entidad: "tarifa",
                    entidad_id: tarifa_id,
                };
                anotar(tx, &cambio)?;
            }
            Ok(resultado)
        })
        .await
}

/// Deshace la versión vigente de un proveedor: vuelve la anterior y se quitan los artículos
/// que creó su importación si nada más los usa.
pub async fn deshacer_tarifa(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FilaTarifa, ErrorApp> {
    escribir_version(estado, actor, "deshacer_tarifa", move |tx, (ahora, _)| {
        let tarifa = tarifas::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        if tarifa.estado != "vigente" {
            return Err(ErrorApp::Validacion(
                "Solo se puede deshacer la última versión de la tarifa de cada proveedor.".into(),
            ));
        }
        if crate::bd::consultas::presupuestos::tarifa_usada(tx, id)? {
            return Err(ErrorApp::Validacion(
                "Esta versión ya se usa en un presupuesto aceptado y no se puede deshacer.".into(),
            ));
        }
        tarifas::borrar_vigente(tx, &tarifa)?;
        if let Some(importacion) = tarifa.importacion_id {
            importaciones::marcar_deshecha(tx, importacion, ahora)?;
            tarifas::quitar_articulos_importados(tx, importacion)?;
        }
        Ok((tarifa, id))
    })
    .await
}

/// Sube o baja un porcentaje los artículos marcados de un proveedor.
pub async fn aplicar_ajuste(
    estado: &EstadoApp,
    actor: &Actor,
    (proveedor_id, articulo_ids): (i64, Vec<i64>),
    porcentaje: Porcentaje,
) -> Result<usize, ErrorApp> {
    validar_ajuste(porcentaje).map_err(|m| ErrorApp::Validacion(m.into()))?;
    if articulo_ids.is_empty() {
        return Err(ErrorApp::Validacion("Marca al menos un artículo.".into()));
    }
    escribir_version(estado, actor, "ajustar_precios", move |tx, momento| {
        let vigentes: PreciosVigentes = articulos::de_proveedor(tx, proveedor_id)?
            .into_iter()
            .map(|a| (a.id, (a.referencia, a.precio_pu)))
            .collect();
        let nuevos = calcular_ajuste(&vigentes, &articulo_ids, porcentaje)?;
        let datos = DatosDeVersion {
            proveedor_id,
            nombre: format!(
                "Ajuste {} ({} artículos)",
                porcentaje.formatear(true),
                nuevos.len()
            ),
            fecha_tarifa: None,
            descuento_compra_bp: None,
            formato_origen: "ajuste",
            archivo_origen: None,
            importacion_id: None,
        };
        let tarifa = crear_version(tx, &datos, &nuevos, momento)?;
        Ok((nuevos.len(), tarifa))
    })
    .await
}

type PreciosVigentes = BTreeMap<i64, (String, Option<i64>)>;

fn calcular_ajuste(
    vigentes: &PreciosVigentes,
    articulo_ids: &[i64],
    porcentaje: Porcentaje,
) -> Result<BTreeMap<i64, i64>, ErrorApp> {
    let mut nuevos = BTreeMap::new();
    for id in articulo_ids {
        let (referencia, actual) = vigentes.get(id).ok_or(ErrorApp::NoEncontrado)?;
        let actual = actual.ok_or_else(|| {
            ErrorApp::Validacion(format!(
                "{referencia} no tiene precio: escríbelo antes de ajustarlo."
            ))
        })?;
        let nuevo = PrecioUnitario::desde_diezmilesimas(actual)
            .aplicar_porcentaje(porcentaje)
            .filter(|p| p.es_positivo())
            .ok_or_else(|| {
                ErrorApp::Validacion(format!("{referencia}: el precio resultante no es válido."))
            })?;
        nuevos.insert(*id, nuevo.diezmilesimas());
    }
    Ok(nuevos)
}

/// Guarda los precios escritos en la tabla; solo crea versión si alguno cambia.
pub async fn guardar_precios(
    estado: &EstadoApp,
    actor: &Actor,
    proveedor_id: i64,
    escritos: Vec<(i64, String)>,
) -> Result<usize, ErrorApp> {
    escribir_version(estado, actor, "editar_precios", move |tx, momento| {
        let vigentes: PreciosVigentes = articulos::de_proveedor(tx, proveedor_id)?
            .into_iter()
            .map(|a| (a.id, (a.referencia, a.precio_pu)))
            .collect();
        let nuevos = leer_precios_escritos(&vigentes, &escritos)?;
        if nuevos.is_empty() {
            return Ok((0, 0));
        }
        let datos = DatosDeVersion {
            proveedor_id,
            nombre: format!("Edición manual ({} precios)", nuevos.len()),
            fecha_tarifa: None,
            descuento_compra_bp: None,
            formato_origen: "edicion",
            archivo_origen: None,
            importacion_id: None,
        };
        let tarifa = crear_version(tx, &datos, &nuevos, momento)?;
        Ok((nuevos.len(), tarifa))
    })
    .await
}

/// Todos los errores a la vez; un campo vacío no cambia nada (un precio no se borra).
fn leer_precios_escritos(
    vigentes: &PreciosVigentes,
    escritos: &[(i64, String)],
) -> Result<BTreeMap<i64, i64>, ErrorApp> {
    let mut nuevos = BTreeMap::new();
    let mut errores = Vec::new();
    for (id, texto) in escritos.iter().filter(|(_, t)| !t.trim().is_empty()) {
        let Some((referencia, actual)) = vigentes.get(id) else {
            continue;
        };
        match PrecioUnitario::desde_texto_es(texto) {
            Ok(p) if p.es_positivo() && Some(p.diezmilesimas()) != *actual => {
                nuevos.insert(*id, p.diezmilesimas());
            }
            Ok(p) if p.es_positivo() => {}
            Ok(_) => errores.push(format!("{referencia}: el precio debe ser mayor que cero")),
            Err(e) => errores.push(format!("{referencia}: {e}")),
        }
    }
    if errores.is_empty() {
        Ok(nuevos)
    } else {
        Err(ErrorApp::Validacion(format!(
            "No se ha guardado nada. {}",
            errores.join(" · ")
        )))
    }
}
