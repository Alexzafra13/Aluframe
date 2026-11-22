mod conexion;
pub mod consultas;
pub mod copia;
mod esquema;

pub use conexion::BaseDeDatos;
pub use esquema::{
    ErrorEsquema, Migracion, aplicar_migraciones, migraciones_pendientes, preparar_esquema,
    version_de_la_base, version_del_programa,
};
