use std::collections::BTreeMap;

use super::importe::{aumentar, parte, sumar};
use crate::unidades::{Dinero, Porcentaje};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineaParaTotales {
    pub importe: Dinero,
    pub tipo_iva: Porcentaje,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseDeIva {
    pub tipo: Porcentaje,
    pub base: Dinero,
    pub cuota: Dinero,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Totales {
    pub suma_de_lineas: Dinero,
    pub descuento: Dinero,
    pub bases: Vec<BaseDeIva>,
    pub base: Dinero,
    pub iva: Dinero,
    pub total: Dinero,
}

/// El único sitio donde se calculan IVA y totales. El descuento del documento se reparte por
/// tipo de IVA y la cuota se redondea una vez por tipo, sobre la suma de bases (BACKEND 5.4;
/// la regla de redondeo se confirma con el gestor).
pub fn calcular_totales(lineas: &[LineaParaTotales], descuento: Porcentaje) -> Option<Totales> {
    let mut por_tipo: BTreeMap<Porcentaje, Dinero> = BTreeMap::new();
    for linea in lineas {
        let acumulado = por_tipo.entry(linea.tipo_iva).or_insert(Dinero::CERO);
        *acumulado = acumulado.sumar(linea.importe)?;
    }
    let menos = Porcentaje::desde_puntos_basicos(-descuento.puntos_basicos());
    let mut bases = Vec::with_capacity(por_tipo.len());
    for (tipo, suma) in por_tipo {
        let base = aumentar(suma, menos)?;
        bases.push(BaseDeIva {
            tipo,
            base,
            cuota: parte(base, tipo)?,
        });
    }
    let suma_de_lineas = sumar(lineas.iter().map(|l| l.importe))?;
    let base = sumar(bases.iter().map(|b| b.base))?;
    let iva = sumar(bases.iter().map(|b| b.cuota))?;
    Some(Totales {
        descuento: suma_de_lineas.restar(base)?,
        suma_de_lineas,
        bases,
        base,
        iva,
        total: base.sumar(iva)?,
    })
}
