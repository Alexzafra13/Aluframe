use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problema {
    pub ruta: String,
    pub mensaje: String,
}

impl fmt::Display for Problema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ruta.is_empty() {
            write!(f, "{}", self.mensaje)
        } else {
            write!(f, "{}: {}", self.ruta, self.mensaje)
        }
    }
}

/// Resultado de importar: los errores impiden guardar; los avisos no.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Informe {
    pub errores: Vec<Problema>,
    pub avisos: Vec<Problema>,
}

impl Informe {
    pub fn error(&mut self, ruta: &str, mensaje: impl Into<String>) {
        self.errores.push(Problema {
            ruta: ruta.to_owned(),
            mensaje: mensaje.into(),
        });
    }

    pub fn aviso(&mut self, ruta: &str, mensaje: impl Into<String>) {
        self.avisos.push(Problema {
            ruta: ruta.to_owned(),
            mensaje: mensaje.into(),
        });
    }

    pub fn tiene_errores(&self) -> bool {
        !self.errores.is_empty()
    }
}
