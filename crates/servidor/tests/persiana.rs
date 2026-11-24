#![allow(clippy::unwrap_used, clippy::panic)]

//! Corredera de Aluprom 21 con persiana en la ventana (ADR-0032), con la serie real: la medida
//! que se escribe es la de todo el conjunto y el marco sale de quitarle guías y cajón.

use std::collections::BTreeMap;
use std::path::Path;

use dominio::dibujo::{DatosDeDibujo, dibujar_ventana};
use dominio::series::{ResultadoVentana, Serie, calcular_ventana_con_opciones, importar_serie};
use dominio::unidades::{Decimal, Longitud};
use servidor::series::leer_yaml;

fn serie() -> Serie {
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/series/aluprom-21.yaml");
    let valor = leer_yaml(&std::fs::read_to_string(ruta).unwrap()).unwrap();
    importar_serie(&valor).unwrap().0
}

fn calcular(tipologia: &str, pares: &[(&str, &str)]) -> ResultadoVentana {
    let opciones: BTreeMap<String, String> = pares
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let medida = Decimal::desde_entero(120);
    calcular_ventana_con_opciones(&serie(), tipologia, (medida, medida), &opciones).unwrap()
}

fn longitud_de(r: &ResultadoVentana, perfil: &str) -> Option<Longitud> {
    r.piezas
        .iter()
        .find(|p| p.perfil == perfil)
        .map(|p| p.longitud)
}

fn cm(decimas_de_mm: i64) -> Option<Longitud> {
    Some(Longitud::desde_decimas_de_mm(decimas_de_mm))
}

#[test]
fn sin_persiana_la_medida_es_la_del_marco() {
    let r = calcular("corredera_2_hojas", &[]);
    assert!(r.persiana.is_none());
    assert_eq!(longitud_de(&r, "MARCO_SUPERIOR"), cm(11_590), "120 - 4,1");
    assert_eq!(longitud_de(&r, "MARCO_LATERAL"), cm(12_000));
}

/// 120 de conjunto con persiana: 4 cm de guía a cada lado, el marco es de 112 de ancho.
#[test]
fn con_persiana_el_ancho_del_marco_pierde_las_dos_guias() {
    let r = calcular("corredera_2_hojas", &[("persiana", "si")]);
    let persiana = r.persiana.as_ref().unwrap();
    assert_eq!(
        persiana.ancho_ventana,
        Longitud::desde_decimas_de_mm(11_200)
    );
    assert_eq!(longitud_de(&r, "MARCO_SUPERIOR"), cm(10_790), "112 - 4,1");
    assert_eq!(longitud_de(&r, "MARCO_INFERIOR"), cm(10_790));
    assert_eq!(longitud_de(&r, "HOJA_RODAMIENTO"), cm(5_400), "112 / 2 - 2");
    assert_eq!(
        r.ancho,
        Decimal::desde_entero(120),
        "la medida escrita no cambia"
    );
}

/// El alto del cajón lo dará el propietario: hasta entonces lo que depende del alto queda
/// pendiente, la ventana sale incompleta y nada se inventa.
#[test]
fn sin_el_alto_del_cajon_lo_que_depende_del_alto_queda_pendiente() {
    let r = calcular("corredera_2_hojas", &[("persiana", "si")]);
    assert_eq!(r.persiana.as_ref().unwrap().alto_ventana, None);
    let pendientes: Vec<&str> = r
        .piezas_pendientes
        .iter()
        .map(|p| p.perfil.as_str())
        .collect();
    assert_eq!(pendientes, ["MARCO_LATERAL", "HOJA_LATERAL", "HOJA_CIERRE"]);
    let numeros: Vec<u32> = r.piezas_pendientes.iter().map(|p| p.numero).collect();
    assert_eq!(numeros, [1, 4, 5], "conservan su número en la tipología");
    assert!(
        r.vidrio.as_ref().unwrap().is_empty(),
        "el cristal espera al alto"
    );
    assert!(r.es_incompleta());
    assert!(
        r.pendientes.iter().any(|p| p.contains("alto del cajón")),
        "{:?}",
        r.pendientes
    );
    assert!(r.accesorios.iter().any(|a| a.articulo == "PERSIANA"));
}

#[test]
fn la_cuatro_hojas_tambien_admite_persiana() {
    let r = calcular("corredera_4_hojas", &[("persiana", "si")]);
    assert_eq!(
        longitud_de(&r, "HOJA_RODAMIENTO"),
        cm(2_720),
        "(112 - 3) / 4 = 27,25, a 0,1 hacia abajo"
    );
    assert!(
        r.piezas_pendientes
            .iter()
            .any(|p| p.perfil == "CIERRE_4_HOJAS")
    );
}

fn dibujo(pares: &[(&str, &str)]) -> String {
    let serie = serie();
    let r = calcular("corredera_2_hojas", pares);
    let tipologia = serie.tipologia("corredera_2_hojas").unwrap();
    let opciones = pares
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let medida = Longitud::desde_decimas_de_mm(12_000);
    dibujar_ventana(&DatosDeDibujo::desde_calculo(
        tipologia,
        &r,
        &opciones,
        (medida, medida),
    ))
}

#[test]
fn el_dibujo_lleva_cajon_guias_lamas_y_cinta_y_acota_el_marco() {
    let svg = dibujo(&[("persiana", "si")]);
    for parte in [
        "persiana-cajon",
        "persiana-guia",
        "persiana-lamas",
        "persiana-cinta",
    ] {
        assert!(svg.contains(parte), "falta {parte}");
    }
    assert!(svg.contains(">120<"), "la medida del conjunto");
    assert!(svg.contains(">marco 112<"), "la del marco");
    assert!(!dibujo(&[]).contains("persiana-"), "sin persiana, nada");
}

#[test]
fn la_cinta_va_a_la_derecha_salvo_que_se_elija_la_izquierda() {
    let x_de_la_cinta = |svg: &str| -> i64 {
        let resto = &svg[svg.find("persiana-cinta").unwrap()..];
        let x = &resto[resto.find("x=\"").unwrap() + 3..];
        x[..x.find('"').unwrap()].parse().unwrap()
    };
    let derecha = x_de_la_cinta(&dibujo(&[("persiana", "si")]));
    let izquierda = x_de_la_cinta(&dibujo(&[("persiana", "si"), ("cinta", "izquierda")]));
    assert!(
        derecha > 12_000,
        "fuera del conjunto, a la derecha: {derecha}"
    );
    assert!(
        izquierda < 0,
        "fuera del conjunto, a la izquierda: {izquierda}"
    );
}
