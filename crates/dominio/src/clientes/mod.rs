//! Clientes: el alta a mano exige nombre y teléfono; la importación no exige nada,
//! pero lo que se escribe se valida.

mod datos;
mod direccion;
mod importacion;
mod telefono;

#[cfg(test)]
mod pruebas;

pub use datos::{
    DatosDeCliente, EntradaDeCliente, ErrorDeCampo, TipoCliente, nombre_visible,
    validar_alta_de_cliente, validar_cliente,
};
pub use direccion::{DatosDeDireccion, EntradaDeDireccion, validar_direccion};
pub use importacion::{COLUMNAS_DE_CLIENTES, FilaDeCliente, apartar_existentes, leer_clientes};
pub use telefono::{es_espana, normalizar_telefono};
