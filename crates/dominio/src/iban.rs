//! IBAN de la empresa para el pago por transferencia: un número mal copiado en una factura
//! manda el dinero a otra cuenta, así que se comprueba su dígito de control (ISO 13616).

/// Devuelve el IBAN agrupado de cuatro en cuatro («ES91 2100 0418 4502 0005 1332»).
pub fn normalizar_iban(texto: &str) -> Result<String, &'static str> {
    let limpio: String = texto
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .flat_map(char::to_uppercase)
        .collect();
    let forma_valida = (15..=34).contains(&limpio.len())
        && limpio.chars().all(|c| c.is_ascii_alphanumeric())
        && limpio.chars().take(2).all(|c| c.is_ascii_uppercase())
        && limpio.chars().skip(2).take(2).all(|c| c.is_ascii_digit());
    if !forma_valida {
        return Err(
            "el IBAN no tiene la forma de una cuenta (ejemplo: ES91 2100 0418 4502 0005 1332)",
        );
    }
    if limpio.starts_with("ES") && limpio.len() != 24 {
        return Err("un IBAN español tiene 24 caracteres");
    }
    if resto_mod_97(&limpio) != 1 {
        return Err("el IBAN no cuadra: revisa que esté bien copiado");
    }
    let grupos: Vec<String> = limpio
        .as_bytes()
        .chunks(4)
        .map(|g| String::from_utf8_lossy(g).into_owned())
        .collect();
    Ok(grupos.join(" "))
}

/// Se mueven los 4 primeros caracteres al final y cada letra vale 10..35.
fn resto_mod_97(iban: &str) -> u32 {
    let (cabeza, cola) = iban.split_at(4);
    cola.chars().chain(cabeza.chars()).fold(0, |resto, c| {
        let valor = c.to_digit(36).unwrap_or(0);
        if valor >= 10 {
            (resto * 100 + valor) % 97
        } else {
            (resto * 10 + valor) % 97
        }
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn acepta_y_agrupa_un_iban_correcto() {
        assert_eq!(
            normalizar_iban("es9121000418450200051332").unwrap(),
            "ES91 2100 0418 4502 0005 1332"
        );
        assert_eq!(
            normalizar_iban("GB82 WEST 1234 5698 7654 32").unwrap(),
            "GB82 WEST 1234 5698 7654 32"
        );
    }

    #[test]
    fn rechaza_un_iban_mal_copiado() {
        assert!(normalizar_iban("ES91 2100 0418 4502 0005 1333").is_err());
        assert!(normalizar_iban("ES91 2100").is_err());
        assert!(normalizar_iban("1234567890123456").is_err());
        assert!(normalizar_iban("ES91 2100 0418 4502 0005 13321").is_err());
    }
}
