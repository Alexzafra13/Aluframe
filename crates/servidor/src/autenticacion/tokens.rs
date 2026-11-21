use sha2::{Digest, Sha256};

use crate::errores::ErrorApp;

const BYTES_DE_TOKEN: usize = 32;

pub fn generar_token() -> Result<String, ErrorApp> {
    let mut bytes = [0u8; BYTES_DE_TOKEN];
    getrandom::fill(&mut bytes)
        .map_err(|error| ErrorApp::Interno(format!("sin fuente aleatoria: {error}")))?;
    Ok(a_hexadecimal(&bytes))
}

/// En la base solo se guarda la huella del token: quien lea la base no puede suplantar sesiones.
pub fn huella_de_token(token: &str) -> String {
    a_hexadecimal(&Sha256::digest(token.as_bytes()))
}

pub fn tokens_iguales(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acumulado, (x, y)| acumulado | (x ^ y))
            == 0
}

fn a_hexadecimal(bytes: &[u8]) -> String {
    const CIFRAS: &[u8; 16] = b"0123456789abcdef";
    let mut texto = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        for medio in [byte >> 4, byte & 0x0f] {
            texto.push(char::from(CIFRAS[usize::from(medio)]));
        }
    }
    texto
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genera_tokens_de_256_bits_distintos() {
        let a = generar_token().unwrap();
        let b = generar_token().unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
    }

    #[test]
    fn la_huella_es_sha256_en_hexadecimal() {
        assert_eq!(
            huella_de_token("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn compara_tokens_sin_atajos() {
        assert!(tokens_iguales("abcd", "abcd"));
        assert!(!tokens_iguales("abcd", "abce"));
        assert!(!tokens_iguales("abc", "abcd"));
    }
}
