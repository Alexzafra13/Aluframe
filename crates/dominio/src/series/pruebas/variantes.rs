//! `perfil_segun`: el perfil de una pieza cambia con una opción de la línea (ADR-0018).

use std::collections::BTreeMap;

use super::super::{calcular_ventana, calcular_ventana_con_opciones, importar_serie};
use super::*;

fn opcion_acristalamiento() -> Valor {
    Valor::Lista(vec![m(vec![
        ("codigo", t("acristalamiento")),
        ("nombre", t("Acristalamiento")),
        ("valores", Valor::Lista(vec![t("sencillo"), t("climalit")])),
        ("por_defecto", t("sencillo")),
    ])])
}

fn hoja_segun(opcion: &str, valor: &str, perfil: &str) -> Valor {
    m(vec![
        ("perfil", t("HOJA")),
        (
            "perfil_segun",
            m(vec![(opcion, m(vec![(valor, t(perfil))]))]),
        ),
        ("cantidad", n("4")),
        ("longitud", t("alto - 5")),
        ("corte", n("90")),
    ])
}

fn serie_con(hoja: Valor, con_referencia: bool) -> Valor {
    let mut valor = serie_base();
    let marco = m(vec![
        ("perfil", t("MARCO")),
        ("cantidad", n("2")),
        ("longitud", t("alto")),
        ("corte", n("90")),
    ]);
    let ruta = ["tipologias", "corredera"];
    cambiar(
        &mut valor,
        &[ruta[0], ruta[1], "piezas"],
        Some(Valor::Lista(vec![marco, hoja])),
    );
    cambiar(
        &mut valor,
        &[ruta[0], ruta[1], "opciones"],
        Some(opcion_acristalamiento()),
    );
    if con_referencia {
        cambiar(
            &mut valor,
            &["referencias", "HOJA_CLIMALIT"],
            Some(m(vec![("referencia", t("R-3"))])),
        );
    }
    valor
}

fn errores(valor: &Valor) -> Vec<String> {
    importar_serie(valor)
        .unwrap_err()
        .errores
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn sin_elegir_se_usa_el_perfil_base_y_con_climalit_su_variante() {
    let valor = serie_con(
        hoja_segun("acristalamiento", "climalit", "HOJA_CLIMALIT"),
        true,
    );
    let (serie, _) = importar_serie(&valor).unwrap();
    let base = calcular_ventana(&serie, "corredera", d("150"), d("120")).unwrap();
    assert_eq!(base.piezas[1].perfil, "HOJA");
    let opciones = BTreeMap::from([("acristalamiento".to_owned(), "climalit".to_owned())]);
    let climalit =
        calcular_ventana_con_opciones(&serie, "corredera", (d("150"), d("120")), &opciones)
            .unwrap();
    assert_eq!(climalit.piezas[1].perfil, "HOJA_CLIMALIT");
    assert_eq!(climalit.piezas[1].longitud, base.piezas[1].longitud);
    assert_eq!(climalit.piezas[1].numero, 2, "la numeración no cambia");
}

#[test]
fn un_valor_sin_variante_se_queda_con_el_perfil_base() {
    let valor = serie_con(
        hoja_segun("acristalamiento", "climalit", "HOJA_CLIMALIT"),
        true,
    );
    let (serie, _) = importar_serie(&valor).unwrap();
    let opciones = BTreeMap::from([("acristalamiento".to_owned(), "sencillo".to_owned())]);
    let resultado =
        calcular_ventana_con_opciones(&serie, "corredera", (d("150"), d("120")), &opciones)
            .unwrap();
    assert_eq!(resultado.piezas[1].perfil, "HOJA");
}

#[test]
fn la_variante_tiene_que_estar_en_referencias() {
    let valor = serie_con(
        hoja_segun("acristalamiento", "climalit", "HOJA_CLIMALIT"),
        false,
    );
    assert!(
        errores(&valor)
            .iter()
            .any(|e| e.contains("HOJA_CLIMALIT") && e.contains("referencias")),
        "{:?}",
        errores(&valor)
    );
}

#[test]
fn rechaza_opciones_o_valores_que_la_tipologia_no_tiene() {
    let opcion_mala = serie_con(hoja_segun("vidrio", "climalit", "HOJA_CLIMALIT"), true);
    assert!(
        errores(&opcion_mala)
            .iter()
            .any(|e| e.contains("«vidrio» no es una opción")),
        "{:?}",
        errores(&opcion_mala)
    );
    let valor_malo = serie_con(
        hoja_segun("acristalamiento", "triple", "HOJA_CLIMALIT"),
        true,
    );
    assert!(
        errores(&valor_malo)
            .iter()
            .any(|e| e.contains("«triple» no es un valor")),
        "{:?}",
        errores(&valor_malo)
    );
}
