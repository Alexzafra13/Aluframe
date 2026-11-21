use serde::{Deserialize, Serialize};

/// Importe en céntimos de euro.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Dinero(i64);

impl Dinero {
    pub const CERO: Dinero = Dinero(0);

    pub fn desde_centimos(centimos: i64) -> Dinero {
        Dinero(centimos)
    }

    pub fn centimos(self) -> i64 {
        self.0
    }

    pub fn sumar(self, otro: Dinero) -> Option<Dinero> {
        self.0.checked_add(otro.0).map(Dinero)
    }

    pub fn restar(self, otro: Dinero) -> Option<Dinero> {
        self.0.checked_sub(otro.0).map(Dinero)
    }

    /// Formato es-ES: «1.234,56 €».
    pub fn formatear(self) -> String {
        let signo = if self.0 < 0 { "-" } else { "" };
        let absoluto = self.0.unsigned_abs();
        let euros = (absoluto / 100).to_string();
        let mut con_puntos = String::with_capacity(euros.len() + euros.len() / 3);
        for (indice, cifra) in euros.chars().enumerate() {
            if indice > 0 && (euros.len() - indice).is_multiple_of(3) {
                con_puntos.push('.');
            }
            con_puntos.push(cifra);
        }
        format!("{signo}{con_puntos},{:02} €", absoluto % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatea_en_espanol() {
        assert_eq!(Dinero::desde_centimos(123_456).formatear(), "1.234,56 €");
        assert_eq!(Dinero::desde_centimos(5).formatear(), "0,05 €");
        assert_eq!(
            Dinero::desde_centimos(100_000_000).formatear(),
            "1.000.000,00 €"
        );
        assert_eq!(Dinero::desde_centimos(-1_050).formatear(), "-10,50 €");
    }

    #[test]
    fn suma_y_resta_sin_desbordar_en_silencio() {
        let diez = Dinero::desde_centimos(1_000);
        assert_eq!(diez.sumar(diez), Some(Dinero::desde_centimos(2_000)));
        assert_eq!(Dinero::desde_centimos(i64::MAX).sumar(diez), None);
        assert_eq!(diez.restar(diez), Some(Dinero::CERO));
    }
}
