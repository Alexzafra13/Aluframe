mod arbol;
mod error;
mod evaluacion;
mod lexico;
mod sintaxis;

#[cfg(test)]
mod pruebas;

pub use arbol::{Comparador, Condicion, Expresion, Funcion, Operador};
pub use error::ErrorFormula;
pub use evaluacion::Valores;
pub use sintaxis::PROFUNDIDAD_MAXIMA;

use crate::unidades::Decimal;

pub const LONGITUD_MAXIMA: usize = 200;

/// Fórmula de una serie ya analizada: se analiza una vez al cargar la serie y se evalúa muchas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formula {
    texto: String,
    expresion: Expresion,
}

impl Formula {
    pub fn analizar(
        texto: &str,
        es_variable: &dyn Fn(&str) -> bool,
    ) -> Result<Formula, ErrorFormula> {
        if texto.chars().count() > LONGITUD_MAXIMA {
            return Err(ErrorFormula::DemasiadoLarga {
                maximo: LONGITUD_MAXIMA,
            });
        }
        let piezas = lexico::separar_en_simbolos(texto)?;
        let expresion = sintaxis::Analizador::new(&piezas, es_variable).analizar_todo()?;
        Ok(Formula {
            texto: texto.trim().to_owned(),
            expresion,
        })
    }

    pub fn evaluar(&self, valores: &Valores) -> Result<Decimal, ErrorFormula> {
        evaluacion::evaluar(&self.expresion, valores)
    }

    pub fn texto(&self) -> &str {
        &self.texto
    }

    pub fn expresion(&self) -> &Expresion {
        &self.expresion
    }

    pub fn variables(&self) -> Vec<String> {
        let mut nombres = Vec::new();
        recoger_variables(&self.expresion, &mut nombres);
        nombres.sort();
        nombres.dedup();
        nombres
    }
}

fn recoger_variables(expresion: &Expresion, nombres: &mut Vec<String>) {
    match expresion {
        Expresion::Numero(_) => {}
        Expresion::Variable(nombre) => nombres.push(nombre.clone()),
        Expresion::Negativo(interior) => recoger_variables(interior, nombres),
        Expresion::Operacion {
            izquierda, derecha, ..
        } => {
            recoger_variables(izquierda, nombres);
            recoger_variables(derecha, nombres);
        }
        Expresion::Llamada { argumentos, .. } => {
            argumentos
                .iter()
                .for_each(|a| recoger_variables(a, nombres));
        }
        Expresion::Si {
            condicion,
            entonces,
            si_no,
        } => {
            for parte in [&condicion.izquierda, &condicion.derecha, entonces, si_no] {
                recoger_variables(parte, nombres);
            }
        }
    }
}
