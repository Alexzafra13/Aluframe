//! Lo que descuenta una orden al terminarla: barras nuevas y retales usados del plan de corte,
//! accesorios de cada ventana y los retales que sobran, que el propietario revisa antes de que
//! entren en el stock (decisión del propietario, 2026-11-22).

use dominio::stock::{ClaveDeStock, leer_cantidad_mil};
use dominio::taller::ClaveDePerfil;
use dominio::texto::nombre_legible;
use dominio::unidades::Longitud;

use super::orden::EnlaceDeStock;
use crate::aplicacion::trabajos::orden::OrdenDeTaller;

pub struct Salida {
    pub texto: String,
    pub clave: ClaveDeStock,
    pub cantidad_mil: i64,
}

pub struct RetalPropuesto {
    /// Índice del grupo (perfil y color) en `EnlaceDeStock::perfiles`.
    pub grupo: usize,
    pub texto: String,
    pub largo_dmm: i64,
}

#[derive(Default)]
pub struct Prevision {
    pub salidas: Vec<Salida>,
    pub retales: Vec<RetalPropuesto>,
    pub avisos: Vec<String>,
}

pub fn prever(orden: &OrdenDeTaller) -> Prevision {
    let mut p = Prevision::default();
    salidas_de_perfiles(orden, &mut p);
    salidas_de_accesorios(orden, &mut p);
    for codigo in &orden.enlace.sin_articulo {
        p.avisos.push(format!(
            "{}: no hay un artículo con su referencia en el catálogo; no se descuenta.",
            nombre_legible(codigo)
        ));
    }
    p
}

fn indice_de_grupo(enlace: &EnlaceDeStock, clave: &ClaveDePerfil) -> Option<usize> {
    enlace.perfiles.keys().position(|k| k == clave)
}

fn salidas_de_perfiles(orden: &OrdenDeTaller, p: &mut Prevision) {
    let enlace = &orden.enlace;
    for grupo in &orden.plan {
        let clave = grupo.clave();
        let nombre = clave.nombre();
        let (Some(&(articulo, color)), Ok(plan)) = (enlace.perfiles.get(&clave), &grupo.plan)
        else {
            if let Err(error) = &grupo.plan {
                p.avisos.push(format!("{nombre}: {error}"));
            }
            continue;
        };
        let nuevas = i64::try_from(plan.barras_nuevas()).unwrap_or_default();
        if nuevas > 0 {
            p.salidas.push(Salida {
                texto: format!("{nombre}: {nuevas} barra(s) entera(s)"),
                clave: (articulo, color, None),
                cantidad_mil: -nuevas * 1_000,
            });
        }
        for barra in &plan.barras {
            if let Some(largo) = barra.retal.and_then(|id| enlace.largo_de_retal.get(&id)) {
                p.salidas.push(Salida {
                    texto: format!(
                        "{nombre}: retal de {} mm",
                        Longitud::desde_decimas_de_mm(*largo).formatear_mm()
                    ),
                    clave: (articulo, color, Some(*largo)),
                    cantidad_mil: -1_000,
                });
            }
            let sobra = barra.sobrante.decimas_de_mm();
            if sobra > 0 && barra.sobrante >= plan.retal_minimo {
                p.retales.push(RetalPropuesto {
                    grupo: indice_de_grupo(enlace, &clave).unwrap_or_default(),
                    texto: nombre.clone(),
                    largo_dmm: sobra,
                });
            }
        }
    }
}

fn salidas_de_accesorios(orden: &OrdenDeTaller, p: &mut Prevision) {
    for ventana in &orden.ficha.ventanas {
        for (codigo, cantidad) in &ventana.congelado.resumen.accesorios {
            let clave = (ventana.fila.orden, codigo.clone());
            let (Some(&(articulo, color)), Ok(mil)) = (
                orden.enlace.accesorios.get(&clave),
                leer_cantidad_mil(cantidad),
            ) else {
                continue;
            };
            p.salidas.push(Salida {
                texto: format!(
                    "{} ×{cantidad} (ventana {})",
                    nombre_legible(codigo),
                    ventana.fila.orden
                ),
                clave: (articulo, color, None),
                cantidad_mil: -mil,
            });
        }
    }
}
