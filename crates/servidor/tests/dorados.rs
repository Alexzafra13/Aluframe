#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use dominio::series::{ResultadoVentana, Serie, calcular_ventana, importar_serie};
use dominio::unidades::{Decimal, Longitud, Unidad};
use serde_json::Value;
use servidor::series::leer_yaml;

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn importar(ruta: &Path) -> Serie {
    let texto = std::fs::read_to_string(ruta).unwrap();
    let valor = leer_yaml(&texto).unwrap();
    match importar_serie(&valor) {
        Ok((serie, _)) => serie,
        Err(informe) => panic!("{} no se importa: {:#?}", ruta.display(), informe.errores),
    }
}

fn series_reales() -> BTreeMap<String, Serie> {
    std::fs::read_dir(raiz().join("data/series"))
        .unwrap()
        .map(|entrada| importar(&entrada.unwrap().path()))
        .map(|serie| (serie.codigo.clone(), serie))
        .collect()
}

fn decimal(valor: &Value) -> Decimal {
    Decimal::desde_texto(&valor.to_string()).unwrap()
}

fn cm(valor: &Value) -> Longitud {
    Longitud::desde_decimal(decimal(valor), Unidad::Cm).unwrap()
}

type PiezaComparable = (String, u32, Longitud, Option<u16>);

fn piezas_esperadas(esperado: &Value) -> Vec<PiezaComparable> {
    esperado["piezas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let corte = p["corte"].as_u64().map(|c| u16::try_from(c).unwrap());
            let cantidad = u32::try_from(p["cantidad"].as_u64().unwrap()).unwrap();
            (
                p["perfil"].as_str().unwrap().to_owned(),
                cantidad,
                cm(&p["longitud"]),
                corte,
            )
        })
        .collect()
}

fn accesorios_esperados(esperado: &Value) -> Vec<(String, Decimal)> {
    let mut accesorios: Vec<(String, Decimal)> = esperado["accesorios"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            (
                a["articulo"].as_str().unwrap().to_owned(),
                decimal(&a["cantidad"]),
            )
        })
        .collect();
    accesorios.sort();
    accesorios
}

fn comprobar(fixture: &Value, resultado: &ResultadoVentana) {
    let id = fixture["id"].as_str().unwrap();
    let esperado = &fixture["esperado"];
    let incompleto = esperado["incompleto"].as_bool().unwrap();
    assert_eq!(resultado.es_incompleta(), incompleto, "{id}: incompleto");
    let piezas: Vec<PiezaComparable> = resultado
        .piezas
        .iter()
        .map(|p| (p.perfil.clone(), p.cantidad, p.longitud, p.corte))
        .collect();
    assert_eq!(piezas, piezas_esperadas(esperado), "{id}: piezas");
    let mut accesorios = resultado.accesorios_sumados();
    accesorios.sort();
    assert_eq!(
        accesorios,
        accesorios_esperados(esperado),
        "{id}: accesorios"
    );
    assert_eq!(
        resultado.vidrio.is_none(),
        esperado["vidrio"].is_null(),
        "{id}: vidrio"
    );
}

#[test]
fn las_ventanas_reales_salen_como_en_sus_fixtures() {
    let series = series_reales();
    let mut comprobadas = 0;
    for entrada in std::fs::read_dir(raiz().join("fixtures/real")).unwrap() {
        let fixture: Value =
            serde_json::from_str(&std::fs::read_to_string(entrada.unwrap().path()).unwrap())
                .unwrap();
        let serie = &series[fixture["serie"].as_str().unwrap()];
        let entrada = &fixture["entrada"];
        let resultado = calcular_ventana(
            serie,
            fixture["tipologia"].as_str().unwrap(),
            decimal(&entrada["ancho"]),
            decimal(&entrada["alto"]),
        )
        .unwrap();
        comprobar(&fixture, &resultado);
        comprobadas += 1;
    }
    assert_eq!(comprobadas, 3);
}

#[test]
fn la_corredera_de_2_hojas_sale_incompleta_mientras_falte_el_cristal() {
    let series = series_reales();
    let cien_cincuenta = Decimal::desde_entero(150);
    let resultado = calcular_ventana(
        &series["ALUPROM_21"],
        "corredera_2_hojas",
        cien_cincuenta,
        Decimal::desde_entero(120),
    )
    .unwrap();
    assert!(resultado.es_incompleta());
    assert!(resultado.pendientes.iter().any(|p| p.contains("Vidrio")));
}

#[test]
fn la_serie_demo_se_importa_y_calcula_el_vidrio() {
    let demo = importar(&raiz().join("fixtures/demo/serie-demo.yaml"));
    let resultado = calcular_ventana(
        &demo,
        "corredera_2_hojas_demo",
        Decimal::desde_entero(150),
        Decimal::desde_entero(120),
    )
    .unwrap();
    let vidrio = resultado.vidrio.unwrap();
    assert_eq!(vidrio[0].cantidad, 2);
    assert_eq!(vidrio[0].ancho, Longitud::desde_decimas_de_mm(6_800));
    assert_eq!(vidrio[0].alto, Longitud::desde_decimas_de_mm(11_100));
    assert_eq!(
        resultado.pendientes,
        vec!["Serie de demostración con datos ficticios.".to_owned()]
    );
}
