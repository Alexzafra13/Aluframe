use serde::{Deserialize, Serialize};

use super::numero_es::{formatear_es, leer_decimal_es};
use super::{Dinero, ErrorUnidad, Porcentaje, SentidoRedondeo};

const ESCALA: i128 = 10_000;

/// Precio unitario en diezmilésimas de euro (DATOS 7.1, `*_pu`): las tarifas dan precios
/// por metro o por unidad con más de dos decimales y redondear a céntimos antes de
/// multiplicar por la cantidad perdería dinero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PrecioUnitario(i64);

impl PrecioUnitario {
    pub fn desde_diezmilesimas(valor: i64) -> PrecioUnitario {
        PrecioUnitario(valor)
    }

    pub fn diezmilesimas(self) -> i64 {
        self.0
    }

    /// Lee un importe en euros escrito a la española, con hasta 4 decimales.
    pub fn desde_texto_es(texto: &str) -> Result<PrecioUnitario, ErrorUnidad> {
        let valor = leer_decimal_es(texto)?;
        i64::try_from(valor.unidades())
            .map(PrecioUnitario)
            .map_err(|_| ErrorUnidad::Desbordamiento)
    }

    pub fn es_positivo(self) -> bool {
        self.0 > 0
    }

    /// «12,3456 €» o «12,30 €»: siempre al menos los céntimos.
    pub fn formatear(self) -> String {
        format!("{} €", self.formatear_numero())
    }

    /// El número sin símbolo, para rellenar un campo de formulario.
    pub fn formatear_numero(self) -> String {
        formatear_es(i128::from(self.0), 4, 2)
    }

    /// Sube o baja un porcentaje redondeando a la diezmilésima, con la mitad alejándose de cero.
    pub fn aplicar_porcentaje(self, porcentaje: Porcentaje) -> Option<PrecioUnitario> {
        let factor = ESCALA.checked_add(i128::from(porcentaje.puntos_basicos()))?;
        let producto = i128::from(self.0).checked_mul(factor)?;
        let resultado = SentidoRedondeo::Cercano.dividir(producto, ESCALA)?;
        i64::try_from(resultado).ok().map(PrecioUnitario)
    }

    /// Variación respecto a un precio anterior, en puntos básicos redondeados.
    pub fn variacion_desde(self, anterior: PrecioUnitario) -> Option<Porcentaje> {
        if anterior.0 == 0 {
            return None;
        }
        let diferencia = i128::from(self.0) - i128::from(anterior.0);
        let puntos =
            SentidoRedondeo::Cercano.dividir(diferencia * ESCALA, i128::from(anterior.0))?;
        i64::try_from(puntos)
            .ok()
            .map(Porcentaje::desde_puntos_basicos)
    }

    /// Importe de una cantidad en milésimas con un único redondeo a céntimos (BACKEND 5.4).
    pub fn importe(self, cantidad_mil: i64) -> Option<Dinero> {
        let producto = i128::from(self.0).checked_mul(i128::from(cantidad_mil))?;
        let centimos = SentidoRedondeo::Cercano.dividir(producto, 100 * 1000)?;
        i64::try_from(centimos).ok().map(Dinero::desde_centimos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pu(texto: &str) -> PrecioUnitario {
        PrecioUnitario::desde_texto_es(texto).unwrap()
    }

    #[test]
    fn lee_precios_con_cuatro_decimales() {
        assert_eq!(pu("12,3456").diezmilesimas(), 123_456);
        assert_eq!(pu("12,5").diezmilesimas(), 125_000);
        assert!(PrecioUnitario::desde_texto_es("1,23456").is_err());
    }

    #[test]
    fn formatea_al_menos_con_centimos() {
        assert_eq!(pu("12,3").formatear(), "12,30 €");
        assert_eq!(pu("1234,5678").formatear(), "1.234,5678 €");
        assert_eq!(pu("7").formatear_numero(), "7,00");
    }

    #[test]
    fn aplica_porcentajes_con_un_redondeo() {
        let subir = Porcentaje::desde_puntos_basicos(1_000);
        let bajar = Porcentaje::desde_puntos_basicos(-250);
        assert_eq!(pu("10").aplicar_porcentaje(subir), Some(pu("11")));
        assert_eq!(
            pu("0,0001").aplicar_porcentaje(Porcentaje::desde_puntos_basicos(5_000)),
            Some(pu("0,0002"))
        );
        assert_eq!(pu("10").aplicar_porcentaje(bajar), Some(pu("9,75")));
        assert_eq!(PrecioUnitario(i64::MAX).aplicar_porcentaje(subir), None);
    }

    #[test]
    fn calcula_la_variacion_en_puntos_basicos() {
        assert_eq!(
            pu("13").variacion_desde(pu("10")),
            Some(Porcentaje::desde_puntos_basicos(3_000))
        );
        assert_eq!(
            pu("9").variacion_desde(pu("12")),
            Some(Porcentaje::desde_puntos_basicos(-2_500))
        );
        assert_eq!(pu("9").variacion_desde(PrecioUnitario(0)), None);
    }

    #[test]
    fn el_importe_redondea_una_sola_vez() {
        assert_eq!(
            pu("0,3333").importe(3_000),
            Some(Dinero::desde_centimos(100))
        );
        assert_eq!(
            pu("12,345").importe(1_000),
            Some(Dinero::desde_centimos(1_235))
        );
    }
}
