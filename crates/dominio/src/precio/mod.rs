//! Precio de una línea de ventana y totales del documento (BACKEND, apartado 5.4).
//! Orden: coste de materiales → margen → mano de obra → descuento de línea; después, en el
//! documento, descuento general → base por tipo de IVA → cuota → total.

mod importe;
mod materiales;
mod totales;
mod venta;

#[cfg(test)]
mod pruebas;

use crate::tarifa::UnidadDeVenta;
use crate::unidades::{Dinero, Longitud, Porcentaje, PrecioUnitario};

pub use materiales::{CosteDeMateriales, PerfilUsado, calcular_coste_de_materiales};
pub use totales::{BaseDeIva, LineaParaTotales, Totales, calcular_totales};
pub use venta::{PrecioDeLinea, calcular_precio_de_linea};

/// Un artículo de la tarifa vigente, enlazado desde un código de la serie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticuloConPrecio {
    pub referencia: String,
    pub unidad: UnidadDeVenta,
    pub precio: PrecioUnitario,
    pub descuento_compra: Porcentaje,
    pub longitud_barra: Option<Longitud>,
    pub unidades_por_caja: Option<i64>,
    pub con_acabado: bool,
}

/// Cómo se cobran los perfiles (pendiente del propietario: se elige en Ajustes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModoPerfiles {
    BarrasEnteras,
    MetrosConMerma(Porcentaje),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModoMargen {
    SobreCoste,
    SobreVenta,
}

/// Lo que el propietario configura. `None` significa «sin elegir»: la línea no tiene precio final.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReglasDePrecio {
    pub modo_perfiles: Option<ModoPerfiles>,
    pub margen: Option<(ModoMargen, Porcentaje)>,
}

/// Datos propios de la línea.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatosDeLinea {
    pub cantidad: u32,
    pub recargo_acabado: Porcentaje,
    pub mano_de_obra: Dinero,
    pub descuento: Porcentaje,
}
