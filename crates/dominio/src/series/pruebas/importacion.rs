use super::super::importar_serie;
use super::*;

#[test]
fn importa_una_serie_correcta_con_avisos_de_lo_pendiente() {
    let (serie, informe) = importar_serie(&serie_base()).unwrap();
    assert_eq!(serie.tipologias.len(), 2);
    assert_eq!(serie.parametros["descuento"], d("2"));
    let avisos: Vec<String> = informe.avisos.iter().map(ToString::to_string).collect();
    assert!(
        avisos.iter().any(|a| a.contains("abatible.vidrio")),
        "{avisos:?}"
    );
    assert!(
        avisos
            .iter()
            .any(|a| a.contains("abatible: no tiene límites")),
        "{avisos:?}"
    );
}

#[test]
fn informa_de_todos_los_errores_a_la_vez() {
    let mut valor = serie_base();
    cambiar(&mut valor, &["unidades"], Some(t("pulgadas")));
    cambiar(&mut valor, &["acesorios"], Some(t("errata")));
    cambiar(&mut valor, &["estado"], Some(t("final")));
    let errores = importar_serie(&valor).unwrap_err().errores;
    assert!(errores.len() >= 3, "{errores:?}");
}

#[test]
fn exige_los_campos_obligatorios_y_valida_su_formato() {
    assert!(errores_con(&["fabricante"], None)[0].contains("falta este campo"));
    assert!(errores_con(&["serie"], Some(t("minusculas")))[0].contains("MAYÚSCULAS"));
    assert!(errores_con(&["redondeo", "sentido"], Some(t("lateral")))[0].contains("abajo"));
    assert!(errores_con(&["redondeo", "precision"], Some(n("0")))[0].contains("positivo"));
    assert!(errores_con(&["longitud_barra"], Some(n("-5")))[0].contains("positiva"));
    assert!(errores_con(&["parametros", "alto"], Some(n("1")))[0].contains("nombre no válido"));
    assert!(errores_con(&["parametros"], Some(t("x")))[0].contains("bloque de campos"));
}

#[test]
fn rechaza_codigos_que_no_estan_en_referencias() {
    let errores = errores_con(&["referencias", "HOJA"], None);
    assert!(
        errores
            .iter()
            .any(|e| e.contains("«HOJA» no está en referencias")),
        "{errores:?}"
    );
}

#[test]
fn rechaza_formulas_con_nombres_desconocidos() {
    let mut valor = serie_base();
    cambiar(&mut valor, &["parametros", "descuento"], None);
    let errores: Vec<String> = importar_serie(&valor)
        .unwrap_err()
        .errores
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        errores
            .iter()
            .any(|e| e.contains("piezas[2].longitud") && e.contains("descuento")),
        "{errores:?}"
    );
}

#[test]
fn prueba_cada_tipologia_con_100_por_100() {
    let mut valor = serie_base();
    cambiar(&mut valor, &["parametros", "descuento"], Some(n("60")));
    let errores: Vec<String> = importar_serie(&valor)
        .unwrap_err()
        .errores
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        errores
            .iter()
            .any(|e| e.contains("ancho = 100") && e.contains("HOJA")),
        "{errores:?}"
    );
}

#[test]
fn comprueba_el_dibujo_y_las_opciones_de_la_abatible() {
    let errores = errores_con(
        &["tipologias", "abatible", "dibujo", "bisagras"],
        Some(t("NADA")),
    );
    assert!(
        errores.iter().any(|e| e.contains("bisagras «NADA»")),
        "{errores:?}"
    );
    let errores = errores_con(&["tipologias", "abatible", "dibujo", "cremona"], None);
    assert!(
        errores.iter().any(|e| e.contains("necesita «cremona»")),
        "{errores:?}"
    );
    let errores = errores_con(
        &["tipologias", "corredera", "dibujo", "hojas"],
        Some(n("3")),
    );
    assert!(
        errores.iter().any(|e| e.contains("2 o 4 hojas")),
        "{errores:?}"
    );
}
