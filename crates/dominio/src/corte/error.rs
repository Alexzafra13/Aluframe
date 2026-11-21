use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorCorte {
    #[error("La barra no deja longitud útil tras el recorte de los extremos.")]
    SinLongitudUtil,
    #[error("La pieza {id} mide {longitud} cm y no cabe en una barra.")]
    PiezaDemasiadoLarga { id: u32, longitud: String },
    #[error("La pieza {0} no tiene una longitud positiva.")]
    PiezaSinLongitud(u32),
    #[error("La pieza {0} aparece dos veces.")]
    PiezaRepetida(u32),
    #[error("El cálculo de longitudes se ha desbordado.")]
    Desbordamiento,
}
