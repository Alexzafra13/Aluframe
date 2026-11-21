//! Tarifas de proveedor: lectura del CSV, validación y comparación con la vigente (ADR-0012).

mod articulo;
mod comparacion;
mod lectura;

#[cfg(test)]
mod pruebas;

pub use articulo::{TipoArticulo, UnidadDeVenta};
pub use comparacion::{ArticuloVigente, CambioDePrecio, Comparacion, comparar_con_vigente};
pub use lectura::{COLUMNAS_DE_TARIFA, FilaDeTarifa, leer_tarifa};

use crate::unidades::Porcentaje;

/// Por encima de este salto el cambio de precio se avisa (SPEC, apartado 7).
pub const SALTO_DE_PRECIO_AVISADO: Porcentaje = Porcentaje::desde_puntos_basicos(3_000);

/// Un ajuste masivo de más del 100 % casi siempre es un error de tecleo («100» por «10»).
pub fn validar_ajuste(porcentaje: Porcentaje) -> Result<(), &'static str> {
    let puntos = porcentaje.puntos_basicos();
    if puntos == 0 {
        Err("Escribe un porcentaje distinto de 0.")
    } else if puntos <= -10_000 {
        Err("No se puede bajar un 100 % o más: el precio quedaría a cero.")
    } else if puntos > 10_000 {
        Err("Un ajuste de más del 100 % parece un error; si es correcto, hazlo en dos pasos.")
    } else {
        Ok(())
    }
}
