//! Enlaza los códigos de una serie (MARCO_LATERAL…) con los artículos de la tarifa vigente
//! a través de su tabla `referencias` (FORMATO_SERIES, apartado 5).

use std::collections::{BTreeMap, BTreeSet};

use dominio::precio::ArticuloConPrecio;
use dominio::series::{Dato, ResultadoVentana, Serie};
use dominio::tarifa::UnidadDeVenta;
use dominio::unidades::{Longitud, Porcentaje, PrecioUnitario};
use rusqlite::Connection;

use crate::bd::consultas::articulos_con_precio::{self, ArticuloParaPrecio};
use crate::errores::ErrorApp;

pub struct Enlace {
    pub catalogo: BTreeMap<String, ArticuloConPrecio>,
    pub tarifas: BTreeSet<i64>,
    pub avisos: Vec<String>,
}

/// Códigos que usa el cálculo: perfiles, vidrio y accesorios.
fn codigos_usados(resultado: &ResultadoVentana) -> BTreeSet<String> {
    let mut codigos: BTreeSet<String> = resultado.piezas.iter().map(|p| p.perfil.clone()).collect();
    codigos.extend(resultado.accesorios.iter().map(|a| a.articulo.clone()));
    codigos.extend(resultado.vidrio.iter().flatten().map(|v| v.tipo.clone()));
    codigos
}

pub fn enlazar(
    con: &Connection,
    serie: &Serie,
    resultado: &ResultadoVentana,
    vidrio_elegido: Option<i64>,
) -> Result<Enlace, ErrorApp> {
    let mut enlace = Enlace {
        catalogo: BTreeMap::new(),
        tarifas: BTreeSet::new(),
        avisos: Vec::new(),
    };
    let codigo_vidrio: BTreeSet<&str> = resultado
        .vidrio
        .iter()
        .flatten()
        .map(|v| v.tipo.as_str())
        .collect();
    for codigo in codigos_usados(resultado) {
        let encontrado = match vidrio_elegido.filter(|_| codigo_vidrio.contains(codigo.as_str())) {
            Some(id) => articulos_con_precio::por_id(con, id)?,
            None => buscar_por_serie(con, serie, &codigo, &mut enlace.avisos)?,
        };
        let Some(articulo) = encontrado else { continue };
        if let Some(convertido) = convertir(&articulo) {
            enlace.tarifas.extend(articulo.tarifa_id);
            enlace.catalogo.insert(codigo, convertido);
        }
    }
    Ok(enlace)
}

/// Con varios proveedores con la misma referencia, manda el proveedor que declara la serie.
fn buscar_por_serie(
    con: &Connection,
    serie: &Serie,
    codigo: &str,
    avisos: &mut Vec<String>,
) -> Result<Option<ArticuloParaPrecio>, ErrorApp> {
    let Some(referencia) = serie.referencias.get(codigo) else {
        return Ok(None);
    };
    let Dato::Valor(texto) = &referencia.referencia else {
        return Ok(None);
    };
    let mut candidatos = articulos_con_precio::por_referencia(con, texto)?;
    let proveedor = referencia
        .proveedor
        .clone()
        .or_else(|| match &serie.proveedor {
            Dato::Valor(p) => Some(p.clone()),
            _ => None,
        });
    if let Some(p) = proveedor.filter(|_| candidatos.len() > 1) {
        candidatos.retain(|a| a.proveedor.eq_ignore_ascii_case(&p));
    }
    if candidatos.len() > 1 {
        avisos.push(format!("{codigo}: la referencia {texto} está en varios proveedores; indica el proveedor en la serie"));
        return Ok(None);
    }
    Ok(candidatos.pop())
}

/// Sin precio en la tarifa vigente no hay artículo para el cálculo: la línea dirá qué falta.
fn convertir(a: &ArticuloParaPrecio) -> Option<ArticuloConPrecio> {
    Some(ArticuloConPrecio {
        referencia: a.referencia.clone(),
        unidad: UnidadDeVenta::desde_texto(&a.unidad)?,
        precio: PrecioUnitario::desde_diezmilesimas(a.precio_pu?),
        descuento_compra: Porcentaje::desde_puntos_basicos(a.descuento_compra_bp),
        longitud_barra: a.longitud_barra_dmm.map(Longitud::desde_decimas_de_mm),
        unidades_por_caja: a.unidades_por_caja,
        con_acabado: a.con_acabado,
    })
}
