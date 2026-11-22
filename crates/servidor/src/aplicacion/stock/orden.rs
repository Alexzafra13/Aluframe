//! Qué artículo del stock es cada perfil y accesorio de un trabajo, y qué retales de su color hay
//! para aprovechar. El enlace va por la tabla `referencias` de la versión de la serie con que se
//! aceptó cada ventana (la misma que da el precio).

use std::collections::{BTreeMap, BTreeSet};

use dominio::corte::RetalDisponible;
use dominio::series::{Dato, Serie};
use dominio::taller::RetalesPorPerfil;
use dominio::unidades::Longitud;
use rusqlite::Connection;

use crate::aplicacion::series::reconstruir;
use crate::aplicacion::trabajos::consultar::{FichaDeTrabajo, VentanaDeTrabajo};
use crate::bd::consultas::articulos_con_precio;
use crate::bd::consultas::{series, stock};
use crate::errores::ErrorApp;

/// Artículo y color del stock (sin color: `None`).
pub type ArticuloEnStock = (i64, Option<i64>);

#[derive(Debug, Clone, Default)]
pub struct EnlaceDeStock {
    /// Por (perfil, color) del plan de corte.
    pub perfiles: BTreeMap<(String, String), ArticuloEnStock>,
    /// Por (ventana, código del accesorio).
    pub accesorios: BTreeMap<(i64, String), ArticuloEnStock>,
    pub retales: RetalesPorPerfil,
    /// Largo de cada retal que se ofrece al plan, por su identificador.
    pub largo_de_retal: BTreeMap<u32, i64>,
    /// Códigos sin artículo en el catálogo: no se descuentan y se avisa.
    pub sin_articulo: BTreeSet<String>,
}

struct Buscador<'a> {
    con: &'a Connection,
    series: BTreeMap<i64, Serie>,
}

impl Buscador<'_> {
    fn serie(&mut self, version: i64) -> Result<&Serie, ErrorApp> {
        if !self.series.contains_key(&version) {
            let guardada =
                series::buscar_version(self.con, version)?.ok_or(ErrorApp::NoEncontrado)?;
            self.series.insert(version, reconstruir(guardada)?.serie);
        }
        self.series.get(&version).ok_or(ErrorApp::NoEncontrado)
    }

    /// El artículo de un código de la serie y si se guarda por colores.
    fn articulo(&mut self, version: i64, codigo: &str) -> Result<Option<(i64, bool)>, ErrorApp> {
        let referencia = match self
            .serie(version)?
            .referencias
            .get(codigo)
            .map(|r| &r.referencia)
        {
            Some(Dato::Valor(texto)) => texto.clone(),
            _ => return Ok(None),
        };
        let candidatos = articulos_con_precio::por_referencia(self.con, &referencia)?;
        Ok(match candidatos.as_slice() {
            [unico] => Some((unico.id, unico.con_acabado)),
            _ => None,
        })
    }
}

pub fn enlazar_stock(con: &Connection, ficha: &FichaDeTrabajo) -> Result<EnlaceDeStock, ErrorApp> {
    let mut buscador = Buscador {
        con,
        series: BTreeMap::new(),
    };
    let mut enlace = EnlaceDeStock::default();
    for ventana in &ficha.ventanas {
        enlazar_perfiles(&mut buscador, &mut enlace, ventana)?;
        enlazar_accesorios(&mut buscador, &mut enlace, ventana)?;
    }
    cargar_retales(con, &mut enlace)?;
    Ok(enlace)
}

fn enlazar_perfiles(
    buscador: &mut Buscador<'_>,
    enlace: &mut EnlaceDeStock,
    ventana: &VentanaDeTrabajo,
) -> Result<(), ErrorApp> {
    let c = &ventana.congelado;
    let perfiles = c.resumen.piezas.iter().map(|p| p.perfil.clone());
    let repetidos = ventana.repetidas.iter().map(|r| r.perfil.clone());
    for perfil in perfiles.chain(repetidos) {
        let clave = (perfil.clone(), c.acabado.clone());
        if enlace.perfiles.contains_key(&clave) {
            continue;
        }
        match buscador.articulo(c.serie_version_id, &perfil)? {
            Some((id, con_color)) => {
                enlace
                    .perfiles
                    .insert(clave, (id, c.acabado_id.filter(|_| con_color)));
            }
            None => {
                enlace.sin_articulo.insert(perfil);
            }
        }
    }
    Ok(())
}

fn enlazar_accesorios(
    buscador: &mut Buscador<'_>,
    enlace: &mut EnlaceDeStock,
    ventana: &VentanaDeTrabajo,
) -> Result<(), ErrorApp> {
    let c = &ventana.congelado;
    for (codigo, _) in &c.resumen.accesorios {
        match buscador.articulo(c.serie_version_id, codigo)? {
            Some((id, con_color)) => {
                let color = c.acabado_id.filter(|_| con_color);
                enlace
                    .accesorios
                    .insert((ventana.fila.orden, codigo.clone()), (id, color));
            }
            None => {
                enlace.sin_articulo.insert(codigo.clone());
            }
        }
    }
    Ok(())
}

/// Cada retal del stock (varios iguales cuentan uno a uno) con un identificador para el plan.
fn cargar_retales(con: &Connection, enlace: &mut EnlaceDeStock) -> Result<(), ErrorApp> {
    let mut siguiente: u32 = 1;
    for (clave, (articulo, color)) in &enlace.perfiles {
        let mut disponibles = Vec::new();
        for (largo, cantidad_mil) in stock::retales(con, *articulo, *color)? {
            for _ in 0..(cantidad_mil / 1_000) {
                disponibles.push(RetalDisponible {
                    id: siguiente,
                    longitud: Longitud::desde_decimas_de_mm(largo),
                });
                enlace.largo_de_retal.insert(siguiente, largo);
                siguiente += 1;
            }
        }
        if !disponibles.is_empty() {
            enlace.retales.insert(clave.clone(), disponibles);
        }
    }
    Ok(())
}
