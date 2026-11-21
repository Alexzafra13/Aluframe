use std::collections::{BTreeMap, HashMap};

use dominio::acceso::Accion;
use dominio::catalogo::DatosDeArticulo;
use dominio::informe::Informe;
use dominio::tarifa::{
    ArticuloVigente, Comparacion, FilaDeTarifa, TipoArticulo, UnidadDeVenta, comparar_con_vigente,
    leer_tarifa,
};
use dominio::unidades::PrecioUnitario;
use rusqlite::Connection;

use super::auditar::{Cambio, anotar};
use super::tarifas::{DatosDeVersion, crear_version};
use super::{Actor, autorizar};
use crate::autenticacion::calcular_sha256;
use crate::bd::consultas::articulos::{self, FilaArticulo};
use crate::bd::consultas::importaciones::{self, NuevaImportacion};
use crate::bd::consultas::proveedores;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

/// Lo que se elige en el paso 1 además del archivo.
#[derive(Debug, Clone, Default)]
pub struct CabeceraDeTarifa {
    pub proveedor_id: i64,
    pub nombre: String,
    pub fecha_tarifa: Option<String>,
    pub descuento_compra_bp: Option<i64>,
}

pub struct PrevisualizacionDeTarifa {
    pub proveedor: String,
    pub comparacion: Comparacion,
    pub errores: Vec<String>,
    pub avisos: Vec<String>,
    pub sha256: String,
}

impl PrevisualizacionDeTarifa {
    pub fn se_puede_importar(&self) -> bool {
        self.errores.is_empty()
    }
}

struct Analisis {
    filas: Vec<FilaDeTarifa>,
    comparacion: Comparacion,
    informe: Informe,
    existentes: HashMap<String, FilaArticulo>,
}

fn analizar(con: &Connection, proveedor_id: i64, texto: &str) -> Result<Analisis, ErrorApp> {
    let (filas, mut informe) = leer_tarifa(texto);
    let existentes: HashMap<String, FilaArticulo> = articulos::de_proveedor(con, proveedor_id)?
        .into_iter()
        .map(|a| (a.referencia.clone(), a))
        .collect();
    let vigentes: Vec<ArticuloVigente> = existentes
        .values()
        .filter_map(|a| {
            Some(ArticuloVigente {
                referencia: a.referencia.clone(),
                unidad: UnidadDeVenta::desde_texto(&a.unidad)?,
                precio: a.precio_pu.map(PrecioUnitario::desde_diezmilesimas),
            })
        })
        .collect();
    let comparacion = comparar_con_vigente(&filas, &vigentes, &mut informe);
    if !informe.tiene_errores() && !comparacion.hay_cambios() {
        informe.error(
            "",
            "la tarifa es igual a la vigente: no hay cambios que importar",
        );
    }
    Ok(Analisis {
        filas,
        comparacion,
        informe,
        existentes,
    })
}

pub async fn previsualizar_tarifa(
    estado: &EstadoApp,
    actor: &Actor,
    proveedor_id: i64,
    texto: String,
) -> Result<PrevisualizacionDeTarifa, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let sha256 = calcular_sha256(&texto);
    let (proveedor, analisis) = estado
        .bd
        .leer(move |con| {
            let proveedor = proveedores::buscar(con, proveedor_id)?
                .ok_or_else(|| ErrorApp::Validacion("Elige el proveedor de la tarifa.".into()))?;
            Ok((proveedor.nombre, analizar(con, proveedor_id, &texto)?))
        })
        .await?;
    let mut comparacion = analisis.comparacion;
    comparacion
        .cambios
        .sort_by_key(|c| std::cmp::Reverse(c.variacion.map(|v| v.absoluto())));
    Ok(PrevisualizacionDeTarifa {
        proveedor,
        comparacion,
        errores: analisis
            .informe
            .errores
            .iter()
            .map(ToString::to_string)
            .collect(),
        avisos: analisis
            .informe
            .avisos
            .iter()
            .map(ToString::to_string)
            .collect(),
        sha256,
    })
}

/// Vuelve a analizar dentro de la transacción: se guarda exactamente lo revisado.
pub async fn importar_tarifa(
    estado: &EstadoApp,
    actor: &Actor,
    (cabecera, texto, sha_revisado): (CabeceraDeTarifa, String, String),
) -> Result<(i64, usize), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    if calcular_sha256(&texto) != sha_revisado {
        return Err(ErrorApp::Validacion(
            "El archivo no es el que se revisó. Vuelve a revisarlo antes de confirmar.".into(),
        ));
    }
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| guardar(tx, (&cabecera, &texto, &sha_revisado), (ahora, autor)))
        .await
}

/// La transacción de la importación: artículos, versión de tarifa, importación y auditoría.
fn guardar(
    tx: &Connection,
    (cabecera, texto, sha256): (&CabeceraDeTarifa, &str, &str),
    (ahora, autor): (time::OffsetDateTime, i64),
) -> Result<(i64, usize), ErrorApp> {
    proveedores::buscar(tx, cabecera.proveedor_id)?.ok_or(ErrorApp::NoEncontrado)?;
    let analisis = analizar(tx, cabecera.proveedor_id, texto)?;
    if analisis.informe.tiene_errores() {
        return Err(ErrorApp::Validacion(
            "La tarifa tiene errores; revisa la vista previa.".into(),
        ));
    }
    let cuando = formatear_utc(ahora);
    let nueva = NuevaImportacion {
        tipo: "tarifa",
        sha256,
        filas: i64::try_from(analisis.filas.len()).unwrap_or(i64::MAX),
        usuario_id: autor,
        importada_en: &cuando,
    };
    let importacion = importaciones::insertar(tx, &nueva)?;
    let precios = guardar_articulos(tx, &analisis, (cabecera.proveedor_id, importacion), &cuando)?;
    let datos = DatosDeVersion {
        proveedor_id: cabecera.proveedor_id,
        nombre: cabecera.nombre.clone(),
        fecha_tarifa: cabecera.fecha_tarifa.clone(),
        descuento_compra_bp: cabecera.descuento_compra_bp,
        formato_origen: "csv",
        archivo_origen: Some(texto.to_owned()),
        importacion_id: Some(importacion),
    };
    let tarifa = crear_version(tx, &datos, &precios, (&cuando, autor))?;
    let cambio = Cambio {
        ahora,
        autor,
        accion: "importar_tarifa",
        entidad: "tarifa",
        entidad_id: tarifa,
    };
    anotar(tx, &cambio)?;
    Ok((tarifa, analisis.filas.len()))
}

/// Crea los artículos nuevos, pone al día los existentes y devuelve el precio de cada uno.
fn guardar_articulos(
    tx: &Connection,
    analisis: &Analisis,
    (proveedor_id, importacion): (i64, i64),
    cuando: &str,
) -> Result<BTreeMap<i64, i64>, ErrorApp> {
    let mut precios = BTreeMap::new();
    for fila in &analisis.filas {
        let existente = analisis.existentes.get(&fila.referencia);
        let datos = DatosDeArticulo {
            referencia: fila.referencia.clone(),
            descripcion: fila.descripcion.clone(),
            tipo: fila
                .tipo
                .or_else(|| existente.and_then(|a| TipoArticulo::desde_texto(&a.tipo)))
                .unwrap_or(TipoArticulo::Otro),
            unidad: fila.unidad,
            longitud_barra: fila.longitud_barra,
            unidades_por_caja: existente
                .and_then(|a| a.unidades_por_caja)
                .filter(|_| fila.unidad == UnidadDeVenta::Caja),
            con_acabado: existente.is_some_and(|a| a.con_acabado),
            stock_minimo_mil: existente.map_or(0, |a| a.stock_minimo_mil),
        };
        let id = match existente {
            Some(a) => {
                articulos::actualizar_desde_tarifa(tx, a.id, &datos)?;
                a.id
            }
            None => articulos::insertar(tx, proveedor_id, &datos, (cuando, Some(importacion)))?,
        };
        precios.insert(id, fila.precio.diezmilesimas());
    }
    Ok(precios)
}
