mod bloqueo;
mod credenciales;
mod error;
mod permisos;
mod rol;

pub use bloqueo::calcular_segundos_de_bloqueo;
pub use credenciales::{validar_contrasena, validar_nombre_usuario, validar_pin};
pub use error::ErrorAcceso;
pub use permisos::{Accion, Permiso, Permisos, puede_realizar};
pub use rol::Rol;
