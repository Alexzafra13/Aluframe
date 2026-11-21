//! Clientes: ningún dato es obligatorio (SPEC, apartado 4), pero lo que se escribe se valida.

mod datos;
mod direccion;
mod importacion;

#[cfg(test)]
mod pruebas;

pub use datos::{
    DatosDeCliente, EntradaDeCliente, ErrorDeCampo, TipoCliente, nombre_visible, validar_cliente,
};
pub use direccion::{DatosDeDireccion, EntradaDeDireccion, validar_direccion};
pub use importacion::{COLUMNAS_DE_CLIENTES, FilaDeCliente, apartar_existentes, leer_clientes};
