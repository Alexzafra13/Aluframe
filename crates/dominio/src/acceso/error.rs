use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorAcceso {
    #[error("El rol «{0}» no existe.")]
    RolDesconocido(String),
    #[error("El permiso «{0}» no existe.")]
    PermisoDesconocido(String),
    #[error("El nombre no puede estar vacío.")]
    NombreVacio,
    #[error("El nombre no puede tener más de {maximo} caracteres.")]
    NombreDemasiadoLargo { maximo: usize },
    #[error("El PIN debe tener exactamente 6 cifras.")]
    PinNoValido,
    #[error("La contraseña debe tener al menos {minimo} caracteres.")]
    ContrasenaCorta { minimo: usize },
    #[error("La contraseña no puede tener más de {maximo} caracteres.")]
    ContrasenaLarga { maximo: usize },
}
