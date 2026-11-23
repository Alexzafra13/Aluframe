use std::collections::BTreeSet;

use super::{BarraCortada, ErrorCorte, ParametrosDeCorte, PiezaACortar, PlanDeCorte};
use crate::unidades::{Decimal, Longitud, Unidad};

/// Un trozo de barra que ya está en el stock y se puede aprovechar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetalDisponible {
    pub id: u32,
    pub longitud: Longitud,
}

/// Best Fit Decreasing: de mayor a menor, cada pieza va a la barra donde sobra menos.
pub fn optimizar_cortes(
    piezas: &[PiezaACortar],
    parametros: &ParametrosDeCorte,
) -> Result<PlanDeCorte, ErrorCorte> {
    optimizar_con_retales(piezas, &[], parametros)
}

/// Igual, pero los retales del stock del mismo perfil y color están abiertos desde el principio:
/// una pieza solo abre una barra nueva si no cabe en ninguno (SPEC, H5). Los retales que no se
/// usan no aparecen en el plan.
pub fn optimizar_con_retales(
    piezas: &[PiezaACortar],
    retales: &[RetalDisponible],
    parametros: &ParametrosDeCorte,
) -> Result<PlanDeCorte, ErrorCorte> {
    let util = parametros.longitud_util()?;
    comprobar_piezas(piezas, parametros, util)?;
    let mut ordenadas = piezas.to_vec();
    ordenadas.sort_by(|a, b| b.longitud.cmp(&a.longitud).then(a.id.cmp(&b.id)));
    let mut barras = barras_de_retales(retales, parametros);
    for pieza in ordenadas {
        colocar(&mut barras, pieza, (parametros.consumo(&pieza)?, util))?;
    }
    barras.retain(|b| !b.piezas.is_empty());
    Ok(PlanDeCorte {
        barras,
        retal_minimo: parametros.retal_minimo,
    })
}

/// Los retales del stock como barras vacías, ya sin el recorte de los extremos.
pub(super) fn barras_de_retales(
    retales: &[RetalDisponible],
    parametros: &ParametrosDeCorte,
) -> Vec<BarraCortada> {
    retales
        .iter()
        .filter_map(|r| {
            let util_retal = r.longitud.restar(parametros.recorte_extremos)?;
            (util_retal.decimas_de_mm() > 0).then_some(BarraCortada {
                piezas: Vec::new(),
                sobrante: util_retal,
                retal: Some(r.id),
            })
        })
        .collect()
}

fn colocar(
    barras: &mut Vec<BarraCortada>,
    pieza: PiezaACortar,
    (consumo, util): (Longitud, Longitud),
) -> Result<(), ErrorCorte> {
    let elegida = barras
        .iter()
        .enumerate()
        .filter(|(_, barra)| barra.sobrante >= consumo)
        .min_by_key(|(indice, barra)| (barra.sobrante, *indice))
        .map(|(indice, _)| indice);
    let barra = match elegida.and_then(|indice| barras.get_mut(indice)) {
        Some(barra) => barra,
        None => {
            barras.push(BarraCortada {
                piezas: Vec::new(),
                sobrante: util,
                retal: None,
            });
            barras.last_mut().ok_or(ErrorCorte::Desbordamiento)?
        }
    };
    barra.sobrante = barra
        .sobrante
        .restar(consumo)
        .ok_or(ErrorCorte::Desbordamiento)?;
    barra.piezas.push(pieza);
    Ok(())
}

fn comprobar_piezas(
    piezas: &[PiezaACortar],
    parametros: &ParametrosDeCorte,
    util: Longitud,
) -> Result<(), ErrorCorte> {
    // Por número de pieza: con varios fallos, el aviso señala siempre la misma, venga como venga.
    let mut por_numero: Vec<&PiezaACortar> = piezas.iter().collect();
    por_numero.sort_by_key(|pieza| pieza.id);
    let mut vistas = BTreeSet::new();
    for pieza in por_numero {
        if !vistas.insert(pieza.id) {
            return Err(ErrorCorte::PiezaRepetida(pieza.id));
        }
        if pieza.longitud.decimas_de_mm() <= 0 {
            return Err(ErrorCorte::PiezaSinLongitud(pieza.id));
        }
        if parametros.consumo(pieza)? > util {
            let longitud = pieza
                .longitud
                .a_decimal(Unidad::Cm)
                .map_or_else(|_| String::from("?"), |d: Decimal| d.to_string());
            return Err(ErrorCorte::PiezaDemasiadoLarga {
                id: pieza.id,
                longitud,
            });
        }
    }
    Ok(())
}

/// Mínimo teórico: suma de consumos entre longitud útil, redondeado hacia arriba.
pub fn calcular_minimo_de_barras(
    piezas: &[PiezaACortar],
    parametros: &ParametrosDeCorte,
) -> Result<usize, ErrorCorte> {
    let util = i128::from(parametros.longitud_util()?.decimas_de_mm());
    let mut total: i128 = 0;
    for pieza in piezas {
        total += i128::from(parametros.consumo(pieza)?.decimas_de_mm());
    }
    usize::try_from((total + util - 1) / util).map_err(|_| ErrorCorte::Desbordamiento)
}
