//! NIF, NIE y CIF españoles con su carácter de control (algoritmos públicos de la AEAT).

use std::fmt;

use thiserror::Error;

const LETRAS_DNI: &[u8; 23] = b"TRWAGMYFPDXBNJZSQVHLCKE";
const LETRAS_CIF: &[u8; 10] = b"JABCDEFGHI";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nif(String);

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorNif {
    #[error("el NIF debe tener 9 caracteres (por ejemplo 12345678Z o B12345674)")]
    LongitudIncorrecta,
    #[error("el formato no corresponde a un NIF, NIE ni CIF")]
    FormatoNoValido,
    #[error("la letra o el dígito de control no coincide: revisa que esté bien escrito")]
    ControlIncorrecto,
}

impl Nif {
    /// Admite espacios, guiones, puntos y minúsculas: «12.345.678-z» se guarda «12345678Z».
    pub fn desde_texto(texto: &str) -> Result<Nif, ErrorNif> {
        let limpio: String = texto
            .chars()
            .filter(|c| !matches!(c, ' ' | '-' | '.'))
            .flat_map(char::to_uppercase)
            .collect();
        if limpio.len() != 9 || !limpio.is_ascii() {
            return Err(ErrorNif::LongitudIncorrecta);
        }
        let bytes = limpio.as_bytes();
        let correcto = match bytes.first() {
            Some(b'0'..=b'9') => comprobar_dni(bytes)?,
            Some(b'X' | b'Y' | b'Z') => comprobar_nie(bytes)?,
            Some(b'K' | b'L' | b'M') => comprobar_dni_especial(bytes)?,
            Some(letra) if b"ABCDEFGHJNPQRSUVW".contains(letra) => comprobar_cif(bytes)?,
            _ => return Err(ErrorNif::FormatoNoValido),
        };
        if correcto {
            Ok(Nif(limpio))
        } else {
            Err(ErrorNif::ControlIncorrecto)
        }
    }

    pub fn como_texto(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Nif {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn numero(cifras: &[u8]) -> Result<u32, ErrorNif> {
    if cifras.is_empty() || !cifras.iter().all(u8::is_ascii_digit) {
        return Err(ErrorNif::FormatoNoValido);
    }
    Ok(cifras
        .iter()
        .fold(0, |total, c| total * 10 + u32::from(c - b'0')))
}

fn letra_de_control(valor: u32) -> u8 {
    let indice = usize::try_from(valor % 23).unwrap_or(0);
    LETRAS_DNI.get(indice).copied().unwrap_or(b'T')
}

fn comprobar_dni(nif: &[u8]) -> Result<bool, ErrorNif> {
    let valor = numero(nif.get(..8).unwrap_or_default())?;
    Ok(nif.get(8) == Some(&letra_de_control(valor)))
}

fn comprobar_nie(nif: &[u8]) -> Result<bool, ErrorNif> {
    let prefijo = match nif.first() {
        Some(b'X') => 0,
        Some(b'Y') => 1,
        _ => 2,
    };
    let valor = prefijo * 10_000_000 + numero(nif.get(1..8).unwrap_or_default())?;
    Ok(nif.get(8) == Some(&letra_de_control(valor)))
}

/// K, L y M (españoles sin DNI y extranjeros menores) usan la letra del DNI sobre sus 7 cifras.
fn comprobar_dni_especial(nif: &[u8]) -> Result<bool, ErrorNif> {
    let valor = numero(nif.get(1..8).unwrap_or_default())?;
    Ok(nif.get(8) == Some(&letra_de_control(valor)))
}

fn comprobar_cif(cif: &[u8]) -> Result<bool, ErrorNif> {
    let cifras = cif.get(1..8).unwrap_or_default();
    numero(cifras)?;
    let mut suma = 0;
    for (posicion, cifra) in cifras.iter().enumerate() {
        let valor = u32::from(cifra - b'0');
        suma += if posicion % 2 == 0 {
            (valor * 2) / 10 + (valor * 2) % 10
        } else {
            valor
        };
    }
    let digito = (10 - suma % 10) % 10;
    let letra = LETRAS_CIF
        .get(usize::try_from(digito).unwrap_or(0))
        .copied()
        .unwrap_or(b'J');
    let cifra = b'0' + u8::try_from(digito).unwrap_or(0);
    let control = cif.get(8).copied().unwrap_or_default();
    Ok(match cif.first() {
        Some(b'P' | b'Q' | b'R' | b'S' | b'N' | b'W') => control == letra,
        Some(b'A' | b'B' | b'E' | b'H') => control == cifra,
        _ => control == letra || control == cifra,
    })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn acepta_dni_nie_y_cif_correctos() {
        for valido in [
            "12345678Z",
            "X1234567L",
            "B12345674",
            "Q2826000H",
            "00000000T",
        ] {
            assert!(Nif::desde_texto(valido).is_ok(), "{valido}");
        }
    }

    #[test]
    fn normaliza_la_forma_de_escribirlo() {
        let nif = Nif::desde_texto(" 12.345.678-z ").unwrap();
        assert_eq!(nif.como_texto(), "12345678Z");
        assert_eq!(nif.to_string(), "12345678Z");
    }

    #[test]
    fn rechaza_la_letra_de_control_equivocada() {
        assert_eq!(
            Nif::desde_texto("12345678A"),
            Err(ErrorNif::ControlIncorrecto)
        );
        assert_eq!(
            Nif::desde_texto("B12345675"),
            Err(ErrorNif::ControlIncorrecto)
        );
        assert_eq!(
            Nif::desde_texto("Q28260008"),
            Err(ErrorNif::ControlIncorrecto)
        );
    }

    #[test]
    fn rechaza_formatos_que_no_son_nif() {
        assert_eq!(Nif::desde_texto("1234"), Err(ErrorNif::LongitudIncorrecta));
        assert_eq!(
            Nif::desde_texto("I12345678"),
            Err(ErrorNif::FormatoNoValido)
        );
        assert_eq!(
            Nif::desde_texto("1234567AZ"),
            Err(ErrorNif::FormatoNoValido)
        );
        assert_eq!(
            Nif::desde_texto("ññññññññ1"),
            Err(ErrorNif::LongitudIncorrecta)
        );
    }

    #[test]
    fn el_cif_de_letra_mixta_admite_digito_o_letra() {
        assert!(Nif::desde_texto("G12345674").is_ok());
        assert!(Nif::desde_texto("G1234567D").is_ok());
    }

    proptest! {
        #[test]
        fn cualquier_texto_da_nif_o_error_sin_romperse(texto in "\\PC{0,12}") {
            let _ = Nif::desde_texto(&texto);
        }

        #[test]
        fn todo_dni_con_su_letra_es_valido(valor in 0u32..100_000_000) {
            let texto = format!("{valor:08}{}", char::from(letra_de_control(valor)));
            prop_assert!(Nif::desde_texto(&texto).is_ok());
        }
    }
}
