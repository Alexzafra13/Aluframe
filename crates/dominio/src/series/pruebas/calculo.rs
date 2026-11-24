use super::super::{ErrorCalculo, calcular_ventana, importar_serie};
use super::*;
use crate::unidades::Longitud;

#[test]
fn calcula_la_corredera_completa_con_vidrio_y_accesorios_sumados() {
    let (serie, _) = importar_serie(&serie_base()).unwrap();
    let resultado = calcular_ventana(&serie, "corredera", d("150"), d("120")).unwrap();
    assert!(!resultado.es_incompleta(), "{:?}", resultado.pendientes);
    assert_eq!(
        resultado.piezas[1].longitud,
        Longitud::desde_decimas_de_mm(7_300)
    );
    assert_eq!(resultado.piezas[1].formula, "ancho / 2 - descuento");
    let vidrio = resultado.vidrio.as_ref().unwrap();
    assert_eq!(
        (vidrio[0].ancho, vidrio[0].alto),
        (
            Longitud::desde_decimas_de_mm(6_800),
            Longitud::desde_decimas_de_mm(11_100)
        )
    );
    assert_eq!(
        resultado.accesorios_sumados(),
        vec![("TORNILLO".into(), d("17")), ("JUNTA".into(), d("5.4"))]
    );
    let grupos = resultado.piezas_a_cortar();
    assert_eq!(grupos[1].0, "HOJA");
    assert_eq!(
        grupos[1].1.iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![2001, 2002, 2003, 2004]
    );
}

#[test]
fn deja_fuera_lo_pendiente_y_lo_explica() {
    let (serie, _) = importar_serie(&serie_base()).unwrap();
    let resultado = calcular_ventana(&serie, "abatible", d("120"), d("140")).unwrap();
    let perfiles: Vec<&str> = resultado.piezas.iter().map(|p| p.perfil.as_str()).collect();
    assert_eq!(perfiles, vec!["MARCO", "HOJA"]);
    assert_eq!(resultado.piezas[1].corte, None);
    assert_eq!(
        resultado.accesorios_sumados(),
        vec![("BISAGRA".into(), d("4"))]
    );
    let pendientes = resultado.pendientes.join(" | ");
    for texto in [
        "Inversor: falta la cantidad",
        "Hoja: falta el ángulo",
        "Vidrio",
        "PLETINA",
        "referencia del proveedor (y el precio) de: INVERSOR",
    ] {
        assert!(
            pendientes.contains(texto),
            "{texto} no está en {pendientes}"
        );
    }
}

#[test]
fn avisa_de_medidas_fuera_de_limites_sin_bloquear() {
    let (serie, _) = importar_serie(&serie_base()).unwrap();
    let resultado = calcular_ventana(&serie, "corredera", d("320"), d("30")).unwrap();
    assert_eq!(
        resultado.fuera_de_limites.len(),
        2,
        "{:?}",
        resultado.fuera_de_limites
    );
}

#[test]
fn rechaza_medidas_imposibles_y_tipologias_desconocidas() {
    let (serie, _) = importar_serie(&serie_base()).unwrap();
    assert_eq!(
        calcular_ventana(&serie, "corredera", d("0"), d("120")),
        Err(ErrorCalculo::MedidaNoPositiva("ancho"))
    );
    assert!(matches!(
        calcular_ventana(&serie, "puerta", d("1"), d("1")),
        Err(ErrorCalculo::TipologiaDesconocida(_))
    ));
    assert!(matches!(
        calcular_ventana(&serie, "corredera", d("10"), d("120")),
        Err(ErrorCalculo::MedidaNoValida { .. })
    ));
}
