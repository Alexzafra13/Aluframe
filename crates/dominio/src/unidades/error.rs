use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorUnidad {
    #[error("«{0}» no es un número válido (usa punto decimal, por ejemplo 4.85).")]
    NumeroNoValido(String),
    #[error("«{0}» tiene más de 4 decimales.")]
    DemasiadosDecimales(String),
    #[error("El número es demasiado grande.")]
    Desbordamiento,
    #[error("División por cero.")]
    DivisionPorCero,
    #[error("La medida {0} no es exacta en décimas de milímetro.")]
    MedidaNoExacta(String),
}
