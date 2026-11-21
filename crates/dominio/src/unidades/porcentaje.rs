use serde::{Deserialize, Serialize};

use super::ErrorUnidad;
use super::numero_es::{formatear_es, leer_decimal_es};

/// Porcentaje en puntos básicos (1 % = 100), como los `*_bp` de DATOS 7.1.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Porcentaje(i64);

impl Porcentaje {
    pub const CERO: Porcentaje = Porcentaje(0);

    pub fn desde_puntos_basicos(valor: i64) -> Porcentaje {
        Porcentaje(valor)
    }

    pub fn puntos_basicos(self) -> i64 {
        self.0
    }

    /// «12,5» o «12,5 %» son 1250 puntos básicos; más de dos decimales es un error.
    pub fn desde_texto_es(texto: &str) -> Result<Porcentaje, ErrorUnidad> {
        let valor = leer_decimal_es(texto)?;
        let centesimas = valor.unidades();
        if centesimas % 100 != 0 {
            return Err(ErrorUnidad::DemasiadosDecimales(texto.trim().to_owned()));
        }
        i64::try_from(centesimas / 100)
            .map(Porcentaje)
            .map_err(|_| ErrorUnidad::Desbordamiento)
    }

    /// «12,5 %», «+30 %» con signo si se pide.
    pub fn formatear(self, con_signo: bool) -> String {
        let signo = if con_signo && self.0 > 0 { "+" } else { "" };
        format!("{signo}{} %", formatear_es(i128::from(self.0), 2, 0))
    }

    pub fn absoluto(self) -> Porcentaje {
        Porcentaje(self.0.saturating_abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_y_escribe_porcentajes() {
        let p = Porcentaje::desde_texto_es("12,5 %").unwrap();
        assert_eq!(p.puntos_basicos(), 1_250);
        assert_eq!(p.formatear(false), "12,5 %");
        assert_eq!(
            Porcentaje::desde_puntos_basicos(3_000).formatear(true),
            "+30 %"
        );
        assert_eq!(
            Porcentaje::desde_puntos_basicos(-275).formatear(true),
            "-2,75 %"
        );
        assert_eq!(
            Porcentaje::desde_puntos_basicos(-275)
                .absoluto()
                .puntos_basicos(),
            275
        );
    }

    #[test]
    fn rechaza_mas_de_dos_decimales() {
        assert!(Porcentaje::desde_texto_es("1,255").is_err());
        assert!(Porcentaje::desde_texto_es("mucho").is_err());
    }
}
