use thiserror::Error;

use super::tabla::{FilaCsv, TablaCsv, crear_tabla};

/// Una tarifa de varios miles de artículos ocupa unos cientos de KB.
pub const TAMANO_MAXIMO: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorCsv {
    #[error("el archivo está vacío")]
    Vacio,
    #[error("el archivo es demasiado grande (máximo 2 MB)")]
    DemasiadoGrande,
    #[error("línea {0}: hay unas comillas sin cerrar")]
    ComillaSinCerrar(usize),
    #[error("línea {0}: después de unas comillas de cierre solo puede ir el separador")]
    TextoTrasComillas(usize),
}

pub fn leer_csv(texto: &str) -> Result<TablaCsv, ErrorCsv> {
    if texto.len() > TAMANO_MAXIMO {
        return Err(ErrorCsv::DemasiadoGrande);
    }
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let separador = detectar_separador(texto);
    let registros = Lector::new(separador).leer(texto)?;
    crear_tabla(registros).ok_or(ErrorCsv::Vacio)
}

/// El separador más frecuente en la primera línea; Excel en español usa `;`.
fn detectar_separador(texto: &str) -> char {
    let primera = texto.lines().next().unwrap_or_default();
    let contar = |s: char| primera.chars().filter(|&c| c == s).count();
    [';', ',', '\t']
        .into_iter()
        .max_by_key(|&s| (contar(s), s == ';'))
        .unwrap_or(';')
}

struct Lector {
    separador: char,
    registros: Vec<FilaCsv>,
    campos: Vec<String>,
    campo: String,
    linea: usize,
    inicio: usize,
    entre_comillas: bool,
    tras_comillas: bool,
}

impl Lector {
    fn new(separador: char) -> Lector {
        Lector {
            separador,
            registros: Vec::new(),
            campos: Vec::new(),
            campo: String::new(),
            linea: 1,
            inicio: 1,
            entre_comillas: false,
            tras_comillas: false,
        }
    }

    fn leer(mut self, texto: &str) -> Result<Vec<FilaCsv>, ErrorCsv> {
        let mut letras = texto.chars().peekable();
        while let Some(letra) = letras.next() {
            if !self.entre_comillas {
                self.leer_fuera_de_comillas(letra)?;
            } else if letra == '"' && letras.peek() == Some(&'"') {
                letras.next();
                self.campo.push('"');
            } else {
                self.leer_entre_comillas(letra);
            }
        }
        if self.entre_comillas {
            return Err(ErrorCsv::ComillaSinCerrar(self.inicio));
        }
        self.cerrar_registro();
        Ok(self.registros)
    }

    fn leer_entre_comillas(&mut self, letra: char) {
        match letra {
            '"' => {
                self.entre_comillas = false;
                self.tras_comillas = true;
            }
            '\n' => {
                self.linea += 1;
                self.campo.push('\n');
            }
            otra => self.campo.push(otra),
        }
    }

    fn leer_fuera_de_comillas(&mut self, letra: char) -> Result<(), ErrorCsv> {
        match letra {
            '\r' => {}
            '\n' => {
                self.cerrar_registro();
                self.linea += 1;
                self.inicio = self.linea;
            }
            c if c == self.separador => self.cerrar_campo(),
            '"' if self.campo.trim().is_empty() && !self.tras_comillas => {
                self.campo.clear();
                self.entre_comillas = true;
            }
            c if self.tras_comillas && !c.is_whitespace() => {
                return Err(ErrorCsv::TextoTrasComillas(self.linea));
            }
            _ if self.tras_comillas => {}
            otra => self.campo.push(otra),
        }
        Ok(())
    }

    fn cerrar_campo(&mut self) {
        self.campos.push(std::mem::take(&mut self.campo));
        self.tras_comillas = false;
    }

    /// Las líneas en blanco (o solo con separadores) no son filas.
    fn cerrar_registro(&mut self) {
        self.cerrar_campo();
        let campos = std::mem::take(&mut self.campos);
        if campos.iter().any(|c| !c.trim().is_empty()) {
            self.registros.push(FilaCsv {
                linea: self.inicio,
                campos,
            });
        }
    }
}
