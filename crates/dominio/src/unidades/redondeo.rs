use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentidoRedondeo {
    Abajo,
    Arriba,
    Cercano,
}

impl SentidoRedondeo {
    pub fn desde_texto(texto: &str) -> Option<SentidoRedondeo> {
        match texto {
            "abajo" => Some(SentidoRedondeo::Abajo),
            "arriba" => Some(SentidoRedondeo::Arriba),
            "cercano" => Some(SentidoRedondeo::Cercano),
            _ => None,
        }
    }

    /// Divide redondeando según el sentido; «cercano» redondea la mitad alejándose de cero.
    pub fn dividir(self, dividendo: i128, divisor: i128) -> Option<i128> {
        if divisor == 0 {
            return None;
        }
        let cociente = dividendo.checked_div(divisor)?;
        let resto = dividendo.checked_rem(divisor)?;
        if resto == 0 {
            return Some(cociente);
        }
        let negativo = (dividendo < 0) != (divisor < 0);
        let alejar = match self {
            SentidoRedondeo::Abajo => negativo,
            SentidoRedondeo::Arriba => !negativo,
            SentidoRedondeo::Cercano => resto.unsigned_abs() * 2 >= divisor.unsigned_abs(),
        };
        if !alejar {
            return Some(cociente);
        }
        if negativo {
            cociente.checked_sub(1)
        } else {
            cociente.checked_add(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redondea_hacia_abajo_incluso_con_negativos() {
        assert_eq!(SentidoRedondeo::Abajo.dividir(7, 2), Some(3));
        assert_eq!(SentidoRedondeo::Abajo.dividir(-7, 2), Some(-4));
    }

    #[test]
    fn redondea_hacia_arriba() {
        assert_eq!(SentidoRedondeo::Arriba.dividir(7, 2), Some(4));
        assert_eq!(SentidoRedondeo::Arriba.dividir(-7, 2), Some(-3));
        assert_eq!(SentidoRedondeo::Arriba.dividir(8, 2), Some(4));
    }

    #[test]
    fn redondea_la_mitad_alejandose_de_cero() {
        assert_eq!(SentidoRedondeo::Cercano.dividir(5, 2), Some(3));
        assert_eq!(SentidoRedondeo::Cercano.dividir(-5, 2), Some(-3));
        assert_eq!(SentidoRedondeo::Cercano.dividir(4, 3), Some(1));
    }

    #[test]
    fn no_divide_por_cero() {
        assert_eq!(SentidoRedondeo::Cercano.dividir(5, 0), None);
    }

    #[test]
    fn lee_los_sentidos_del_formato_de_series() {
        assert_eq!(
            SentidoRedondeo::desde_texto("abajo"),
            Some(SentidoRedondeo::Abajo)
        );
        assert_eq!(SentidoRedondeo::desde_texto("hacia_abajo"), None);
    }
}
