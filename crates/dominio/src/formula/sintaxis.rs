use super::ErrorFormula;
use super::arbol::{Comparador, Condicion, Expresion, Funcion, Operador};
use super::lexico::{Pieza, Simbolo};

pub const PROFUNDIDAD_MAXIMA: usize = 16;
const PRECEDENCIA_NEGATIVO: u8 = 3;

pub struct Analizador<'a> {
    piezas: &'a [Pieza],
    indice: usize,
    es_variable: &'a dyn Fn(&str) -> bool,
}

impl<'a> Analizador<'a> {
    pub fn new(piezas: &'a [Pieza], es_variable: &'a dyn Fn(&str) -> bool) -> Analizador<'a> {
        Analizador {
            piezas,
            indice: 0,
            es_variable,
        }
    }

    pub fn analizar_todo(&mut self) -> Result<Expresion, ErrorFormula> {
        if self.piezas.is_empty() {
            return Err(ErrorFormula::Vacia);
        }
        let expresion = self.expresion(0, 0)?;
        match self.actual() {
            None => Ok(expresion),
            Some(pieza) if es_comparador(&pieza.simbolo) => Err(ErrorFormula::ComparacionFueraDeSi),
            Some(pieza) => Err(inesperado(pieza)),
        }
    }

    fn expresion(&mut self, minima: u8, profundidad: usize) -> Result<Expresion, ErrorFormula> {
        if profundidad > PROFUNDIDAD_MAXIMA {
            return Err(ErrorFormula::DemasiadoProfunda {
                maximo: PROFUNDIDAD_MAXIMA,
            });
        }
        let mut izquierda = self.prefijo(profundidad)?;
        while let Some((operador, precedencia)) =
            self.actual().and_then(|p| operador_de(&p.simbolo))
        {
            if precedencia < minima {
                break;
            }
            self.indice += 1;
            let derecha = self.expresion(precedencia + 1, profundidad + 1)?;
            izquierda = Expresion::Operacion {
                operador,
                izquierda: Box::new(izquierda),
                derecha: Box::new(derecha),
            };
        }
        Ok(izquierda)
    }

    fn prefijo(&mut self, profundidad: usize) -> Result<Expresion, ErrorFormula> {
        let pieza = self.siguiente()?;
        match &pieza.simbolo {
            Simbolo::Numero(valor) => Ok(Expresion::Numero(*valor)),
            Simbolo::Menos => {
                let interior = self.expresion(PRECEDENCIA_NEGATIVO, profundidad + 1)?;
                Ok(Expresion::Negativo(Box::new(interior)))
            }
            Simbolo::AbreParentesis => {
                let interior = self.expresion(0, profundidad + 1)?;
                self.esperar(&Simbolo::CierraParentesis)?;
                Ok(interior)
            }
            Simbolo::Nombre(nombre) => self.nombre(nombre, profundidad),
            _ => Err(inesperado(pieza)),
        }
    }

    fn nombre(&mut self, nombre: &str, profundidad: usize) -> Result<Expresion, ErrorFormula> {
        let es_llamada = self
            .actual()
            .is_some_and(|p| p.simbolo == Simbolo::AbreParentesis);
        if !es_llamada {
            return if (self.es_variable)(nombre) {
                Ok(Expresion::Variable(nombre.to_owned()))
            } else {
                Err(ErrorFormula::VariableDesconocida(nombre.to_owned()))
            };
        }
        self.indice += 1;
        if nombre == "si" {
            return self.si(profundidad + 1);
        }
        let funcion = Funcion::desde_nombre(nombre)
            .ok_or_else(|| ErrorFormula::FuncionDesconocida(nombre.to_owned()))?;
        let argumentos = self.argumentos(profundidad + 1)?;
        comprobar_aridad(funcion, argumentos.len())?;
        Ok(Expresion::Llamada {
            funcion,
            argumentos,
        })
    }

    fn si(&mut self, profundidad: usize) -> Result<Expresion, ErrorFormula> {
        let izquierda = self.expresion(0, profundidad)?;
        let comparador = self
            .actual()
            .and_then(|p| comparador_de(&p.simbolo))
            .ok_or(ErrorFormula::SiSinComparacion)?;
        self.indice += 1;
        let derecha = self.expresion(0, profundidad)?;
        let mut resto = Vec::new();
        while self.actual().is_some_and(|p| p.simbolo == Simbolo::Coma) {
            self.indice += 1;
            resto.push(self.expresion(0, profundidad)?);
        }
        self.esperar(&Simbolo::CierraParentesis)?;
        let [entonces, si_no]: [Expresion; 2] =
            resto.try_into().map_err(|r: Vec<_>| ErrorFormula::Aridad {
                funcion: "si",
                esperado: "3 argumentos",
                recibidos: r.len() + 1,
            })?;
        Ok(Expresion::Si {
            condicion: Condicion {
                comparador,
                izquierda: Box::new(izquierda),
                derecha: Box::new(derecha),
            },
            entonces: Box::new(entonces),
            si_no: Box::new(si_no),
        })
    }

    fn argumentos(&mut self, profundidad: usize) -> Result<Vec<Expresion>, ErrorFormula> {
        let mut argumentos = vec![self.expresion(0, profundidad)?];
        while self.actual().is_some_and(|p| p.simbolo == Simbolo::Coma) {
            self.indice += 1;
            argumentos.push(self.expresion(0, profundidad)?);
        }
        if self.actual().is_some_and(|p| es_comparador(&p.simbolo)) {
            return Err(ErrorFormula::ComparacionFueraDeSi);
        }
        self.esperar(&Simbolo::CierraParentesis)?;
        Ok(argumentos)
    }

    fn actual(&self) -> Option<&'a Pieza> {
        self.piezas.get(self.indice)
    }

    fn siguiente(&mut self) -> Result<&'a Pieza, ErrorFormula> {
        let pieza = self.actual().ok_or(ErrorFormula::FinInesperado)?;
        self.indice += 1;
        Ok(pieza)
    }

    fn esperar(&mut self, esperado: &Simbolo) -> Result<(), ErrorFormula> {
        let pieza = self.siguiente()?;
        if &pieza.simbolo == esperado {
            Ok(())
        } else if es_comparador(&pieza.simbolo) {
            Err(ErrorFormula::ComparacionFueraDeSi)
        } else {
            Err(inesperado(pieza))
        }
    }
}

fn comprobar_aridad(funcion: Funcion, recibidos: usize) -> Result<(), ErrorFormula> {
    let (valida, esperado) = match funcion {
        Funcion::Minimo | Funcion::Maximo => (recibidos >= 2, "al menos 2 argumentos"),
        Funcion::Redondear | Funcion::Techo | Funcion::Suelo => (recibidos == 1, "1 argumento"),
    };
    if valida {
        Ok(())
    } else {
        Err(ErrorFormula::Aridad {
            funcion: funcion.nombre(),
            esperado,
            recibidos,
        })
    }
}

fn operador_de(simbolo: &Simbolo) -> Option<(Operador, u8)> {
    match simbolo {
        Simbolo::Mas => Some((Operador::Suma, 1)),
        Simbolo::Menos => Some((Operador::Resta, 1)),
        Simbolo::Por => Some((Operador::Producto, 2)),
        Simbolo::Entre => Some((Operador::Cociente, 2)),
        _ => None,
    }
}

fn comparador_de(simbolo: &Simbolo) -> Option<Comparador> {
    match simbolo {
        Simbolo::Mayor => Some(Comparador::Mayor),
        Simbolo::MayorOIgual => Some(Comparador::MayorOIgual),
        Simbolo::Menor => Some(Comparador::Menor),
        Simbolo::MenorOIgual => Some(Comparador::MenorOIgual),
        Simbolo::Igual => Some(Comparador::Igual),
        _ => None,
    }
}

fn es_comparador(simbolo: &Simbolo) -> bool {
    comparador_de(simbolo).is_some()
}

fn inesperado(pieza: &Pieza) -> ErrorFormula {
    let encontrado = match &pieza.simbolo {
        Simbolo::Numero(valor) => valor.to_string(),
        Simbolo::Nombre(nombre) => nombre.clone(),
        Simbolo::Coma => ",".into(),
        Simbolo::AbreParentesis => "(".into(),
        Simbolo::CierraParentesis => ")".into(),
        otro => format!("{otro:?}"),
    };
    ErrorFormula::Inesperado {
        encontrado,
        posicion: pieza.posicion,
    }
}
