//! Fichas del catálogo escritas a mano: proveedores, acabados (colores) y artículos.

mod acabado;
mod acabados_incluidos;
mod articulo;
mod proveedor;

#[cfg(test)]
mod pruebas;

pub use acabado::{DatosDeAcabado, EntradaDeAcabado, validar_acabado};
pub use acabados_incluidos::{AcabadoIncluido, leer_acabados_incluidos};
pub use articulo::{DatosDeArticulo, EntradaDeArticulo, validar_articulo};
pub use proveedor::{DatosDeProveedor, EntradaDeProveedor, validar_proveedor};

use crate::clientes::ErrorDeCampo;
use crate::texto::limpiar_opcional;
use crate::unidades::Porcentaje;

fn error(campo: &'static str, mensaje: impl Into<String>) -> ErrorDeCampo {
    ErrorDeCampo {
        campo,
        mensaje: mensaje.into(),
    }
}

/// Texto opcional con un largo máximo.
fn texto(
    campo: &'static str,
    valor: &str,
    largo: usize,
    errores: &mut Vec<ErrorDeCampo>,
) -> Option<String> {
    let limpio = limpiar_opcional(valor);
    if limpio.as_ref().is_some_and(|v| v.chars().count() > largo) {
        errores.push(error(
            campo,
            format!("no puede tener más de {largo} caracteres"),
        ));
    }
    limpio
}

/// Texto obligatorio: vacío es un error con el mensaje dado.
fn obligatorio(
    campo: &'static str,
    valor: &str,
    (largo, falta): (usize, &str),
    errores: &mut Vec<ErrorDeCampo>,
) -> String {
    let limpio = texto(campo, valor, largo, errores);
    if limpio.is_none() {
        errores.push(error(campo, falta));
    }
    limpio.unwrap_or_default()
}

/// Porcentaje entre 0 y el máximo dado; vacío es 0.
fn porcentaje(
    campo: &'static str,
    valor: &str,
    maximo: i64,
    errores: &mut Vec<ErrorDeCampo>,
) -> Porcentaje {
    if valor.trim().is_empty() {
        return Porcentaje::CERO;
    }
    match Porcentaje::desde_texto_es(valor) {
        Ok(p) if (0..=maximo).contains(&p.puntos_basicos()) => p,
        Ok(_) => {
            let tope = Porcentaje::desde_puntos_basicos(maximo).formatear(false);
            errores.push(error(campo, format!("debe estar entre 0 y {tope}")));
            Porcentaje::CERO
        }
        Err(e) => {
            errores.push(error(campo, e.to_string()));
            Porcentaje::CERO
        }
    }
}
