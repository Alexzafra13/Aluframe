use proptest::prelude::*;

use super::*;
use crate::series::{Dibujo, TipoDibujo};
use crate::unidades::Longitud;

fn cm(valor: i64) -> Longitud {
    Longitud::desde_decimas_de_mm(valor * 100)
}

fn datos(tipo: TipoDibujo, hojas: u8) -> DatosDeDibujo {
    DatosDeDibujo {
        dibujo: Dibujo {
            tipo,
            hojas,
            hoja_movil: None,
            bisagras: None,
            cremona: None,
        },
        ancho: cm(150),
        alto: cm(120),
        piezas: vec![
            (1, "MARCO_LATERAL".into()),
            (2, "MARCO_SUPERIOR".into()),
            (3, "HOJA_CIERRE".into()),
            (4, "HOJA_CENTRAL".into()),
            (5, "HOJA_RODAMIENTO".into()),
        ],
        bisagras_por_hoja: 0,
        hoja_cremona: None,
        titulo: "Corredera 2 hojas <prueba>".into(),
    }
}

fn contar(svg: &str, trozo: &str) -> usize {
    svg.matches(trozo).count()
}

#[test]
fn dibuja_una_corredera_con_sus_piezas_numeradas_y_cotas() {
    let svg = dibujar_ventana(&datos(TipoDibujo::Corredera, 2));
    assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
    assert_eq!(contar(&svg, r#"class="hoja""#), 2);
    assert_eq!(contar(&svg, r#"class="cristal""#), 2);
    assert_eq!(contar(&svg, r#"class="flecha""#), 1);
    assert_eq!(contar(&svg, r#"class="manilla""#), 1);
    for numero in 1..=5 {
        assert!(
            svg.contains(&format!(r#"data-pieza="{numero}""#)),
            "falta la pieza {numero}"
        );
    }
    assert!(svg.contains(">150</text>") && svg.contains(">120</text>"));
    assert!(
        svg.contains("Corredera 2 hojas &lt;prueba&gt;"),
        "el título se escapa"
    );
}

#[test]
fn la_corredera_de_cuatro_hojas_mueve_las_centrales() {
    let svg = dibujar_ventana(&datos(TipoDibujo::Corredera, 4));
    assert_eq!(contar(&svg, r#"class="hoja""#), 4);
    assert_eq!(contar(&svg, r#"class="flecha""#), 2);
    assert_eq!(contar(&svg, r#"class="manilla""#), 3);
}

#[test]
fn la_abatible_lleva_sus_bisagras_y_la_cremona_en_la_hoja_elegida() {
    let mut abatible = datos(TipoDibujo::Abatible, 2);
    abatible.bisagras_por_hoja = 3;
    abatible.hoja_cremona = Some(Lado::Izquierda);
    let svg = dibujar_ventana(&abatible);
    assert_eq!(contar(&svg, r#"class="bisagra""#), 6);
    assert_eq!(contar(&svg, r#"class="apertura""#), 2);
    assert_eq!(contar(&svg, r#"class="manilla""#), 1);
    let marca = r#"class="manilla" x=""#;
    let despues = &svg[svg.find(marca).unwrap() + marca.len()..];
    let x: i64 = despues.split('"').next().unwrap().parse().unwrap();
    assert!(x < 7_500, "la cremona está en la hoja izquierda (x = {x})");
}

#[test]
fn la_fija_solo_tiene_marco_y_cristal() {
    let svg = dibujar_ventana(&datos(TipoDibujo::Fija, 1));
    assert_eq!(contar(&svg, r#"class="hoja""#), 0);
    assert_eq!(contar(&svg, r#"class="cristal""#), 1);
}

#[test]
fn los_numeros_no_se_pisan_y_los_desconocidos_no_se_pierden() {
    let mut d = datos(TipoDibujo::Corredera, 2);
    d.piezas.push((6, "TAPAJUNTAS".into()));
    d.piezas.push((7, "MARCO_LATERAL_2".into()));
    let plano = geometria::Plano::nuevo(&d);
    let colocados = marcadores::colocar(&plano, &d.piezas);
    assert_eq!(colocados.len(), 7);
    let r = marcadores::radio(&plano);
    for (i, a) in colocados.iter().enumerate() {
        for b in colocados.iter().skip(i + 1) {
            assert!(
                (a.1 - b.1).abs() >= 2 * r || (a.2 - b.2).abs() >= 2 * r,
                "{a:?} pisa {b:?}"
            );
        }
    }
}

proptest! {
    #[test]
    fn cualquier_medida_da_un_dibujo_completo(ancho in 1i64..2_000, alto in 1i64..2_000, hojas in 1u8..5) {
        for tipo in [TipoDibujo::Corredera, TipoDibujo::Abatible, TipoDibujo::Fija] {
            let mut d = datos(tipo, hojas);
            d.ancho = cm(ancho);
            d.alto = cm(alto);
            let svg = dibujar_ventana(&d);
            prop_assert!(svg.ends_with("</svg>"));
            prop_assert_eq!(contar(&svg, "data-pieza="), 5);
            prop_assert!(!svg.contains("NaN") && !svg.contains("--"));
        }
    }
}
