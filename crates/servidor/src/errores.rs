use dominio::acceso::ErrorAcceso;
use dominio::apuntes::ErrorApunte;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorApp {
    #[error("{0}")]
    Validacion(String),
    #[error("Inicia sesión para continuar.")]
    SinSesion,
    #[error("No encontrado.")]
    NoEncontrado,
    #[error("Otra persona ha cambiado esto. Recarga para ver sus cambios.")]
    Conflicto,
    #[error("Demasiados intentos. Vuelve a probar dentro de {minutos} min.")]
    Bloqueado { minutos: u64 },
    #[error("Error interno: {0}")]
    Interno(String),
}

impl From<ErrorAcceso> for ErrorApp {
    fn from(error: ErrorAcceso) -> Self {
        ErrorApp::Validacion(error.to_string())
    }
}

impl From<ErrorApunte> for ErrorApp {
    fn from(error: ErrorApunte) -> Self {
        ErrorApp::Validacion(error.to_string())
    }
}

impl From<rusqlite::Error> for ErrorApp {
    fn from(error: rusqlite::Error) -> Self {
        ErrorApp::Interno(format!("base de datos: {error}"))
    }
}

impl From<askama::Error> for ErrorApp {
    fn from(error: askama::Error) -> Self {
        ErrorApp::Interno(format!("plantilla: {error}"))
    }
}
