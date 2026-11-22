use proptest::prelude::*;

use super::*;
use crate::unidades::Longitud;

fn cm(valor: i64) -> Longitud {
    Longitud::desde_decimas_de_mm(valor * 100)
}

fn parametros(barra_cm: i64, sierra_dmm: i64) -> ParametrosDeCorte {
    ParametrosDeCorte {
        longitud_barra: cm(barra_cm),
        grosor_sierra: Longitud::desde_decimas_de_mm(sierra_dmm),
        recorte_extremos: Longitud::CERO,
        retal_minimo: cm(50),
    }
}

fn piezas(longitudes_dmm: &[i64]) -> Vec<PiezaACortar> {
    longitudes_dmm
        .iter()
        .zip(1..)
        .map(|(&l, id)| PiezaACortar {
            id,
            longitud: Longitud::desde_decimas_de_mm(l),
        })
        .collect()
}

#[test]
fn las_cuatro_hojas_de_rodamiento_caben_en_una_barra() {
    let plan = optimizar_cortes(&piezas(&[7_300; 4]), &parametros(640, 0)).unwrap();
    assert_eq!(plan.numero_de_barras(), 1);
    assert_eq!(plan.barras[0].sobrante, cm(348));
    assert_eq!(plan.retales(), vec![cm(348)]);
    assert_eq!(plan.desperdicio(), Longitud::CERO);
}

#[test]
fn el_grosor_de_sierra_puede_obligar_a_abrir_otra_barra() {
    let justas = piezas(&[32_000, 32_000]);
    assert_eq!(
        optimizar_cortes(&justas, &parametros(640, 0))
            .unwrap()
            .numero_de_barras(),
        1
    );
    assert_eq!(
        optimizar_cortes(&justas, &parametros(640, 30))
            .unwrap()
            .numero_de_barras(),
        2
    );
}

#[test]
fn el_recorte_de_extremos_reduce_la_longitud_util() {
    let mut con_recorte = parametros(640, 0);
    con_recorte.recorte_extremos = cm(2);
    let plan = optimizar_cortes(&piezas(&[64_000]), &parametros(640, 0)).unwrap();
    assert_eq!(plan.numero_de_barras(), 1);
    assert!(matches!(
        optimizar_cortes(&piezas(&[64_000]), &con_recorte),
        Err(ErrorCorte::PiezaDemasiadoLarga { id: 1, .. })
    ));
}

#[test]
fn coloca_cada_pieza_donde_menos_sobra() {
    let plan = optimizar_cortes(
        &piezas(&[40_000, 30_000, 23_000, 20_000]),
        &parametros(640, 0),
    )
    .unwrap();
    assert_eq!(plan.numero_de_barras(), 2);
    let primera: Vec<u32> = plan.barras[0].piezas.iter().map(|p| p.id).collect();
    assert_eq!(primera, vec![1, 3]);
}

#[test]
fn rechaza_datos_imposibles() {
    let mut sin_util = parametros(640, 0);
    sin_util.recorte_extremos = cm(640);
    assert_eq!(
        optimizar_cortes(&piezas(&[100]), &sin_util),
        Err(ErrorCorte::SinLongitudUtil)
    );
    let mut repetidas = piezas(&[100, 200]);
    repetidas[1].id = 1;
    assert_eq!(
        optimizar_cortes(&repetidas, &parametros(640, 0)),
        Err(ErrorCorte::PiezaRepetida(1))
    );
    assert_eq!(
        optimizar_cortes(&piezas(&[0]), &parametros(640, 0)),
        Err(ErrorCorte::PiezaSinLongitud(1))
    );
}

/// Fallo encontrado por proptest: con dos piezas demasiado largas, el error señalaba la primera
/// que llegaba, así que el resultado dependía del orden de entrada.
#[test]
fn con_varias_piezas_demasiado_largas_el_error_senala_siempre_la_misma() {
    let larga = |id| PiezaACortar {
        id,
        longitud: Longitud::desde_decimas_de_mm(63_971),
    };
    let a = optimizar_cortes(&[larga(16), larga(15)], &parametros(640, 30));
    let b = optimizar_cortes(&[larga(15), larga(16)], &parametros(640, 30));
    assert_eq!(a, b);
    assert!(matches!(
        a,
        Err(ErrorCorte::PiezaDemasiadoLarga { id: 15, .. })
    ));
}

#[test]
fn sin_piezas_no_hay_barras() {
    assert_eq!(
        optimizar_cortes(&[], &parametros(640, 0))
            .unwrap()
            .numero_de_barras(),
        0
    );
}

fn optimo_exhaustivo(longitudes: &[i64], util: i64) -> usize {
    fn colocar(resto: &[i64], barras: &mut Vec<i64>, util: i64, mejor: &mut usize) {
        if barras.len() >= *mejor {
            return;
        }
        let Some((&primera, siguientes)) = resto.split_first() else {
            *mejor = barras.len();
            return;
        };
        for indice in 0..barras.len() {
            if barras[indice] + primera <= util {
                barras[indice] += primera;
                colocar(siguientes, barras, util, mejor);
                barras[indice] -= primera;
            }
        }
        barras.push(primera);
        colocar(siguientes, barras, util, mejor);
        barras.pop();
    }
    let mut ordenadas = longitudes.to_vec();
    ordenadas.sort_unstable_by(|a, b| b.cmp(a));
    let mut mejor = ordenadas.len().max(1);
    colocar(&ordenadas, &mut Vec::new(), util, &mut mejor);
    mejor
}

proptest! {
    #[test]
    fn nunca_supera_la_barra_ni_pierde_ni_duplica_piezas(
        longitudes in prop::collection::vec(1i64..=64_000, 0..60),
        sierra in 0i64..=40,
    ) {
        let parametros = parametros(640, sierra);
        let lista = piezas(&longitudes);
        let Ok(plan) = optimizar_cortes(&lista, &parametros) else {
            prop_assert!(longitudes.iter().any(|l| l + sierra > 64_000));
            return Ok(());
        };
        let util = parametros.longitud_util().unwrap().decimas_de_mm();
        for barra in &plan.barras {
            let usado: i64 = barra.piezas.iter().map(|p| p.longitud.decimas_de_mm() + sierra).sum();
            prop_assert!(usado <= util);
            prop_assert_eq!(usado + barra.sobrante.decimas_de_mm(), util);
        }
        let mut ids: Vec<u32> = plan.piezas().map(|p| p.id).collect();
        ids.sort_unstable();
        let esperados: Vec<u32> = lista.iter().map(|p| p.id).collect();
        prop_assert_eq!(ids, esperados);
        prop_assert!(plan.numero_de_barras() >= calcular_minimo_de_barras(&lista, &parametros).unwrap());
    }

    #[test]
    fn el_resultado_no_depende_del_orden_de_entrada(
        longitudes in prop::collection::vec(1i64..=64_000, 0..40),
        semilla in any::<u64>(),
    ) {
        let lista = piezas(&longitudes);
        let mut barajada = lista.clone();
        let total = barajada.len();
        for indice in 0..total {
            let otro = usize::try_from(semilla.rotate_left(u32::try_from(indice % 64).unwrap())).unwrap() % total;
            barajada.swap(indice, otro);
        }
        let a = optimizar_cortes(&lista, &parametros(640, 30));
        let b = optimizar_cortes(&barajada, &parametros(640, 30));
        prop_assert_eq!(a, b);
    }

    #[test]
    fn en_casos_pequenos_se_acerca_al_optimo(longitudes in prop::collection::vec(1_000i64..=64_000, 1..8)) {
        let plan = optimizar_cortes(&piezas(&longitudes), &parametros(640, 0)).unwrap();
        let optimo = optimo_exhaustivo(&longitudes, 64_000);
        prop_assert!(plan.numero_de_barras() >= optimo);
        prop_assert!(9 * plan.numero_de_barras() <= 11 * optimo + 6);
    }
}

fn retales(longitudes_dmm: &[i64]) -> Vec<RetalDisponible> {
    (0u32..)
        .zip(longitudes_dmm)
        .map(|(id, l)| RetalDisponible {
            id: 100 + id,
            longitud: Longitud::desde_decimas_de_mm(*l),
        })
        .collect()
}

#[test]
fn usa_el_retal_antes_de_abrir_una_barra_nueva() {
    let plan = optimizar_con_retales(
        &piezas(&[7_300, 7_300]),
        &retales(&[15_000]),
        &parametros(640, 0),
    )
    .unwrap();
    assert_eq!(plan.numero_de_barras(), 1);
    assert_eq!(plan.barras_nuevas(), 0);
    assert_eq!(plan.barras[0].retal, Some(100));
    assert_eq!(plan.barras[0].sobrante, Longitud::desde_decimas_de_mm(400));
}

#[test]
fn si_no_cabe_en_el_retal_abre_barra_y_los_retales_sin_usar_no_salen() {
    let plan = optimizar_con_retales(
        &piezas(&[12_000]),
        &retales(&[5_000, 3_000]),
        &parametros(640, 0),
    )
    .unwrap();
    assert_eq!(plan.barras_nuevas(), 1);
    assert_eq!(
        plan.numero_de_barras(),
        1,
        "los retales sin usar no aparecen"
    );
}

proptest! {
    #[test]
    fn con_retales_ninguna_pieza_supera_su_barra_ni_se_pierde(
        longitudes in prop::collection::vec(500i64..=60_000, 0..20),
        disponibles in prop::collection::vec(500i64..=30_000, 0..6),
    ) {
        let lista = piezas(&longitudes);
        let stock = retales(&disponibles);
        let params = parametros(640, 30);
        let plan = optimizar_con_retales(&lista, &stock, &params).unwrap();
        let mut colocadas: Vec<u32> = plan.piezas().map(|p| p.id).collect();
        colocadas.sort_unstable();
        let mut esperadas: Vec<u32> = lista.iter().map(|p| p.id).collect();
        esperadas.sort_unstable();
        prop_assert_eq!(colocadas, esperadas);
        for barra in &plan.barras {
            let capacidad = match barra.retal {
                Some(id) => stock.iter().find(|r| r.id == id).unwrap().longitud.decimas_de_mm(),
                None => params.longitud_util().unwrap().decimas_de_mm(),
            };
            let gastado: i64 = barra.piezas.iter().map(|p| params.consumo(p).unwrap().decimas_de_mm()).sum();
            prop_assert!(gastado + barra.sobrante.decimas_de_mm() == capacidad);
            prop_assert!(gastado <= capacidad);
        }
        let usados: Vec<u32> = plan.barras.iter().filter_map(|b| b.retal).collect();
        let mut sin_repetir = usados.clone();
        sin_repetir.dedup();
        prop_assert_eq!(usados.len(), sin_repetir.len(), "cada retal se usa una vez");
    }
}
