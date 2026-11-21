use super::ErrorFormula;
use crate::unidades::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Simbolo {
    Numero(Decimal),
    Nombre(String),
    Mas,
    Menos,
    Por,
    Entre,
    AbreParentesis,
    CierraParentesis,
    Coma,
    Mayor,
    MayorOIgual,
    Menor,
    MenorOIgual,
    Igual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pieza {
    pub simbolo: Simbolo,
    pub posicion: usize,
}

pub fn separar_en_simbolos(texto: &str) -> Result<Vec<Pieza>, ErrorFormula> {
    let caracteres: Vec<char> = texto.chars().collect();
    let mut piezas = Vec::new();
    let mut indice = 0;
    while let Some(&caracter) = caracteres.get(indice) {
        if caracter.is_whitespace() {
            indice += 1;
            continue;
        }
        let (simbolo, avance) = leer_simbolo(&caracteres, indice)?;
        piezas.push(Pieza {
            simbolo,
            posicion: indice + 1,
        });
        indice += avance;
    }
    Ok(piezas)
}

fn leer_simbolo(caracteres: &[char], inicio: usize) -> Result<(Simbolo, usize), ErrorFormula> {
    let actual = caracteres.get(inicio).copied().unwrap_or(' ');
    let siguiente = caracteres.get(inicio + 1).copied();
    let simple = match (actual, siguiente) {
        ('>', Some('=')) => Some((Simbolo::MayorOIgual, 2)),
        ('<', Some('=')) => Some((Simbolo::MenorOIgual, 2)),
        ('=', Some('=')) => Some((Simbolo::Igual, 2)),
        ('>', _) => Some((Simbolo::Mayor, 1)),
        ('<', _) => Some((Simbolo::Menor, 1)),
        ('+', _) => Some((Simbolo::Mas, 1)),
        ('-', _) => Some((Simbolo::Menos, 1)),
        ('*', _) => Some((Simbolo::Por, 1)),
        ('/', _) => Some((Simbolo::Entre, 1)),
        ('(', _) => Some((Simbolo::AbreParentesis, 1)),
        (')', _) => Some((Simbolo::CierraParentesis, 1)),
        (',', _) => Some((Simbolo::Coma, 1)),
        _ => None,
    };
    if let Some(resultado) = simple {
        return Ok(resultado);
    }
    if actual.is_ascii_digit() {
        return leer_numero(caracteres, inicio);
    }
    if actual.is_ascii_lowercase() || actual == '_' {
        let largo = contar_mientras(caracteres, inicio, |c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
        });
        let nombre: String = caracteres[inicio..inicio + largo].iter().collect();
        return Ok((Simbolo::Nombre(nombre), largo));
    }
    Err(ErrorFormula::CaracterNoPermitido {
        caracter: actual,
        posicion: inicio + 1,
    })
}

fn leer_numero(caracteres: &[char], inicio: usize) -> Result<(Simbolo, usize), ErrorFormula> {
    let largo = contar_mientras(caracteres, inicio, |c| c.is_ascii_digit() || c == '.');
    let texto: String = caracteres[inicio..inicio + largo].iter().collect();
    let numero =
        Decimal::desde_texto(&texto).map_err(|_| ErrorFormula::NumeroNoValido(texto.clone()))?;
    Ok((Simbolo::Numero(numero), largo))
}

fn contar_mientras(caracteres: &[char], inicio: usize, condicion: impl Fn(char) -> bool) -> usize {
    caracteres[inicio..]
        .iter()
        .take_while(|&&c| condicion(c))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simbolos(texto: &str) -> Vec<Simbolo> {
        separar_en_simbolos(texto)
            .unwrap()
            .into_iter()
            .map(|p| p.simbolo)
            .collect()
    }

    #[test]
    fn separa_una_formula_real() {
        assert_eq!(
            simbolos("(ancho - 4.85) / 2"),
            vec![
                Simbolo::AbreParentesis,
                Simbolo::Nombre("ancho".into()),
                Simbolo::Menos,
                Simbolo::Numero(Decimal::desde_texto("4.85").unwrap()),
                Simbolo::CierraParentesis,
                Simbolo::Entre,
                Simbolo::Numero(Decimal::desde_entero(2)),
            ]
        );
    }

    #[test]
    fn distingue_comparaciones_de_uno_y_dos_caracteres() {
        assert_eq!(
            simbolos("> >= < <= =="),
            vec![
                Simbolo::Mayor,
                Simbolo::MayorOIgual,
                Simbolo::Menor,
                Simbolo::MenorOIgual,
                Simbolo::Igual
            ]
        );
    }

    #[test]
    fn rechaza_caracteres_no_permitidos_con_su_posicion() {
        assert_eq!(
            separar_en_simbolos("ancho ^ 2"),
            Err(ErrorFormula::CaracterNoPermitido {
                caracter: '^',
                posicion: 7
            })
        );
        assert!(separar_en_simbolos("Ancho").is_err());
        assert!(separar_en_simbolos("a = 1").is_err());
    }

    #[test]
    fn rechaza_numeros_mal_escritos() {
        assert_eq!(
            separar_en_simbolos("1.2.3"),
            Err(ErrorFormula::NumeroNoValido("1.2.3".into()))
        );
    }
}
