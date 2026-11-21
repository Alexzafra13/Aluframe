use super::ErrorAcceso;

const MAXIMO_NOMBRE: usize = 60;
const MINIMO_CONTRASENA: usize = 10;
const MAXIMO_CONTRASENA: usize = 128;
const CIFRAS_PIN: usize = 6;

pub fn validar_nombre_usuario(nombre: &str) -> Result<String, ErrorAcceso> {
    let limpio = nombre.trim();
    if limpio.is_empty() {
        return Err(ErrorAcceso::NombreVacio);
    }
    if limpio.chars().count() > MAXIMO_NOMBRE {
        return Err(ErrorAcceso::NombreDemasiadoLargo {
            maximo: MAXIMO_NOMBRE,
        });
    }
    Ok(limpio.to_owned())
}

pub fn validar_pin(pin: &str) -> Result<(), ErrorAcceso> {
    let es_valido = pin.len() == CIFRAS_PIN && pin.bytes().all(|b| b.is_ascii_digit());
    if es_valido {
        Ok(())
    } else {
        Err(ErrorAcceso::PinNoValido)
    }
}

pub fn validar_contrasena(contrasena: &str) -> Result<(), ErrorAcceso> {
    let caracteres = contrasena.chars().count();
    if caracteres < MINIMO_CONTRASENA {
        return Err(ErrorAcceso::ContrasenaCorta {
            minimo: MINIMO_CONTRASENA,
        });
    }
    if caracteres > MAXIMO_CONTRASENA {
        return Err(ErrorAcceso::ContrasenaLarga {
            maximo: MAXIMO_CONTRASENA,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorta_espacios_del_nombre() {
        assert_eq!(validar_nombre_usuario("  Juan  "), Ok("Juan".into()));
    }

    #[test]
    fn rechaza_nombre_vacio_o_largo() {
        assert_eq!(validar_nombre_usuario("   "), Err(ErrorAcceso::NombreVacio));
        assert!(validar_nombre_usuario(&"a".repeat(61)).is_err());
    }

    #[test]
    fn acepta_pin_de_seis_cifras() {
        assert_eq!(validar_pin("012345"), Ok(()));
    }

    #[test]
    fn rechaza_pin_con_letras_o_longitud_distinta() {
        for pin in ["12345", "1234567", "12a456", "", "１２３４５６"] {
            assert_eq!(validar_pin(pin), Err(ErrorAcceso::PinNoValido), "{pin}");
        }
    }

    #[test]
    fn exige_contrasena_de_longitud_razonable() {
        assert!(validar_contrasena("corta").is_err());
        assert!(validar_contrasena("suficiente").is_ok());
        assert!(validar_contrasena(&"x".repeat(129)).is_err());
    }
}
