use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use super::{CONFIRMAR, Dato, Informe, Valor};
use crate::formula::Formula;
use crate::unidades::Decimal;

pub type Campos = [(String, Valor)];

pub struct Lector {
    pub informe: Informe,
    pub variables: BTreeSet<String>,
}

pub fn ruta_hija(ruta: &str, nombre: &str) -> String {
    if ruta.is_empty() {
        nombre.to_owned()
    } else {
        format!("{ruta}.{nombre}")
    }
}

impl Lector {
    pub fn new() -> Lector {
        Lector {
            informe: Informe::default(),
            variables: ["ancho", "alto"].iter().map(|v| (*v).to_owned()).collect(),
        }
    }

    pub fn mapa<'a>(&mut self, valor: &'a Valor, ruta: &str) -> Option<&'a Campos> {
        match valor {
            Valor::Mapa(campos) => Some(campos),
            otro => self.tipo_equivocado(ruta, "un bloque de campos", otro),
        }
    }

    pub fn lista<'a>(&mut self, valor: &'a Valor, ruta: &str) -> Option<&'a [Valor]> {
        match valor {
            Valor::Lista(elementos) => Some(elementos),
            otro => self.tipo_equivocado(ruta, "una lista", otro),
        }
    }

    pub fn obligatorio<'a>(
        &mut self,
        campos: &'a Campos,
        nombre: &str,
        ruta: &str,
    ) -> Option<&'a Valor> {
        let encontrado = opcional(campos, nombre);
        if encontrado.is_none() {
            self.informe
                .error(&ruta_hija(ruta, nombre), "falta este campo obligatorio");
        }
        encontrado
    }

    pub fn solo_campos(&mut self, campos: &Campos, permitidos: &[&str], ruta: &str) {
        for (nombre, _) in campos {
            if !permitidos.contains(&nombre.as_str()) {
                let mensaje = format!("campo desconocido; se admiten: {}", permitidos.join(", "));
                self.informe.error(&ruta_hija(ruta, nombre), mensaje);
            }
        }
    }

    pub fn texto(&mut self, valor: &Valor, ruta: &str) -> Option<String> {
        match valor {
            Valor::Texto(texto) | Valor::Numero(texto) if !texto.trim().is_empty() => {
                Some(texto.trim().to_owned())
            }
            otro => self.tipo_equivocado(ruta, "un texto", otro),
        }
    }

    pub fn dato_texto(&mut self, valor: &Valor, ruta: &str) -> Option<Dato<String>> {
        if valor.es_confirmar() {
            self.anotar_confirmar(ruta);
            return Some(Dato::Confirmar);
        }
        self.texto(valor, ruta).map(Dato::Valor)
    }

    pub fn decimal(&mut self, valor: &Valor, ruta: &str) -> Option<Decimal> {
        let Valor::Numero(texto) = valor else {
            return self.tipo_equivocado(ruta, "un número", valor);
        };
        match Decimal::desde_texto(texto) {
            Ok(numero) => Some(numero),
            Err(error) => {
                self.informe.error(ruta, error.to_string());
                None
            }
        }
    }

    pub fn dato_decimal(&mut self, valor: &Valor, ruta: &str) -> Option<Dato<Decimal>> {
        if valor.es_confirmar() {
            self.anotar_confirmar(ruta);
            return Some(Dato::Confirmar);
        }
        self.decimal(valor, ruta).map(Dato::Valor)
    }

    pub fn entero(&mut self, valor: &Valor, ruta: &str, rango: RangeInclusive<i64>) -> Option<i64> {
        let numero = self.decimal(valor, ruta)?;
        match numero.a_entero().filter(|n| rango.contains(n)) {
            Some(entero) => Some(entero),
            None => {
                let mensaje = format!(
                    "debe ser un número entero entre {} y {}",
                    rango.start(),
                    rango.end()
                );
                self.informe.error(ruta, mensaje);
                None
            }
        }
    }

    pub fn formula(&mut self, valor: &Valor, ruta: &str) -> Option<Dato<Formula>> {
        if valor.es_confirmar() {
            self.anotar_confirmar(ruta);
            return Some(Dato::Confirmar);
        }
        let texto = self.texto(valor, ruta)?;
        let variables = &self.variables;
        match Formula::analizar(&texto, &|nombre| variables.contains(nombre)) {
            Ok(formula) => Some(Dato::Valor(formula)),
            Err(error) => {
                self.informe
                    .error(ruta, format!("fórmula «{texto}»: {error}"));
                None
            }
        }
    }

    pub fn anotar_confirmar(&mut self, ruta: &str) {
        self.informe
            .aviso(ruta, format!("dato pendiente ({CONFIRMAR})"));
    }

    fn tipo_equivocado<T>(&mut self, ruta: &str, esperado: &str, encontrado: &Valor) -> Option<T> {
        let mensaje = format!("se esperaba {esperado} y hay {}", encontrado.describir());
        self.informe.error(ruta, mensaje);
        None
    }
}

pub fn opcional<'a>(campos: &'a Campos, nombre: &str) -> Option<&'a Valor> {
    campos
        .iter()
        .find(|(clave, _)| clave == nombre)
        .map(|(_, valor)| valor)
}

/// Lee cada elemento aunque alguno falle, para mostrar todos los errores de una vez.
pub fn leer_lista<T>(
    lector: &mut Lector,
    valor: &Valor,
    ruta: &str,
    leer: impl Fn(&mut Lector, &Valor, &str) -> Option<T>,
) -> Option<Vec<T>> {
    let elementos = lector.lista(valor, ruta)?;
    let mut leidos = Vec::with_capacity(elementos.len());
    let mut todos_validos = true;
    for (indice, elemento) in elementos.iter().enumerate() {
        match leer(lector, elemento, &format!("{ruta}[{}]", indice + 1)) {
            Some(leido) => leidos.push(leido),
            None => todos_validos = false,
        }
    }
    todos_validos.then_some(leidos)
}
