//! Parteluces (ADR-0030): opciones que son medidas, partes que solo existen si se cumple una
//! condición (`solo_si`), cristal partido y chapas.

use std::collections::BTreeMap;

use super::super::{calcular_ventana, calcular_ventana_con_opciones, importar_serie};
use super::*;
use crate::unidades::Longitud;

fn si_parteluces() -> Valor {
    m(vec![("parteluces", t("si"))])
}

fn opciones() -> Valor {
    Valor::Lista(vec![
        m(vec![
            ("codigo", t("parteluces")),
            ("nombre", t("Parteluces")),
            ("valores", Valor::Lista(vec![t("no"), t("si")])),
            ("por_defecto", t("no")),
        ]),
        m(vec![
            ("codigo", t("relleno_abajo")),
            ("nombre", t("Abajo")),
            ("valores", Valor::Lista(vec![t("chapa"), t("cristal")])),
            ("por_defecto", t("chapa")),
            ("solo_si", si_parteluces()),
        ]),
        m(vec![
            ("codigo", t("medida_abajo")),
            ("nombre", t("Medida de abajo")),
            ("tipo", t("medida")),
            ("por_defecto", n("50")),
            ("min", n("10")),
            ("max", n("200")),
            ("solo_si", si_parteluces()),
        ]),
    ])
}

fn vidrio() -> Valor {
    let cristal = |nombre: &str, alto: Valor, solo_si: Valor| {
        m(vec![
            ("nombre", t(nombre)),
            ("cantidad", n("2")),
            ("ancho", t("ancho / 2 - 7")),
            ("alto", alto),
            ("solo_si", solo_si),
        ])
    };
    let mut abajo = si_parteluces();
    cambiar(&mut abajo, &["relleno_abajo"], Some(t("cristal")));
    let piezas = vec![
        cristal("Entero", t("alto - 9"), m(vec![("parteluces", t("no"))])),
        cristal("Arriba", t(CONFIRMAR), si_parteluces()),
        cristal("Abajo", t("medida_abajo"), abajo),
    ];
    m(vec![
        ("tipo", t("CRISTAL")),
        ("piezas", Valor::Lista(piezas)),
    ])
}

/// Las piezas de la base más el parteluces, que solo existe con «Parteluces: Sí».
fn piezas_con_parteluces() -> Valor {
    let travesano = m(vec![
        ("perfil", t("PARTELUCES")),
        ("cantidad", n("2")),
        ("longitud", t(CONFIRMAR)),
        ("corte", n("90")),
        ("solo_si", si_parteluces()),
    ]);
    Valor::Lista(vec![
        pieza("MARCO", n("2"), "alto", n("90")),
        pieza("HOJA", n("4"), "ancho / 2 - descuento", n("90")),
        travesano,
    ])
}

fn serie_con_parteluces() -> Valor {
    let mut valor = serie_base();
    let ruta = |campo: &'static str| ["tipologias", "corredera", campo];
    cambiar(&mut valor, &ruta("opciones"), Some(opciones()));
    cambiar(&mut valor, &ruta("vidrio"), Some(vidrio()));
    let mut solo_chapa = si_parteluces();
    cambiar(&mut solo_chapa, &["relleno_abajo"], Some(t("chapa")));
    let chapa = m(vec![
        ("articulo", t("CHAPA")),
        ("nombre", t("Chapa de abajo")),
        ("cantidad", n("2")),
        ("ancho", t(CONFIRMAR)),
        ("alto", t("medida_abajo")),
        ("solo_si", solo_chapa),
    ]);
    cambiar(&mut valor, &ruta("chapas"), Some(Valor::Lista(vec![chapa])));
    let parteluces = m(vec![
        ("solo_si", si_parteluces()),
        ("abajo", t("medida_abajo")),
        ("altura", t(CONFIRMAR)),
    ]);
    cambiar(&mut valor, &ruta("parteluces"), Some(parteluces));
    cambiar(&mut valor, &ruta("piezas"), Some(piezas_con_parteluces()));
    for codigo in ["CHAPA", "PARTELUCES"] {
        cambiar(
            &mut valor,
            &["referencias", codigo],
            Some(m(vec![("referencia", t("R-9"))])),
        );
    }
    valor
}

fn elegir(pares: &[(&str, &str)]) -> BTreeMap<String, String> {
    pares
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn calcular(pares: &[(&str, &str)]) -> ResultadoVentana {
    let (serie, _) = importar_serie(&serie_con_parteluces()).unwrap();
    calcular_ventana_con_opciones(&serie, "corredera", (d("150"), d("120")), &elegir(pares))
        .unwrap()
}

#[test]
fn sin_parteluces_la_ventana_no_cambia() {
    let (serie, _) = importar_serie(&serie_con_parteluces()).unwrap();
    let (base, _) = importar_serie(&serie_base()).unwrap();
    let con = calcular_ventana(&serie, "corredera", d("150"), d("120")).unwrap();
    let sin = calcular_ventana(&base, "corredera", d("150"), d("120")).unwrap();
    assert_eq!(con.piezas, sin.piezas);
    assert!(con.chapas.is_empty() && con.parteluces.is_none());
    let vidrio = con.vidrio.unwrap();
    assert_eq!(vidrio.len(), 1);
    assert_eq!(vidrio[0].alto, Longitud::desde_decimas_de_mm(11_100));
}

#[test]
fn con_parteluces_lleva_su_perfil_la_chapa_y_queda_pendiente_lo_que_falta() {
    let r = calcular(&[("parteluces", "si")]);
    // Sin medida de corte no se puede cortar: no entra en la lista y queda como pendiente.
    assert!(!r.piezas.iter().any(|p| p.perfil == "PARTELUCES"));
    assert_eq!(r.chapas.len(), 1);
    let chapa = &r.chapas[0];
    assert_eq!((chapa.cantidad, chapa.ancho), (2, None));
    assert_eq!(chapa.alto, Some(Longitud::desde_decimas_de_mm(5_000)));
    let parteluces = r.parteluces.unwrap();
    assert_eq!(parteluces.abajo, Longitud::desde_decimas_de_mm(5_000));
    assert_eq!(parteluces.altura, None);
    assert!(r.es_incompleta());
    for falta in [
        "PARTELUCES",
        "Vidrio arriba",
        "Chapa de abajo",
        "Parteluces",
    ] {
        assert!(
            r.pendientes.iter().any(|p| p.contains(falta)),
            "falta «{falta}»: {:?}",
            r.pendientes
        );
    }
}

#[test]
fn con_cristal_abajo_no_hay_chapa_y_el_cristal_de_abajo_mide_lo_escrito() {
    let r = calcular(&[
        ("parteluces", "si"),
        ("relleno_abajo", "cristal"),
        ("medida_abajo", "62,5"),
    ]);
    assert!(r.chapas.is_empty());
    let vidrio = r.vidrio.unwrap();
    let abajo = vidrio
        .iter()
        .find(|v| v.nombre.as_deref() == Some("Abajo"))
        .unwrap();
    assert_eq!(abajo.alto, Longitud::desde_decimas_de_mm(6_250));
}

#[test]
fn rechaza_una_medida_de_abajo_fuera_de_su_rango() {
    let (serie, _) = importar_serie(&serie_con_parteluces()).unwrap();
    let pares = elegir(&[("parteluces", "si"), ("medida_abajo", "5")]);
    let error = calcular_ventana_con_opciones(&serie, "corredera", (d("150"), d("120")), &pares)
        .unwrap_err();
    assert!(error.to_string().contains("entre 10 y 200"), "{error}");
}

#[test]
fn rechaza_una_condicion_con_una_opcion_o_un_valor_que_no_existen() {
    let mut valor = serie_con_parteluces();
    let ruta = ["tipologias", "corredera", "parteluces", "solo_si"];
    cambiar(
        &mut valor,
        &ruta,
        Some(m(vec![("parteluces", t("quizas"))])),
    );
    let errores: Vec<String> = importar_serie(&valor)
        .unwrap_err()
        .errores
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        errores
            .iter()
            .any(|e| e.contains("«quizas» no es un valor")),
        "{errores:?}"
    );
}

#[test]
fn una_medida_desconocida_en_una_formula_es_un_error_de_importacion() {
    let mut valor = serie_con_parteluces();
    let ruta = ["tipologias", "abatible", "parteluces"];
    let parteluces = m(vec![("abajo", t("medida_abajo")), ("altura", t(CONFIRMAR))]);
    cambiar(&mut valor, &ruta, Some(parteluces));
    let errores = importar_serie(&valor).unwrap_err().errores;
    assert!(
        errores
            .iter()
            .any(|e| e.to_string().contains("medida_abajo")),
        "la medida solo vale en su tipología: {errores:?}"
    );
}

/// Una fórmula que solo falla con dos opciones a la vez (parteluces y Climalit) se detecta al
/// importar, no al presupuestar.
#[test]
fn detecta_al_importar_un_error_que_solo_sale_combinando_opciones() {
    let mut valor = serie_con_parteluces();
    let ruta = |campo: &'static str| ["tipologias", "corredera", campo];
    let Valor::Lista(mut lista) = opciones() else {
        panic!("las opciones son una lista")
    };
    lista.push(m(vec![
        ("codigo", t("acristalamiento")),
        ("nombre", t("Acristalamiento")),
        ("valores", Valor::Lista(vec![t("normal"), t("climalit")])),
        ("por_defecto", t("normal")),
    ]));
    cambiar(&mut valor, &ruta("opciones"), Some(Valor::Lista(lista)));
    let mut solo_si = si_parteluces();
    cambiar(&mut solo_si, &["acristalamiento"], Some(t("climalit")));
    let chapa = m(vec![
        ("articulo", t("CHAPA")),
        ("cantidad", n("4")),
        ("ancho", t("ancho / (alto - 100)")),
        ("alto", t("medida_abajo")),
        ("solo_si", solo_si),
    ]);
    cambiar(&mut valor, &ruta("chapas"), Some(Valor::Lista(vec![chapa])));
    let errores = importar_serie(&valor).unwrap_err().errores;
    assert!(
        errores
            .iter()
            .any(|e| e.to_string().contains("acristalamiento = climalit")),
        "{errores:?}"
    );
}
