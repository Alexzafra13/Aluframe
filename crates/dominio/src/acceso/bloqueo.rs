const FALLOS_PERMITIDOS: u32 = 4;
const ESCALA_DE_BLOQUEO_EN_SEGUNDOS: [u64; 3] = [60, 300, 900];

/// Bloqueo progresivo tras 5 fallos seguidos: 1, 5 y 15 minutos (BACKEND.md, apartado 9).
pub fn calcular_segundos_de_bloqueo(fallos_seguidos: u32) -> Option<u64> {
    let exceso = fallos_seguidos.checked_sub(FALLOS_PERMITIDOS + 1)?;
    let indice = usize::try_from(exceso)
        .unwrap_or(usize::MAX)
        .min(ESCALA_DE_BLOQUEO_EN_SEGUNDOS.len() - 1);
    ESCALA_DE_BLOQUEO_EN_SEGUNDOS.get(indice).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_bloquea_los_cuatro_primeros_fallos() {
        for fallos in 0..=4 {
            assert_eq!(calcular_segundos_de_bloqueo(fallos), None);
        }
    }

    #[test]
    fn bloquea_de_forma_progresiva() {
        assert_eq!(calcular_segundos_de_bloqueo(5), Some(60));
        assert_eq!(calcular_segundos_de_bloqueo(6), Some(300));
        assert_eq!(calcular_segundos_de_bloqueo(7), Some(900));
        assert_eq!(calcular_segundos_de_bloqueo(u32::MAX), Some(900));
    }
}
