//! Lo que se suele cortar de cada perfil (ADR-0026): las medidas habituales que da el propietario
//! y el historial de ventanas aceptadas, con más peso lo reciente. De aquí sale la valoración de
//! los sobrantes del plan de corte.

use std::collections::BTreeMap;

use super::plan::ValoracionesPorPerfil;
use crate::corte::ValoracionDeSobrantes;
use crate::unidades::Longitud;

/// Peso de una ventana aceptada hoy.
const PESO_DE_UNA_VENTANA: u64 = 1_000;
/// Una medida habitual cuenta como cinco ventanas: orienta al principio y, con historial,
/// pesa cada vez menos.
const VENTANAS_POR_HABITUAL: u64 = 5;
/// Solo cuentan las medidas más cortadas de cada perfil: bastan para decidir y la búsqueda
/// sigue siendo rápida.
const MEDIDAS_POR_PERFIL: usize = 12;
const UN_CM_DMM: i64 = 100;

/// Una pieza de una ventana: de una medida habitual (`dias` = 0) o del historial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiezaCortada {
    pub serie: String,
    pub perfil: String,
    pub longitud: Longitud,
    pub cantidad: u32,
    /// Días entre la aceptación de esa ventana y la del trabajo que se planifica.
    pub dias: i64,
}

/// El peso se reduce a la mitad cada año, en enteros (sin coma flotante).
pub fn peso_por_antiguedad(dias: i64) -> u64 {
    let dias = dias.max(0);
    let anios = u32::try_from(dias / 365).unwrap_or(u32::MAX).min(30);
    let base = PESO_DE_UNA_VENTANA >> anios;
    let resto = u64::try_from(dias % 365).unwrap_or(0);
    base - base * resto / (2 * 365)
}

/// Las valoraciones por (serie, perfil): medidas agrupadas por centímetro (hacia arriba, para
/// no creer que cabe lo que no cabe), las más cortadas primero.
pub fn valorar_lo_habitual(
    habituales: &[PiezaCortada],
    historial: &[PiezaCortada],
) -> ValoracionesPorPerfil {
    let mut pesos: BTreeMap<(String, String), BTreeMap<i64, u64>> = BTreeMap::new();
    let de_habitual = PESO_DE_UNA_VENTANA * VENTANAS_POR_HABITUAL;
    let todas = habituales
        .iter()
        .map(|p| (p, de_habitual))
        .chain(historial.iter().map(|p| (p, peso_por_antiguedad(p.dias))));
    for (pieza, peso) in todas {
        let largo = pieza.longitud.decimas_de_mm();
        if largo <= 0 || peso == 0 {
            continue;
        }
        let centimetro = (largo + UN_CM_DMM - 1) / UN_CM_DMM * UN_CM_DMM;
        let clave = (pieza.serie.clone(), pieza.perfil.clone());
        *pesos
            .entry(clave)
            .or_default()
            .entry(centimetro)
            .or_default() += peso * u64::from(pieza.cantidad);
    }
    pesos
        .into_iter()
        .map(|(clave, por_largo)| (clave, las_mas_cortadas(por_largo)))
        .collect()
}

fn las_mas_cortadas(por_largo: BTreeMap<i64, u64>) -> ValoracionDeSobrantes {
    let mut lista: Vec<(i64, u64)> = por_largo.into_iter().collect();
    lista.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    lista.truncate(MEDIDAS_POR_PERFIL);
    ValoracionDeSobrantes {
        minimo_util: Longitud::CERO,
        habituales: lista
            .into_iter()
            .map(|(largo, peso)| {
                let peso = u32::try_from(peso).unwrap_or(u32::MAX);
                (Longitud::desde_decimas_de_mm(largo), peso)
            })
            .collect(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn pieza(perfil: &str, largo_dmm: i64, cantidad: u32, dias: i64) -> PiezaCortada {
        PiezaCortada {
            serie: "ALUPROM_21".into(),
            perfil: perfil.into(),
            longitud: Longitud::desde_decimas_de_mm(largo_dmm),
            cantidad,
            dias,
        }
    }

    fn clave(perfil: &str) -> (String, String) {
        ("ALUPROM_21".into(), perfil.into())
    }

    #[test]
    fn el_peso_se_reduce_a_la_mitad_cada_anio() {
        assert_eq!(peso_por_antiguedad(0), 1_000);
        assert_eq!(peso_por_antiguedad(365), 500);
        assert_eq!(peso_por_antiguedad(730), 250);
        assert!(peso_por_antiguedad(180) < 1_000 && peso_por_antiguedad(180) > 500);
        assert_eq!(
            peso_por_antiguedad(-3),
            1_000,
            "una fecha rara no da más peso"
        );
        assert_eq!(peso_por_antiguedad(365 * 40), 0);
    }

    #[test]
    fn una_medida_habitual_pesa_como_cinco_ventanas_y_el_historial_la_alcanza() {
        let habitual = [pieza("MARCO_LATERAL", 12_000, 2, 0)];
        let solo = valorar_lo_habitual(&habitual, &[]);
        assert_eq!(
            solo[&clave("MARCO_LATERAL")].habituales,
            vec![(Longitud::desde_decimas_de_mm(12_000), 10_000)]
        );
        let historial: Vec<PiezaCortada> = (0..6)
            .map(|_| pieza("MARCO_LATERAL", 10_000, 2, 0))
            .collect();
        let con_historial = valorar_lo_habitual(&habitual, &historial);
        let lista = &con_historial[&clave("MARCO_LATERAL")].habituales;
        assert_eq!(
            lista[0].0,
            Longitud::desde_decimas_de_mm(10_000),
            "seis ventanas reales pesan más que la habitual"
        );
    }

    #[test]
    fn agrupa_por_centimetro_hacia_arriba_y_separa_perfiles_y_series() {
        let mut otra_serie = pieza("MARCO_LATERAL", 9_000, 1, 0);
        otra_serie.serie = "SOPENA_V94".into();
        let piezas = [
            pieza("MARCO_LATERAL", 11_951, 1, 0),
            pieza("MARCO_LATERAL", 12_000, 1, 0),
            pieza("HOJA_RODAMIENTO", 5_800, 4, 0),
            otra_serie,
        ];
        let v = valorar_lo_habitual(&[], &piezas);
        assert_eq!(v.len(), 3);
        assert_eq!(
            v[&clave("MARCO_LATERAL")].habituales,
            vec![(Longitud::desde_decimas_de_mm(12_000), 2_000)]
        );
        assert_eq!(
            v[&clave("HOJA_RODAMIENTO")].habituales,
            vec![(Longitud::desde_decimas_de_mm(5_800), 4_000)]
        );
    }

    #[test]
    fn se_queda_con_las_doce_medidas_mas_cortadas() {
        let piezas: Vec<PiezaCortada> = (1..=20)
            .map(|cm| pieza("HOJA", cm * 1_000, u32::try_from(cm).unwrap(), 0))
            .collect();
        let v = valorar_lo_habitual(&[], &piezas);
        let lista = &v[&clave("HOJA")].habituales;
        assert_eq!(lista.len(), 12);
        assert_eq!(lista[0].0, Longitud::desde_decimas_de_mm(20_000));
        assert!(lista.iter().all(|(l, _)| l.decimas_de_mm() >= 9_000));
    }
}
