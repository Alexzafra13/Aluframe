use thiserror::Error;

use crate::unidades::ErrorUnidad;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorFormula {
    #[error("La fórmula está vacía.")]
    Vacia,
    #[error("La fórmula tiene más de {maximo} caracteres.")]
    DemasiadoLarga { maximo: usize },
    #[error("La fórmula tiene demasiados niveles de paréntesis o funciones (máximo {maximo}).")]
    DemasiadoProfunda { maximo: usize },
    #[error("Carácter no permitido «{caracter}» en la posición {posicion}.")]
    CaracterNoPermitido { caracter: char, posicion: usize },
    #[error("Número no válido «{0}».")]
    NumeroNoValido(String),
    #[error("Sobra o falta algo cerca de la posición {posicion}: «{encontrado}».")]
    Inesperado { encontrado: String, posicion: usize },
    #[error("La fórmula termina antes de tiempo.")]
    FinInesperado,
    #[error("Nombre desconocido «{0}»: no es «ancho», «alto» ni un parámetro de la serie.")]
    VariableDesconocida(String),
    #[error("Función desconocida «{0}».")]
    FuncionDesconocida(String),
    #[error("La función «{funcion}» necesita {esperado} y tiene {recibidos}.")]
    Aridad {
        funcion: &'static str,
        esperado: &'static str,
        recibidos: usize,
    },
    #[error("Las comparaciones (>, <, ==…) solo se permiten como condición de «si».")]
    ComparacionFueraDeSi,
    #[error("El primer argumento de «si» debe ser una comparación, por ejemplo «alto > 140».")]
    SiSinComparacion,
    #[error("{0}")]
    Calculo(#[from] ErrorUnidad),
}
