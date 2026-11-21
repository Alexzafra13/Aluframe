use std::collections::BTreeMap;

use super::ErrorFormula;
use super::arbol::{Comparador, Condicion, Expresion, Funcion, Operador};
use crate::unidades::{Decimal, SentidoRedondeo};

pub type Valores = BTreeMap<String, Decimal>;

pub fn evaluar(expresion: &Expresion, valores: &Valores) -> Result<Decimal, ErrorFormula> {
    match expresion {
        Expresion::Numero(valor) => Ok(*valor),
        Expresion::Variable(nombre) => valores
            .get(nombre)
            .copied()
            .ok_or_else(|| ErrorFormula::VariableDesconocida(nombre.clone())),
        Expresion::Negativo(interior) => Ok(Decimal::CERO.restar(evaluar(interior, valores)?)?),
        Expresion::Operacion {
            operador,
            izquierda,
            derecha,
        } => operar(
            *operador,
            evaluar(izquierda, valores)?,
            evaluar(derecha, valores)?,
        ),
        Expresion::Llamada {
            funcion,
            argumentos,
        } => llamar(*funcion, argumentos, valores),
        Expresion::Si {
            condicion,
            entonces,
            si_no,
        } => {
            let rama = if cumple(condicion, valores)? {
                entonces
            } else {
                si_no
            };
            evaluar(rama, valores)
        }
    }
}

fn operar(operador: Operador, a: Decimal, b: Decimal) -> Result<Decimal, ErrorFormula> {
    let resultado = match operador {
        Operador::Suma => a.sumar(b),
        Operador::Resta => a.restar(b),
        Operador::Producto => a.multiplicar(b),
        Operador::Cociente => a.dividir(b),
    };
    Ok(resultado?)
}

fn llamar(
    funcion: Funcion,
    argumentos: &[Expresion],
    valores: &Valores,
) -> Result<Decimal, ErrorFormula> {
    let mut resultados = Vec::with_capacity(argumentos.len());
    for argumento in argumentos {
        resultados.push(evaluar(argumento, valores)?);
    }
    let primero = *resultados.first().ok_or(ErrorFormula::FinInesperado)?;
    let resultado = match funcion {
        Funcion::Minimo => resultados.into_iter().min().unwrap_or(primero),
        Funcion::Maximo => resultados.into_iter().max().unwrap_or(primero),
        Funcion::Redondear => primero.redondear(Decimal::UNO, SentidoRedondeo::Cercano)?,
        Funcion::Techo => primero.redondear(Decimal::UNO, SentidoRedondeo::Arriba)?,
        Funcion::Suelo => primero.redondear(Decimal::UNO, SentidoRedondeo::Abajo)?,
    };
    Ok(resultado)
}

fn cumple(condicion: &Condicion, valores: &Valores) -> Result<bool, ErrorFormula> {
    let a = evaluar(&condicion.izquierda, valores)?;
    let b = evaluar(&condicion.derecha, valores)?;
    Ok(match condicion.comparador {
        Comparador::Mayor => a > b,
        Comparador::MayorOIgual => a >= b,
        Comparador::Menor => a < b,
        Comparador::MenorOIgual => a <= b,
        Comparador::Igual => a == b,
    })
}
