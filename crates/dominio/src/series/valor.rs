/// Árbol neutro de un archivo de serie: el servidor lo construye desde YAML
/// y el dominio lo valida sin depender de ninguna biblioteca de formatos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Valor {
    Texto(String),
    Numero(String),
    Booleano(bool),
    Lista(Vec<Valor>),
    Mapa(Vec<(String, Valor)>),
    Nulo,
}

pub const CONFIRMAR: &str = "CONFIRMAR";

impl Valor {
    pub fn es_confirmar(&self) -> bool {
        matches!(self, Valor::Texto(texto) if texto == CONFIRMAR)
    }

    pub fn describir(&self) -> &'static str {
        match self {
            Valor::Texto(_) => "un texto",
            Valor::Numero(_) => "un número",
            Valor::Booleano(_) => "un sí/no",
            Valor::Lista(_) => "una lista",
            Valor::Mapa(_) => "un bloque de campos",
            Valor::Nulo => "un valor vacío",
        }
    }
}

/// Dato que puede faltar a propósito: `CONFIRMAR` en el archivo de la serie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dato<T> {
    Valor(T),
    Confirmar,
}

impl<T> Dato<T> {
    pub fn valor(&self) -> Option<&T> {
        match self {
            Dato::Valor(valor) => Some(valor),
            Dato::Confirmar => None,
        }
    }

    pub fn es_confirmar(&self) -> bool {
        matches!(self, Dato::Confirmar)
    }
}
