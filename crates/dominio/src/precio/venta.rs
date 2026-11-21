use super::importe::{aumentar, dividir_por_complemento, escalar, sumar};
use super::materiales::CosteDeMateriales;
use super::{DatosDeLinea, ModoMargen, ReglasDePrecio};
use crate::unidades::{Dinero, Porcentaje};

/// Precio de una línea de ventana. Con algo en `faltan` no hay precio final: los importes son
/// orientativos y el presupuesto no se puede aceptar.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrecioDeLinea {
    pub coste_unitario: Dinero,
    pub venta_unitaria: Dinero,
    pub descuento: Dinero,
    pub precio: Dinero,
    pub faltan: Vec<String>,
}

impl PrecioDeLinea {
    pub fn es_final(&self) -> bool {
        self.faltan.is_empty()
    }
}

/// Coste de materiales → margen → + mano de obra (ya es precio de venta: no lleva margen)
/// → × cantidad → − descuento de línea.
pub fn calcular_precio_de_linea(
    materiales: &CosteDeMateriales,
    reglas: &ReglasDePrecio,
    linea: &DatosDeLinea,
) -> PrecioDeLinea {
    let mut faltan = materiales.faltan.clone();
    let coste = sumar([
        materiales.perfiles,
        materiales.cristal,
        materiales.accesorios,
    ]);
    let venta_materiales = match (coste, reglas.margen) {
        (Some(c), Some((modo, margen))) => aplicar_margen(c, modo, margen),
        (_, None) => {
            faltan.push("elige el margen (Ajustes > Presupuestos)".into());
            coste
        }
        (None, _) => None,
    };
    let venta_unitaria = venta_materiales.and_then(|v| v.sumar(linea.mano_de_obra));
    let bruto = venta_unitaria.and_then(|v| escalar(v, i128::from(linea.cantidad), 1));
    let precio = bruto.and_then(|b| {
        aumentar(
            b,
            Porcentaje::desde_puntos_basicos(-linea.descuento.puntos_basicos()),
        )
    });
    if precio.is_none() {
        faltan.push("el importe es demasiado grande o el margen no es válido".into());
    }
    let (bruto, precio) = (bruto.unwrap_or_default(), precio.unwrap_or_default());
    PrecioDeLinea {
        coste_unitario: coste.unwrap_or_default(),
        venta_unitaria: venta_unitaria.unwrap_or_default(),
        descuento: bruto.restar(precio).unwrap_or_default(),
        precio,
        faltan,
    }
}

fn aplicar_margen(coste: Dinero, modo: ModoMargen, margen: Porcentaje) -> Option<Dinero> {
    match modo {
        ModoMargen::SobreCoste => aumentar(coste, margen),
        ModoMargen::SobreVenta => dividir_por_complemento(coste, margen),
    }
}
