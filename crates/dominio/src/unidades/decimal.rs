use std::fmt;

use super::{ErrorUnidad, SentidoRedondeo};

const ESCALA: i128 = 10_000;
const DECIMALES: usize = 4;
const LIMITE: i128 = 1_000_000_000_000 * ESCALA;

/// Número de punto fijo con 4 decimales: el cálculo de fórmulas nunca usa `f64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(i128);

impl Decimal {
    pub const CERO: Decimal = Decimal(0);
    pub const UNO: Decimal = Decimal(ESCALA);

    pub fn desde_entero(valor: i64) -> Decimal {
        Decimal(i128::from(valor) * ESCALA)
    }

    pub fn desde_texto(texto: &str) -> Result<Decimal, ErrorUnidad> {
        let no_valido = || ErrorUnidad::NumeroNoValido(texto.to_owned());
        let (negativo, cuerpo) = match texto.strip_prefix('-') {
            Some(resto) => (true, resto),
            None => (false, texto),
        };
        let (entera, fraccion) = cuerpo.split_once('.').unwrap_or((cuerpo, ""));
        let solo_cifras = |parte: &str| parte.bytes().all(|b| b.is_ascii_digit());
        if entera.is_empty() || !solo_cifras(entera) || !solo_cifras(fraccion) {
            return Err(no_valido());
        }
        if cuerpo.contains('.') && fraccion.is_empty() {
            return Err(no_valido());
        }
        if fraccion.len() > DECIMALES {
            return Err(ErrorUnidad::DemasiadosDecimales(texto.to_owned()));
        }
        if entera.len() > 12 {
            return Err(ErrorUnidad::Desbordamiento);
        }
        let relleno = format!("{fraccion:0<DECIMALES$}");
        let unidades: i128 = format!("{entera}{relleno}")
            .parse()
            .map_err(|_| no_valido())?;
        Ok(Decimal(if negativo { -unidades } else { unidades }))
    }

    pub(crate) fn desde_unidades(unidades: i128) -> Result<Decimal, ErrorUnidad> {
        if unidades.abs() > LIMITE {
            return Err(ErrorUnidad::Desbordamiento);
        }
        Ok(Decimal(unidades))
    }

    pub(crate) fn unidades(self) -> i128 {
        self.0
    }

    pub fn sumar(self, otro: Decimal) -> Result<Decimal, ErrorUnidad> {
        Decimal::desde_unidades(
            self.0
                .checked_add(otro.0)
                .ok_or(ErrorUnidad::Desbordamiento)?,
        )
    }

    pub fn restar(self, otro: Decimal) -> Result<Decimal, ErrorUnidad> {
        Decimal::desde_unidades(
            self.0
                .checked_sub(otro.0)
                .ok_or(ErrorUnidad::Desbordamiento)?,
        )
    }

    pub fn multiplicar(self, otro: Decimal) -> Result<Decimal, ErrorUnidad> {
        let producto = self
            .0
            .checked_mul(otro.0)
            .ok_or(ErrorUnidad::Desbordamiento)?;
        let escalado = SentidoRedondeo::Cercano
            .dividir(producto, ESCALA)
            .ok_or(ErrorUnidad::Desbordamiento)?;
        Decimal::desde_unidades(escalado)
    }

    /// El cociente se redondea a 4 decimales con la mitad alejándose de cero (BACKEND.md, 5.1).
    pub fn dividir(self, otro: Decimal) -> Result<Decimal, ErrorUnidad> {
        if otro.0 == 0 {
            return Err(ErrorUnidad::DivisionPorCero);
        }
        let ampliado = self
            .0
            .checked_mul(ESCALA)
            .ok_or(ErrorUnidad::Desbordamiento)?;
        let cociente = SentidoRedondeo::Cercano
            .dividir(ampliado, otro.0)
            .ok_or(ErrorUnidad::Desbordamiento)?;
        Decimal::desde_unidades(cociente)
    }

    pub fn redondear(
        self,
        precision: Decimal,
        sentido: SentidoRedondeo,
    ) -> Result<Decimal, ErrorUnidad> {
        if precision.0 <= 0 {
            return Err(ErrorUnidad::DivisionPorCero);
        }
        let pasos = sentido
            .dividir(self.0, precision.0)
            .ok_or(ErrorUnidad::Desbordamiento)?;
        Decimal::desde_unidades(
            pasos
                .checked_mul(precision.0)
                .ok_or(ErrorUnidad::Desbordamiento)?,
        )
    }

    pub fn es_positivo(self) -> bool {
        self.0 > 0
    }

    pub fn a_entero(self) -> Option<i64> {
        if self.0 % ESCALA != 0 {
            return None;
        }
        i64::try_from(self.0 / ESCALA).ok()
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let signo = if self.0 < 0 { "-" } else { "" };
        let absoluto = self.0.unsigned_abs();
        let escala = ESCALA.unsigned_abs();
        let (entera, fraccion) = (absoluto / escala, absoluto % escala);
        if fraccion == 0 {
            return write!(f, "{signo}{entera}");
        }
        let decimales = format!("{fraccion:0DECIMALES$}");
        write!(f, "{signo}{entera}.{}", decimales.trim_end_matches('0'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(texto: &str) -> Decimal {
        Decimal::desde_texto(texto).unwrap()
    }

    #[test]
    fn lee_decimales_exactos_sin_pasar_por_coma_flotante() {
        assert_eq!(d("4.85"), Decimal(48_500));
        assert_eq!(d("120"), Decimal::desde_entero(120));
        assert_eq!(d("-0.1"), Decimal(-1_000));
        assert_eq!(d("0.0001"), Decimal(1));
    }

    #[test]
    fn rechaza_textos_que_no_son_numeros() {
        for texto in ["", "abc", "1,5", "1.", ".5", "1.2.3", "--1", "+1", "1e3"] {
            assert!(Decimal::desde_texto(texto).is_err(), "{texto}");
        }
        assert_eq!(
            Decimal::desde_texto("1.23456"),
            Err(ErrorUnidad::DemasiadosDecimales("1.23456".into()))
        );
    }

    #[test]
    fn opera_con_cuatro_decimales() {
        assert_eq!(d("150").restar(d("4.1")).unwrap(), d("145.9"));
        assert_eq!(d("115.15").dividir(d("2")).unwrap(), d("57.575"));
        assert_eq!(d("1").dividir(d("3")).unwrap(), d("0.3333"));
        assert_eq!(d("2").dividir(d("3")).unwrap(), d("0.6667"));
        assert_eq!(d("2.5").multiplicar(d("1.5")).unwrap(), d("3.75"));
    }

    #[test]
    fn la_division_por_cero_es_un_error() {
        assert_eq!(
            d("1").dividir(Decimal::CERO),
            Err(ErrorUnidad::DivisionPorCero)
        );
    }

    #[test]
    fn redondea_a_la_precision_de_la_serie() {
        let decima = d("0.1");
        assert_eq!(
            d("57.575")
                .redondear(decima, SentidoRedondeo::Abajo)
                .unwrap(),
            d("57.5")
        );
        assert_eq!(
            d("57.575")
                .redondear(decima, SentidoRedondeo::Arriba)
                .unwrap(),
            d("57.6")
        );
        assert_eq!(
            d("57.55")
                .redondear(decima, SentidoRedondeo::Cercano)
                .unwrap(),
            d("57.6")
        );
        assert_eq!(
            d("73").redondear(decima, SentidoRedondeo::Abajo).unwrap(),
            d("73")
        );
    }

    #[test]
    fn detecta_desbordamientos() {
        let enorme = d("999999999999");
        assert_eq!(enorme.multiplicar(enorme), Err(ErrorUnidad::Desbordamiento));
        assert_eq!(
            Decimal::desde_texto("9999999999999"),
            Err(ErrorUnidad::Desbordamiento)
        );
    }

    #[test]
    fn se_escribe_sin_ceros_sobrantes() {
        assert_eq!(d("57.5000").to_string(), "57.5");
        assert_eq!(d("120").to_string(), "120");
        assert_eq!(d("-0.25").to_string(), "-0.25");
    }

    #[test]
    fn solo_es_entero_si_no_tiene_decimales() {
        assert_eq!(d("4").a_entero(), Some(4));
        assert_eq!(d("4.5").a_entero(), None);
    }
}
