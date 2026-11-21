use serde::{Deserialize, Serialize};

use super::{Decimal, ErrorUnidad};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unidad {
    Cm,
    Mm,
}

impl Unidad {
    pub fn desde_texto(texto: &str) -> Option<Unidad> {
        match texto {
            "cm" => Some(Unidad::Cm),
            "mm" => Some(Unidad::Mm),
            _ => None,
        }
    }

    fn decimas_de_mm(self) -> i128 {
        match self {
            Unidad::Cm => 100,
            Unidad::Mm => 10,
        }
    }
}

/// Longitud en décimas de milímetro (640 cm = 64 000).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Longitud(i64);

impl Longitud {
    pub const CERO: Longitud = Longitud(0);

    pub fn desde_decimas_de_mm(valor: i64) -> Longitud {
        Longitud(valor)
    }

    pub fn decimas_de_mm(self) -> i64 {
        self.0
    }

    pub fn desde_decimal(valor: Decimal, unidad: Unidad) -> Result<Longitud, ErrorUnidad> {
        let ampliado = valor
            .unidades()
            .checked_mul(unidad.decimas_de_mm())
            .ok_or(ErrorUnidad::Desbordamiento)?;
        if ampliado % 10_000 != 0 {
            return Err(ErrorUnidad::MedidaNoExacta(valor.to_string()));
        }
        i64::try_from(ampliado / 10_000)
            .map(Longitud)
            .map_err(|_| ErrorUnidad::Desbordamiento)
    }

    pub fn a_decimal(self, unidad: Unidad) -> Result<Decimal, ErrorUnidad> {
        let unidades = i128::from(self.0) * 10_000 / unidad.decimas_de_mm();
        Decimal::desde_unidades(unidades)
    }

    pub fn sumar(self, otra: Longitud) -> Option<Longitud> {
        self.0.checked_add(otra.0).map(Longitud)
    }

    pub fn restar(self, otra: Longitud) -> Option<Longitud> {
        self.0.checked_sub(otra.0).map(Longitud)
    }

    pub fn formatear_mm(self) -> String {
        self.a_decimal(Unidad::Mm)
            .map(|d| d.to_string().replace('.', ","))
            .unwrap_or_default()
    }

    /// Formato de pantalla en cm con coma decimal: 1459 décimas de mm se muestran «145,9».
    pub fn formatear_cm(self) -> String {
        let texto = self
            .a_decimal(Unidad::Cm)
            .map(|d| d.to_string())
            .unwrap_or_default();
        texto.replace('.', ",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cm(texto: &str) -> Longitud {
        Longitud::desde_decimal(Decimal::desde_texto(texto).unwrap(), Unidad::Cm).unwrap()
    }

    #[test]
    fn convierte_cm_a_decimas_de_milimetro() {
        assert_eq!(cm("640").decimas_de_mm(), 64_000);
        assert_eq!(cm("145.9").decimas_de_mm(), 14_590);
        assert_eq!(cm("57.57").decimas_de_mm(), 5_757);
    }

    #[test]
    fn rechaza_medidas_por_debajo_de_la_decima_de_milimetro() {
        let fino = Decimal::desde_texto("57.5751").unwrap();
        assert!(Longitud::desde_decimal(fino, Unidad::Cm).is_err());
    }

    #[test]
    fn convierte_milimetros() {
        let mm =
            Longitud::desde_decimal(Decimal::desde_texto("1459").unwrap(), Unidad::Mm).unwrap();
        assert_eq!(mm, cm("145.9"));
    }

    #[test]
    fn se_muestra_en_cm_con_coma() {
        assert_eq!(cm("145.9").formatear_cm(), "145,9");
        assert_eq!(cm("120").formatear_cm(), "120");
        assert_eq!(Longitud::desde_decimas_de_mm(5_757).formatear_cm(), "57,57");
        assert_eq!(Longitud::desde_decimas_de_mm(25).formatear_mm(), "2,5");
    }
}
