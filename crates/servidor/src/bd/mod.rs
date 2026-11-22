mod conexion;
pub mod consultas;
pub mod copia;
mod esquema;

pub use conexion::BaseDeDatos;
pub use esquema::{ErrorEsquema, huella_del_esquema, leer_huella_guardada, preparar_esquema};
