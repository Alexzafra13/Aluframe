mod conexion;
pub mod consultas;
mod migraciones;

pub use conexion::BaseDeDatos;
pub use migraciones::{ErrorMigracion, ULTIMA_VERSION, migrar};
