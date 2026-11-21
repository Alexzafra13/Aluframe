//! Aritmética de dinero con un único redondeo explícito (mitad alejándose de cero).

use crate::unidades::{Dinero, Porcentaje, SentidoRedondeo};

const CIEN_POR_CIEN: i128 = 10_000;

/// `importe × (1 + porcentaje)`; con un porcentaje negativo, descuenta.
pub fn aumentar(importe: Dinero, porcentaje: Porcentaje) -> Option<Dinero> {
    let factor = CIEN_POR_CIEN + i128::from(porcentaje.puntos_basicos());
    escalar(importe, factor, CIEN_POR_CIEN)
}

/// `importe ÷ (1 − porcentaje)`: precio de venta con un margen sobre la venta.
pub fn dividir_por_complemento(importe: Dinero, porcentaje: Porcentaje) -> Option<Dinero> {
    let divisor = CIEN_POR_CIEN - i128::from(porcentaje.puntos_basicos());
    if divisor <= 0 {
        return None;
    }
    escalar(importe, CIEN_POR_CIEN, divisor)
}

/// `importe × porcentaje`.
pub fn parte(importe: Dinero, porcentaje: Porcentaje) -> Option<Dinero> {
    escalar(
        importe,
        i128::from(porcentaje.puntos_basicos()),
        CIEN_POR_CIEN,
    )
}

pub fn escalar(importe: Dinero, por: i128, entre: i128) -> Option<Dinero> {
    let producto = i128::from(importe.centimos()).checked_mul(por)?;
    let resultado = SentidoRedondeo::Cercano.dividir(producto, entre)?;
    i64::try_from(resultado).ok().map(Dinero::desde_centimos)
}

pub fn sumar(importes: impl IntoIterator<Item = Dinero>) -> Option<Dinero> {
    importes.into_iter().try_fold(Dinero::CERO, Dinero::sumar)
}
