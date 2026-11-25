//! El dibujo del presupuesto (propietario, 2026-11-25): la ventana entera con su medida total,
//! sin números de pieza y sin la cota del marco, que quedan para el taller.

use super::pruebas::datos;
use super::*;
use crate::series::{MontajeDePersiana, TipoDibujo};
use crate::unidades::Longitud;

fn para_el_presupuesto(mut d: DatosDeDibujo) -> String {
    d.para_taller = false;
    dibujar_ventana(&d)
}

#[test]
fn el_presupuesto_va_sin_numeros_ni_flechas() {
    let svg = para_el_presupuesto(datos(TipoDibujo::Corredera, 2));
    assert!(!svg.contains("data-pieza"), "sin números de pieza");
    assert!(!svg.contains(r#"class="flecha""#), "sin flechas");
    assert!(
        svg.contains(r#"class="manilla""#),
        "los cierres sí, son parte de la ventana"
    );
    assert!(svg.contains(">150</text>") && svg.contains(">120</text>"));
}

#[test]
fn con_persiana_el_presupuesto_solo_acota_el_total() {
    let mut d = datos(TipoDibujo::Corredera, 2);
    d.persiana = Some(PersianaEnDibujo {
        montaje: MontajeDePersiana::Ventana,
        guia: Some(Longitud::desde_decimas_de_mm(400)),
        cajon: None,
        cinta: Lado::Derecha,
        ancho_ventana: Longitud::desde_decimas_de_mm(14_200),
        alto_ventana: None,
    });
    let presupuesto = para_el_presupuesto(d.clone());
    assert!(!presupuesto.contains(">marco "), "sin la cota del marco");
    assert_eq!(presupuesto.matches("texto-cota").count(), 2, "ancho y alto");
    let taller = dibujar_ventana(&d);
    assert!(taller.contains(">marco 142<"), "el taller sí la lleva");
}
