use super::{Decimal, ErrorUnidad};

/// Lee un número escrito a la española: «1.234,5678», «12,5», «12.5» o «12,50 €».
/// Con coma, los puntos son separadores de miles; sin coma, el punto es el decimal,
/// que es como lo escriben los CSV exportados en inglés.
pub fn leer_decimal_es(texto: &str) -> Result<Decimal, ErrorUnidad> {
    let limpio: String = texto
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '€' | '%'))
        .collect();
    let normalizado = if limpio.contains(',') {
        limpio.replace('.', "").replace(',', ".")
    } else {
        limpio
    };
    Decimal::desde_texto(&normalizado).map_err(|error| match error {
        ErrorUnidad::NumeroNoValido(_) => ErrorUnidad::NumeroNoValido(texto.trim().to_owned()),
        ErrorUnidad::DemasiadosDecimales(_) => {
            ErrorUnidad::DemasiadosDecimales(texto.trim().to_owned())
        }
        otro => otro,
    })
}

/// Escribe un número con coma decimal y puntos de miles, con un mínimo de decimales.
pub fn formatear_es(unidades: i128, escala_decimales: u32, minimo: usize) -> String {
    let escala = 10_i128.pow(escala_decimales);
    let signo = if unidades < 0 { "-" } else { "" };
    let absoluto = unidades.unsigned_abs();
    let escala_u = escala.unsigned_abs();
    let entera = (absoluto / escala_u).to_string();
    let ancho = usize::try_from(escala_decimales).unwrap_or(0);
    let fraccion = format!("{:0ancho$}", absoluto % escala_u);
    let mut decimales = fraccion.trim_end_matches('0').to_owned();
    while decimales.len() < minimo {
        decimales.push('0');
    }
    let mut miles = String::with_capacity(entera.len() + entera.len() / 3);
    for (indice, cifra) in entera.chars().enumerate() {
        if indice > 0 && (entera.len() - indice).is_multiple_of(3) {
            miles.push('.');
        }
        miles.push(cifra);
    }
    if decimales.is_empty() {
        format!("{signo}{miles}")
    } else {
        format!("{signo}{miles},{decimales}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(texto: &str) -> Decimal {
        Decimal::desde_texto(texto).unwrap()
    }

    #[test]
    fn lee_numeros_a_la_espanola_y_a_la_inglesa() {
        assert_eq!(leer_decimal_es("12,5").unwrap(), d("12.5"));
        assert_eq!(leer_decimal_es("1.234,5678 €").unwrap(), d("1234.5678"));
        assert_eq!(leer_decimal_es("12.50").unwrap(), d("12.5"));
        assert_eq!(leer_decimal_es(" 7 ").unwrap(), d("7"));
        assert_eq!(leer_decimal_es("-30 %").unwrap(), d("-30"));
    }

    #[test]
    fn los_errores_citan_el_texto_original() {
        assert_eq!(
            leer_decimal_es("doce"),
            Err(ErrorUnidad::NumeroNoValido("doce".into()))
        );
        assert_eq!(
            leer_decimal_es("1,23456"),
            Err(ErrorUnidad::DemasiadosDecimales("1,23456".into()))
        );
    }

    #[test]
    fn formatea_con_coma_y_miles() {
        assert_eq!(formatear_es(12_345_000, 4, 2), "1.234,50");
        assert_eq!(formatear_es(123_456, 4, 2), "12,3456");
        assert_eq!(formatear_es(-5, 2, 0), "-0,05");
        assert_eq!(formatear_es(1500, 2, 0), "15");
    }
}
