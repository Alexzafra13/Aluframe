//! Stock (H5): cantidades en milésimas, comprobaciones de los movimientos y el resumen de lo que
//! descuenta una orden. El saldo nunca se guarda: es la suma de los movimientos.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::unidades::{Decimal, leer_decimal_es};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErrorDeStock {
    #[error("escribe una cantidad (por ejemplo 3 o 2,5)")]
    CantidadNoValida,
    #[error("una entrada tiene que sumar: escribe una cantidad mayor que cero")]
    EntradaSinCantidad,
    #[error("un ajuste no puede ser cero")]
    AjusteCero,
    #[error("escribe el motivo del ajuste (recuento, rotura…)")]
    AjusteSinMotivo,
    #[error("con este movimiento quedarían {0}: el stock no puede ser negativo")]
    QuedaNegativo(String),
}

/// «2,5» son 2 500 milésimas; con signo para los ajustes.
pub fn leer_cantidad_mil(texto: &str) -> Result<i64, ErrorDeStock> {
    let decimal = leer_decimal_es(texto.trim()).map_err(|_| ErrorDeStock::CantidadNoValida)?;
    decimal
        .multiplicar(Decimal::desde_entero(1_000))
        .ok()
        .and_then(Decimal::a_entero)
        .ok_or(ErrorDeStock::CantidadNoValida)
}

pub fn formatear_cantidad(mil: i64) -> String {
    crate::unidades::formatear_es(i128::from(mil), 3, 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDeMovimiento {
    Entrada,
    Ajuste,
}

/// Comprueba un movimiento hecho a mano frente al saldo actual de ese artículo, color y largo.
pub fn comprobar_movimiento(
    tipo: TipoDeMovimiento,
    (cantidad_mil, motivo): (i64, &str),
    saldo_mil: i64,
) -> Result<(), ErrorDeStock> {
    match tipo {
        TipoDeMovimiento::Entrada if cantidad_mil <= 0 => {
            return Err(ErrorDeStock::EntradaSinCantidad);
        }
        TipoDeMovimiento::Ajuste if cantidad_mil == 0 => return Err(ErrorDeStock::AjusteCero),
        TipoDeMovimiento::Ajuste if motivo.trim().is_empty() => {
            return Err(ErrorDeStock::AjusteSinMotivo);
        }
        _ => {}
    }
    let resultado = saldo_mil + cantidad_mil;
    if resultado < 0 {
        return Err(ErrorDeStock::QuedaNegativo(formatear_cantidad(resultado)));
    }
    Ok(())
}

/// Artículo, color y largo (sin largo, la barra entera o la unidad).
pub type ClaveDeStock = (i64, Option<i64>, Option<i64>);

/// Junta lo que una orden saca y mete por artículo, color y largo; lo que se compensa (usar un
/// retal de 120 y dejar otro de 120) no genera movimiento.
pub fn agrupar_movimientos(movimientos: &[(ClaveDeStock, i64)]) -> Vec<(ClaveDeStock, i64)> {
    let mut suma: BTreeMap<ClaveDeStock, i64> = BTreeMap::new();
    for (clave, cantidad) in movimientos {
        *suma.entry(*clave).or_default() += cantidad;
    }
    suma.into_iter().filter(|(_, c)| *c != 0).collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn lee_cantidades_en_milesimas() {
        assert_eq!(leer_cantidad_mil("3"), Ok(3_000));
        assert_eq!(leer_cantidad_mil(" 2,5 "), Ok(2_500));
        assert_eq!(leer_cantidad_mil("-1"), Ok(-1_000));
        assert_eq!(
            leer_cantidad_mil("tres"),
            Err(ErrorDeStock::CantidadNoValida)
        );
        assert_eq!(formatear_cantidad(2_500), "2,5");
    }

    #[test]
    fn las_entradas_suman_los_ajustes_llevan_motivo_y_nada_queda_negativo() {
        use TipoDeMovimiento::*;
        assert!(comprobar_movimiento(Entrada, (2_000, ""), 0).is_ok());
        assert_eq!(
            comprobar_movimiento(Entrada, (0, ""), 0),
            Err(ErrorDeStock::EntradaSinCantidad)
        );
        assert_eq!(
            comprobar_movimiento(Ajuste, (-1_000, " "), 5_000),
            Err(ErrorDeStock::AjusteSinMotivo)
        );
        assert_eq!(
            comprobar_movimiento(Ajuste, (0, "x"), 5_000),
            Err(ErrorDeStock::AjusteCero)
        );
        assert!(comprobar_movimiento(Ajuste, (-1_000, "Rotura"), 1_000).is_ok());
        assert_eq!(
            comprobar_movimiento(Ajuste, (-2_000, "Rotura"), 1_000),
            Err(ErrorDeStock::QuedaNegativo("-1".into()))
        );
    }

    #[test]
    fn agrupa_y_compensa_los_movimientos_de_una_orden() {
        let movimientos = [
            ((1, None, None), -2_000),
            ((1, None, None), -1_000),
            ((1, None, Some(12_000)), -1_000),
            ((1, None, Some(12_000)), 1_000),
            ((1, None, Some(8_000)), 1_000),
        ];
        assert_eq!(
            agrupar_movimientos(&movimientos),
            vec![((1, None, None), -3_000), ((1, None, Some(8_000)), 1_000)]
        );
    }
}
