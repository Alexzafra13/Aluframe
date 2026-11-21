use std::collections::BTreeSet;

use super::{BarraCortada, ErrorCorte, ParametrosDeCorte, PiezaACortar, PlanDeCorte};
use crate::unidades::{Decimal, Longitud, Unidad};

/// Best Fit Decreasing: de mayor a menor, cada pieza va a la barra donde sobra menos.
pub fn optimizar_cortes(
    piezas: &[PiezaACortar],
    parametros: &ParametrosDeCorte,
) -> Result<PlanDeCorte, ErrorCorte> {
    let util = parametros.longitud_util()?;
    comprobar_piezas(piezas, parametros, util)?;
    let mut ordenadas = piezas.to_vec();
    ordenadas.sort_by(|a, b| b.longitud.cmp(&a.longitud).then(a.id.cmp(&b.id)));
    let mut barras: Vec<BarraCortada> = Vec::new();
    for pieza in ordenadas {
        let consumo = parametros.consumo(&pieza)?;
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
                });
                barras.last_mut().ok_or(ErrorCorte::Desbordamiento)?
            }
        };
        barra.sobrante = barra
            .sobrante
            .restar(consumo)
            .ok_or(ErrorCorte::Desbordamiento)?;
        barra.piezas.push(pieza);
    }
    Ok(PlanDeCorte {
        barras,
        retal_minimo: parametros.retal_minimo,
    })
}

fn comprobar_piezas(
    piezas: &[PiezaACortar],
    parametros: &ParametrosDeCorte,
    util: Longitud,
) -> Result<(), ErrorCorte> {
    let mut vistas = BTreeSet::new();
    for pieza in piezas {
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
