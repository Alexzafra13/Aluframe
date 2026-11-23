use proptest::prelude::*;

use super::pruebas::optimo_exhaustivo;
use super::*;
use crate::unidades::Longitud;

fn cm(valor: i64) -> Longitud {
    Longitud::desde_decimas_de_mm(valor * 100)
}

fn parametros(sierra_dmm: i64) -> ParametrosDeCorte {
    ParametrosDeCorte {
        longitud_barra: cm(640),
        grosor_sierra: Longitud::desde_decimas_de_mm(sierra_dmm),
        recorte_extremos: Longitud::CERO,
        retal_minimo: Longitud::CERO,
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

fn retales(longitudes_dmm: &[i64]) -> Vec<RetalDisponible> {
    (0u32..)
        .zip(longitudes_dmm)
        .map(|(id, l)| RetalDisponible {
            id: 100 + id,
            longitud: Longitud::desde_decimas_de_mm(*l),
        })
        .collect()
}

fn con(
    metodo: MetodoDeCorte,
    (lista, stock): (&[PiezaACortar], &[RetalDisponible]),
    params: &ParametrosDeCorte,
    valoracion: &ValoracionDeSobrantes,
) -> PlanDeCorte {
    planificar_cortes((lista, stock), params, (metodo, valoracion)).unwrap()
}

/// Lo que queda tras el plan, con los retales del stock que no usa.
fn valor(
    plan: &PlanDeCorte,
    stock: &[RetalDisponible],
    params: &ParametrosDeCorte,
    v: &ValoracionDeSobrantes,
) -> i128 {
    let usados: Vec<u32> = plan.barras.iter().filter_map(|b| b.retal).collect();
    let mut sobrantes: Vec<Longitud> = plan.barras.iter().map(|b| b.sobrante).collect();
    sobrantes.extend(
        stock
            .iter()
            .filter(|r| !usados.contains(&r.id))
            .map(|r| r.longitud),
    );
    v.valorar(&sobrantes, params.grosor_sierra)
}

#[test]
fn ahorra_la_barra_que_el_metodo_rapido_gasta_de_mas() {
    let lista = piezas(&[28_900, 26_500, 21_200, 17_400, 13_200, 10_000, 10_000]);
    let v = ValoracionDeSobrantes::default();
    let rapido = con(MetodoDeCorte::Rapido, (&lista, &[]), &parametros(0), &v);
    let mejor = con(
        MetodoDeCorte::MejorAprovechamiento,
        (&lista, &[]),
        &parametros(0),
        &v,
    );
    assert_eq!(rapido.numero_de_barras(), 3);
    assert_eq!(mejor.numero_de_barras(), 2, "289+212+132 y 265+174+100+100");
    assert_eq!(mejor.piezas().count(), 7);
}

#[test]
fn con_las_mismas_barras_deja_el_sobrante_que_sirve_para_lo_habitual() {
    // Dos barras de 640 en cualquier caso; cambia cómo se reparte lo que sobra.
    let lista = piezas(&[33_000, 22_500, 18_000, 15_000]);
    let sobras = |v: &ValoracionDeSobrantes| {
        let plan = con(
            MetodoDeCorte::MejorAprovechamiento,
            (&lista, &[]),
            &parametros(0),
            v,
        );
        assert_eq!(plan.numero_de_barras(), 2);
        let mut s: Vec<Longitud> = plan.barras.iter().map(|b| b.sobrante).collect();
        s.sort_unstable();
        s
    };
    assert_eq!(
        sobras(&ValoracionDeSobrantes::default()),
        vec![cm(85), cm(310)],
        "sin historial, lo más junto (330 + 225 y 180 + 150)"
    );
    let de_45 = ValoracionDeSobrantes {
        minimo_util: Longitud::CERO,
        habituales: vec![(cm(45), 5)],
    };
    assert_eq!(
        sobras(&de_45),
        vec![cm(160), cm(235)],
        "si lo habitual son piezas de 45, de 160 y 235 salen 8 y de 85 y 310 solo 7"
    );
}

#[test]
fn el_metodo_rapido_es_best_fit_decreasing_tal_cual() {
    let lista = piezas(&[40_000, 30_000, 23_000, 20_000]);
    let v = ValoracionDeSobrantes::default();
    assert_eq!(
        con(MetodoDeCorte::Rapido, (&lista, &[]), &parametros(0), &v),
        optimizar_cortes(&lista, &parametros(0)).unwrap()
    );
    assert_eq!(
        MetodoDeCorte::desde_texto("rapido"),
        Some(MetodoDeCorte::Rapido)
    );
    assert_eq!(
        MetodoDeCorte::MejorAprovechamiento.como_texto(),
        "aprovechamiento"
    );
    assert_eq!(MetodoDeCorte::desde_texto("otro"), None);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    #[test]
    fn nunca_sale_peor_que_el_metodo_rapido_ni_pierde_piezas(
        longitudes in prop::collection::vec(500i64..=60_000, 0..18),
        disponibles in prop::collection::vec(500i64..=30_000, 0..5),
        habitual in prop::option::of(5_000i64..=30_000),
        sierra in 0i64..=40,
    ) {
        let lista = piezas(&longitudes);
        let stock = retales(&disponibles);
        let params = parametros(sierra);
        let v = ValoracionDeSobrantes {
            minimo_util: Longitud::CERO,
            habituales: habitual.map(|h| vec![(Longitud::desde_decimas_de_mm(h), 3)]).unwrap_or_default(),
        };
        let rapido = con(MetodoDeCorte::Rapido, (&lista, &stock), &params, &v);
        let mejor = con(MetodoDeCorte::MejorAprovechamiento, (&lista, &stock), &params, &v);
        prop_assert!(mejor.barras_nuevas() <= rapido.barras_nuevas());
        if mejor.barras_nuevas() == rapido.barras_nuevas() {
            prop_assert!(valor(&mejor, &stock, &params, &v) >= valor(&rapido, &stock, &params, &v));
        }
        let mut colocadas: Vec<u32> = mejor.piezas().map(|p| p.id).collect();
        colocadas.sort_unstable();
        prop_assert_eq!(colocadas, lista.iter().map(|p| p.id).collect::<Vec<_>>());
        for barra in &mejor.barras {
            let capacidad = match barra.retal {
                Some(id) => stock.iter().find(|r| r.id == id).unwrap().longitud.decimas_de_mm(),
                None => params.longitud_util().unwrap().decimas_de_mm(),
            };
            let gastado: i64 = barra.piezas.iter().map(|p| params.consumo(p).unwrap().decimas_de_mm()).sum();
            prop_assert_eq!(gastado + barra.sobrante.decimas_de_mm(), capacidad);
            prop_assert!(!barra.piezas.is_empty());
        }
        let mut usados: Vec<u32> = mejor.barras.iter().filter_map(|b| b.retal).collect();
        let total = usados.len();
        usados.sort_unstable();
        usados.dedup();
        prop_assert_eq!(usados.len(), total, "cada retal se usa una vez");
    }

    #[test]
    fn en_casos_pequenos_da_el_optimo(longitudes in prop::collection::vec(1_000i64..=64_000, 1..9)) {
        let lista = piezas(&longitudes);
        let v = ValoracionDeSobrantes::default();
        let plan = con(MetodoDeCorte::MejorAprovechamiento, (&lista, &[]), &parametros(0), &v);
        prop_assert_eq!(plan.numero_de_barras(), optimo_exhaustivo(&longitudes, 64_000));
    }

    #[test]
    fn el_resultado_no_depende_del_orden_de_entrada(
        longitudes in prop::collection::vec(1i64..=64_000, 0..16),
        semilla in any::<u64>(),
    ) {
        let lista = piezas(&longitudes);
        let mut barajada = lista.clone();
        let total = barajada.len();
        for indice in 0..total {
            let otro = usize::try_from(semilla.rotate_left(u32::try_from(indice % 64).unwrap())).unwrap() % total;
            barajada.swap(indice, otro);
        }
        let v = ValoracionDeSobrantes::default();
        let metodo = (MetodoDeCorte::MejorAprovechamiento, &v);
        let a = planificar_cortes((&lista, &[]), &parametros(30), metodo);
        let b = planificar_cortes((&barajada, &[]), &parametros(30), metodo);
        prop_assert_eq!(a, b);
    }
}
