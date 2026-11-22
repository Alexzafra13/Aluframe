//! Teléfonos: en España se escriben las 9 cifras sin prefijo; el +34 se da por puesto.

use crate::texto::normalizar_para_buscar;

/// Sin país escrito se entiende España: es el caso del taller casi siempre.
pub fn es_espana(pais: Option<&str>) -> bool {
    match pais.map(normalizar_para_buscar) {
        None => true,
        Some(p) => matches!(p.as_str(), "" | "espana" | "es" | "esp" | "spain"),
    }
}

/// Devuelve el teléfono listo para guardar o el motivo por el que no vale.
/// Un número español queda como «612 345 678»; uno extranjero, tal cual se escribió.
pub fn normalizar_telefono(texto: &str, pais: Option<&str>) -> Result<String, &'static str> {
    let escrito = texto.trim();
    let caracteres_validos = escrito
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '+' | '-' | '(' | ')' | '.'));
    if !caracteres_validos {
        return Err("el teléfono solo admite cifras y espacios");
    }
    let cifras: String = escrito.chars().filter(char::is_ascii_digit).collect();
    let internacional = escrito.starts_with('+') || cifras.starts_with("00");
    let sin_prefijo_espanol = cifras
        .strip_prefix("0034")
        .or_else(|| internacional.then(|| cifras.strip_prefix("34")).flatten());
    if let Some(nacional) = sin_prefijo_espanol {
        return formatear_espanol(nacional);
    }
    if internacional || !es_espana(pais) {
        return if (6..=15).contains(&cifras.len()) {
            Ok(escrito.to_owned())
        } else {
            Err("un teléfono extranjero lleva entre 6 y 15 cifras")
        };
    }
    formatear_espanol(&cifras)
}

fn formatear_espanol(cifras: &str) -> Result<String, &'static str> {
    let empieza_bien = cifras.starts_with(['6', '7', '8', '9']);
    if cifras.len() != 9 || !empieza_bien {
        return Err("escribe las 9 cifras del teléfono, sin prefijo (ejemplo: 612 345 678)");
    }
    Ok(format!(
        "{} {} {}",
        &cifras[..3],
        &cifras[3..6],
        &cifras[6..]
    ))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn un_movil_espanol_se_guarda_agrupado() {
        assert_eq!(
            normalizar_telefono("612345678", None).unwrap(),
            "612 345 678"
        );
        assert_eq!(
            normalizar_telefono("612-34-56-78", Some("España")).unwrap(),
            "612 345 678"
        );
    }

    #[test]
    fn el_prefijo_34_se_quita_si_alguien_lo_escribe() {
        assert_eq!(
            normalizar_telefono("+34 924 55 55 55", None).unwrap(),
            "924 555 555"
        );
        assert_eq!(
            normalizar_telefono("0034612345678", None).unwrap(),
            "612 345 678"
        );
    }

    #[test]
    fn rechaza_numeros_espanoles_incompletos_o_raros() {
        assert!(normalizar_telefono("12345", None).is_err());
        assert!(normalizar_telefono("61234567", None).is_err());
        assert!(normalizar_telefono("512345678", None).is_err());
        assert!(normalizar_telefono("llámame", None).is_err());
    }

    #[test]
    fn un_numero_extranjero_se_deja_como_se_escribio() {
        assert_eq!(
            normalizar_telefono("+33 6 12 34 56 78", None).unwrap(),
            "+33 6 12 34 56 78"
        );
        assert_eq!(
            normalizar_telefono("06 12 34 56 78", Some("Francia")).unwrap(),
            "06 12 34 56 78"
        );
        assert!(normalizar_telefono("+33 12", None).is_err());
    }

    #[test]
    fn sin_pais_se_entiende_espana() {
        assert!(es_espana(None));
        assert!(es_espana(Some(" ESPAÑA ")));
        assert!(!es_espana(Some("Portugal")));
    }
}
