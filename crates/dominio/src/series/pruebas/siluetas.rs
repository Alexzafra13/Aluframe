//! `silueta` en las referencias y `posicion` en las piezas (ADR-0017).

use super::super::{calcular_ventana, importar_serie};
use super::*;

fn con_silueta(silueta: &str) -> Valor {
    m(vec![("referencia", t("R-1")), ("silueta", t(silueta))])
}

fn piezas_con_posicion(posicion: &str) -> Valor {
    let hoja = m(vec![
        ("perfil", t("HOJA")),
        ("cantidad", n("4")),
        ("longitud", t("alto - 5")),
        ("corte", n("90")),
        ("posicion", t(posicion)),
    ]);
    let marco = m(vec![
        ("perfil", t("MARCO")),
        ("cantidad", n("2")),
        ("longitud", t("alto")),
        ("corte", n("90")),
    ]);
    Valor::Lista(vec![marco, hoja])
}

#[test]
fn lee_la_silueta_de_una_referencia() {
    let mut valor = serie_base();
    cambiar(
        &mut valor,
        &["referencias", "MARCO"],
        Some(con_silueta("aluval/21210")),
    );
    let (serie, _) = importar_serie(&valor).unwrap();
    assert_eq!(
        serie.referencias["MARCO"].silueta.as_deref(),
        Some("aluval/21210")
    );
    assert_eq!(serie.referencias["HOJA"].silueta, None);
}

#[test]
fn rechaza_siluetas_mal_escritas() {
    for mala in [
        "21210",
        "Aluval/21210",
        "aluval/../x",
        "aluval/21 210",
        "/aluval/21210",
    ] {
        let errores = errores_con(&["referencias", "MARCO"], Some(con_silueta(mala)));
        assert!(
            errores.iter().any(|e| e.contains("fabricante/archivo")),
            "{mala}: {errores:?}"
        );
    }
}

#[test]
fn la_posicion_declarada_llega_al_resultado() {
    let mut valor = serie_base();
    cambiar(
        &mut valor,
        &["tipologias", "corredera", "piezas"],
        Some(piezas_con_posicion("hoja_central")),
    );
    let (serie, _) = importar_serie(&valor).unwrap();
    let resultado = calcular_ventana(&serie, "corredera", d("150"), d("120")).unwrap();
    assert_eq!(resultado.piezas[0].posicion, None);
    assert_eq!(
        resultado.piezas[1].posicion.as_deref(),
        Some("HOJA_CENTRAL")
    );
}

#[test]
fn rechaza_una_posicion_desconocida_y_dice_las_validas() {
    let errores = errores_con(
        &["tipologias", "corredera", "piezas"],
        Some(piezas_con_posicion("en_medio")),
    );
    assert!(
        errores.iter().any(|e| e.contains("hoja_central")),
        "{errores:?}"
    );
}
