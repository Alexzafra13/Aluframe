//! Mejor aprovechamiento (ADR-0026): parte del reparto de Best Fit Decreasing y busca otro con
//! menos barras nuevas o, con las mismas, con mejores sobrantes. Solo acepta uno estrictamente
//! mejor, así que nunca sale peor. La búsqueda tiene un número fijo de pasos, no un tiempo:
//! el resultado es siempre el mismo en cualquier equipo.

use super::mejor_ajuste::barras_de_retales;
use super::valoracion::ValoracionDeSobrantes;
use super::{BarraCortada, ParametrosDeCorte, PiezaACortar, PlanDeCorte, RetalDisponible};
use crate::unidades::Longitud;

/// Pasos por perfil y color: en milisegundos para un trabajo normal del taller.
pub const PASOS_MAXIMOS: u32 = 20_000;

#[derive(Debug, Clone)]
struct Barra {
    sobrante: i64,
    retal: Option<u32>,
    piezas: Vec<PiezaACortar>,
}

struct Busqueda<'a> {
    /// De mayor a menor (a igual largo, por número), con lo que gasta cada una.
    piezas: Vec<(PiezaACortar, i64)>,
    /// Lo que gastan las piezas desde cada posición hasta el final.
    resto: Vec<i64>,
    util: i64,
    barras: Vec<Barra>,
    nuevas: usize,
    libre: i64,
    donde: Vec<usize>,
    pasos: u32,
    mejor: (usize, i128),
    mejor_reparto: Option<Vec<Barra>>,
    valorar: &'a dyn Fn(&[Longitud]) -> i128,
}

/// El plan que mejora a `rapido` (el de Best Fit Decreasing), o él mismo si no lo hay.
pub fn mejorar(
    rapido: PlanDeCorte,
    (piezas, retales): (&[PiezaACortar], &[RetalDisponible]),
    parametros: &ParametrosDeCorte,
    valoracion: &ValoracionDeSobrantes,
) -> PlanDeCorte {
    let (Ok(util), Ok(consumos)) = (
        parametros.longitud_util(),
        piezas
            .iter()
            .map(|p| parametros.consumo(p).map(|c| (*p, c.decimas_de_mm())))
            .collect::<Result<Vec<_>, _>>(),
    ) else {
        return rapido;
    };
    let valorar = |sobrantes: &[Longitud]| valoracion.valorar(sobrantes, parametros.grosor_sierra);
    let iniciales = barras_de_retales(retales, parametros);
    let valor_rapido = valorar(&sobrantes_del_plan(&rapido, &iniciales));
    let mut busqueda = Busqueda::nueva(consumos, util.decimas_de_mm(), &iniciales, &valorar);
    busqueda.mejor = (rapido.barras_nuevas(), valor_rapido);
    busqueda.buscar(0);
    match busqueda.mejor_reparto {
        Some(reparto) => PlanDeCorte {
            barras: reparto
                .into_iter()
                .filter(|b| !b.piezas.is_empty())
                .map(|b| BarraCortada {
                    piezas: b.piezas,
                    sobrante: Longitud::desde_decimas_de_mm(b.sobrante),
                    retal: b.retal,
                })
                .collect(),
            retal_minimo: rapido.retal_minimo,
        },
        None => rapido,
    }
}

/// Lo que queda tras el plan: sus sobrantes y los retales del stock que no usa, enteros.
fn sobrantes_del_plan(plan: &PlanDeCorte, retales: &[BarraCortada]) -> Vec<Longitud> {
    let usados: Vec<u32> = plan.barras.iter().filter_map(|b| b.retal).collect();
    let sin_usar = retales
        .iter()
        .filter(|r| r.retal.is_some_and(|id| !usados.contains(&id)))
        .map(|r| r.sobrante);
    plan.barras
        .iter()
        .map(|b| b.sobrante)
        .chain(sin_usar)
        .collect()
}

impl<'a> Busqueda<'a> {
    fn nueva(
        mut piezas: Vec<(PiezaACortar, i64)>,
        util: i64,
        retales: &[BarraCortada],
        valorar: &'a dyn Fn(&[Longitud]) -> i128,
    ) -> Busqueda<'a> {
        piezas.sort_by(|(a, _), (b, _)| b.longitud.cmp(&a.longitud).then(a.id.cmp(&b.id)));
        let mut resto = vec![0; piezas.len() + 1];
        for indice in (0..piezas.len()).rev() {
            resto[indice] = resto[indice + 1] + piezas[indice].1;
        }
        let barras: Vec<Barra> = retales
            .iter()
            .map(|r| Barra {
                sobrante: r.sobrante.decimas_de_mm(),
                retal: r.retal,
                piezas: Vec::new(),
            })
            .collect();
        Busqueda {
            libre: barras.iter().map(|b| b.sobrante).sum(),
            donde: vec![0; piezas.len()],
            piezas,
            resto,
            util,
            barras,
            nuevas: 0,
            pasos: 0,
            mejor: (usize::MAX, i128::MIN),
            mejor_reparto: None,
            valorar,
        }
    }

    fn buscar(&mut self, indice: usize) {
        if self.pasos >= PASOS_MAXIMOS {
            return;
        }
        self.pasos += 1;
        let Some(&(pieza, consumo)) = self.piezas.get(indice) else {
            self.evaluar();
            return;
        };
        let falta = self.resto[indice] - self.libre;
        let minimas = usize::try_from((falta.max(0) + self.util - 1) / self.util).unwrap_or(0);
        if self.nuevas + minimas > self.mejor.0 {
            return;
        }
        for barra in self.candidatas(indice, pieza, consumo) {
            self.probar(indice, barra);
        }
        if self.nuevas < self.mejor.0 {
            self.barras.push(Barra {
                sobrante: self.util,
                retal: None,
                piezas: Vec::new(),
            });
            self.nuevas += 1;
            self.libre += self.util;
            self.probar(indice, self.barras.len() - 1);
            self.barras.pop();
            self.nuevas -= 1;
            self.libre -= self.util;
        }
    }

    /// Las barras abiertas donde cabe, de la que menos sobra a la que más. Dos con el mismo
    /// sobrante dan lo mismo: se prueba una. Una pieza igual que la anterior no va a una barra
    /// anterior a la suya (sería el mismo reparto en otro orden).
    fn candidatas(&self, indice: usize, pieza: PiezaACortar, consumo: i64) -> Vec<usize> {
        let desde = match indice.checked_sub(1).and_then(|i| self.piezas.get(i)) {
            Some((anterior, _)) if anterior.longitud == pieza.longitud => self.donde[indice - 1],
            _ => 0,
        };
        let mut candidatas: Vec<usize> = (desde..self.barras.len())
            .filter(|b| self.barras[*b].sobrante >= consumo)
            .collect();
        candidatas.sort_by_key(|b| (self.barras[*b].sobrante, *b));
        candidatas.dedup_by_key(|b| self.barras[*b].sobrante);
        candidatas
    }

    fn probar(&mut self, indice: usize, barra: usize) {
        let (pieza, consumo) = self.piezas[indice];
        self.barras[barra].sobrante -= consumo;
        self.barras[barra].piezas.push(pieza);
        self.libre -= consumo;
        self.donde[indice] = barra;
        self.buscar(indice + 1);
        self.barras[barra].sobrante += consumo;
        self.barras[barra].piezas.pop();
        self.libre += consumo;
    }

    fn evaluar(&mut self) {
        let sobrantes: Vec<Longitud> = self
            .barras
            .iter()
            .map(|b| Longitud::desde_decimas_de_mm(b.sobrante))
            .collect();
        let valor = (self.valorar)(&sobrantes);
        if self.nuevas < self.mejor.0 || (self.nuevas == self.mejor.0 && valor > self.mejor.1) {
            self.mejor = (self.nuevas, valor);
            self.mejor_reparto = Some(self.barras.clone());
        }
    }
}
