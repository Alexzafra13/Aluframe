use std::fmt;

use crate::unidades::Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operador {
    Suma,
    Resta,
    Producto,
    Cociente,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparador {
    Mayor,
    MayorOIgual,
    Menor,
    MenorOIgual,
    Igual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Funcion {
    Minimo,
    Maximo,
    Redondear,
    Techo,
    Suelo,
}

impl Funcion {
    pub fn desde_nombre(nombre: &str) -> Option<Funcion> {
        match nombre {
            "minimo" => Some(Funcion::Minimo),
            "maximo" => Some(Funcion::Maximo),
            "redondear" => Some(Funcion::Redondear),
            "techo" => Some(Funcion::Techo),
            "suelo" => Some(Funcion::Suelo),
            _ => None,
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            Funcion::Minimo => "minimo",
            Funcion::Maximo => "maximo",
            Funcion::Redondear => "redondear",
            Funcion::Techo => "techo",
            Funcion::Suelo => "suelo",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condicion {
    pub comparador: Comparador,
    pub izquierda: Box<Expresion>,
    pub derecha: Box<Expresion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expresion {
    Numero(Decimal),
    Variable(String),
    Negativo(Box<Expresion>),
    Operacion {
        operador: Operador,
        izquierda: Box<Expresion>,
        derecha: Box<Expresion>,
    },
    Llamada {
        funcion: Funcion,
        argumentos: Vec<Expresion>,
    },
    Si {
        condicion: Condicion,
        entonces: Box<Expresion>,
        si_no: Box<Expresion>,
    },
}

impl fmt::Display for Expresion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expresion::Numero(valor) => write!(f, "{valor}"),
            Expresion::Variable(nombre) => write!(f, "{nombre}"),
            Expresion::Negativo(interior) => write!(f, "(-{interior})"),
            Expresion::Operacion {
                operador,
                izquierda,
                derecha,
            } => write!(f, "({izquierda} {} {derecha})", signo_de(*operador)),
            Expresion::Llamada {
                funcion,
                argumentos,
            } => {
                let lista: Vec<String> = argumentos.iter().map(ToString::to_string).collect();
                write!(f, "{}({})", funcion.nombre(), lista.join(", "))
            }
            Expresion::Si {
                condicion,
                entonces,
                si_no,
            } => write!(
                f,
                "si({} {} {}, {entonces}, {si_no})",
                condicion.izquierda,
                comparacion_de(condicion.comparador),
                condicion.derecha
            ),
        }
    }
}

fn signo_de(operador: Operador) -> &'static str {
    match operador {
        Operador::Suma => "+",
        Operador::Resta => "-",
        Operador::Producto => "*",
        Operador::Cociente => "/",
    }
}

fn comparacion_de(comparador: Comparador) -> &'static str {
    match comparador {
        Comparador::Mayor => ">",
        Comparador::MayorOIgual => ">=",
        Comparador::Menor => "<",
        Comparador::MenorOIgual => "<=",
        Comparador::Igual => "==",
    }
}
