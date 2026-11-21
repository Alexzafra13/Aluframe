use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

use crate::errores::ErrorApp;

pub fn cifrar_clave(clave: &str) -> Result<String, ErrorApp> {
    Argon2::default()
        .hash_password(clave.as_bytes())
        .map(|huella| huella.to_string())
        .map_err(|error| ErrorApp::Interno(format!("argon2: {error}")))
}

pub fn comprobar_clave(clave: &str, huella: &str) -> bool {
    Argon2::default()
        .verify_password(clave.as_bytes(), huella)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_huella_es_argon2id_y_verifica_la_clave_correcta() {
        let huella = cifrar_clave("una clave larga").unwrap();
        assert!(huella.starts_with("$argon2id$"));
        assert!(comprobar_clave("una clave larga", &huella));
        assert!(!comprobar_clave("otra clave", &huella));
    }

    #[test]
    fn una_huella_corrupta_no_verifica() {
        assert!(!comprobar_clave("lo que sea", "no-es-una-huella"));
    }
}
