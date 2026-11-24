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
        color: ColorDeDibujo::default(),
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

fn posiciones_de_manillas(svg: &str) -> Vec<i64> {
    let marca = r#"class="manilla" x=""#;
    svg.match_indices(marca)
        .map(|(i, _)| {
            let resto = &svg[i + marca.len()..];
            resto.split('"').next().unwrap().parse().unwrap()
        })
        .collect()
}

/// Como el catálogo de Aluval (propietario, 2026-11-22): el cierre va en el lateral de cada hoja
/// que se abre, nunca en el centro, y las flechas miran hacia el centro.
#[test]
fn la_corredera_de_dos_hojas_lleva_el_cierre_en_los_laterales() {
    let mut ambas = datos(TipoDibujo::Corredera, 2);
    ambas.dibujo.hoja_movil = Some("ambas".into());
    let svg = dibujar_ventana(&ambas);
    assert_eq!(contar(&svg, r#"class="flecha""#), 2);
    let x = posiciones_de_manillas(&svg);
    assert_eq!(x.len(), 2);
    assert!(
        x[0] < 2_000 && x[1] > 13_000,
        "cierres en los laterales: {x:?}"
    );

    let svg = dibujar_ventana(&datos(TipoDibujo::Corredera, 2));
    let x = posiciones_de_manillas(&svg);
    assert!(x.len() == 1 && x[0] > 13_000, "solo la hoja derecha: {x:?}");
}

/// Propietario (2026-11-23): se abren las cuatro hojas; un cierre en cada lateral y dos en el
/// centro, uno en cada hoja central.
#[test]
fn la_corredera_de_cuatro_hojas_lleva_cierres_en_los_laterales_y_dos_en_el_centro() {
    let svg = dibujar_ventana(&datos(TipoDibujo::Corredera, 4));
    assert_eq!(contar(&svg, r#"class="hoja""#), 4);
    assert_eq!(contar(&svg, r#"class="flecha""#), 4);
    let x = posiciones_de_manillas(&svg);
    assert_eq!(x.len(), 4, "{x:?}");
    let centro = 7_500;
    assert!(x[0] < 2_000 && x[3] > 13_000, "laterales: {x:?}");
    assert!(
        (x[1] - centro).abs() < 1_000 && (x[2] - centro).abs() < 1_000 && x[1] < x[2],
        "dos en el centro: {x:?}"
    );
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

/// El cierre 4 hojas (`posicion: centro`) lleva su número donde se juntan las hojas centrales.
#[test]
fn la_pieza_del_centro_se_numera_en_el_centro_de_la_ventana() {
    let mut d = datos(TipoDibujo::Corredera, 4);
    d.piezas = vec![(1, "CENTRO".into())];
    let plano = geometria::Plano::nuevo(&d);
    let colocados = marcadores::colocar(&plano, &d.piezas);
    let x = colocados[0].1;
    assert!(
        (x - plano.marco.x - plano.marco.ancho / 2).abs() < 10,
        "{x}"
    );
}

/// Un número con dos signos menos (`--5`) delata una cuenta mal hecha; las variables CSS
/// (`--perfil`) no cuentan.
fn tiene_doble_signo(svg: &str) -> bool {
    svg.as_bytes()
        .windows(3)
        .any(|w| w[0] == b'-' && w[1] == b'-' && w[2].is_ascii_digit())
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
            prop_assert!(!svg.contains("NaN") && !tiene_doble_signo(&svg));
        }
    }
}

#[test]
fn sin_acabado_la_ventana_es_blanca_con_cierres_blancos() {
    let svg = dibujar_ventana(&datos(TipoDibujo::Corredera, 2));
    assert!(svg.contains(r#"class="marco" "#) && svg.contains(r##"fill="#f2f2ed""##));
    assert!(
        svg.contains(r#"class="sombra""#),
        "el marco destaca del fondo"
    );
    assert!(svg.contains(r##"<g class="herrajes" fill="#f4f4f2">"##));
    assert!(
        !svg.contains("style="),
        "la CSP no deja aplicar estilos en línea"
    );
}

#[test]
fn con_un_color_se_pinta_de_ese_color_y_los_cierres_negros() {
    let mut verde = datos(TipoDibujo::Corredera, 2);
    verde.color = ColorDeDibujo::nuevo("#0F4336", false, Herrajes::Negro).unwrap();
    let svg = dibujar_ventana(&verde);
    assert!(svg.contains(r##"fill="#0f4336""##));
    assert!(svg.contains(r##"<g class="herrajes" fill="#15191d">"##));
    assert!(!svg.contains("url(#veta"));
}

#[test]
fn la_imitacion_madera_lleva_veta_en_marco_y_hojas() {
    let mut nogal = datos(TipoDibujo::Corredera, 2);
    nogal.color = ColorDeDibujo::nuevo("#5e3a23", true, Herrajes::Negro).unwrap();
    let svg = dibujar_ventana(&nogal);
    assert_eq!(contar(&svg, r#"fill="url(#veta-5e3a23bm-h)""#), 1, "marco");
    assert_eq!(contar(&svg, r#"fill="url(#veta-5e3a23bm-v)""#), 2, "hojas");
    assert!(
        svg.contains(r#"patternTransform="rotate(90)""#),
        "veta vertical en las hojas"
    );
}

#[test]
fn cada_superficie_tiene_su_luz_y_el_texturado_su_grano() {
    let pintar = |superficie| {
        let mut d = datos(TipoDibujo::Corredera, 2);
        d.color = ColorDeDibujo::nuevo("#373f43", false, Herrajes::Negro)
            .unwrap()
            .con_superficie(superficie);
        dibujar_ventana(&d)
    };
    let texturado = pintar(Superficie::Texturado);
    assert!(texturado.contains(r#"<pattern id="textura-373f43t""#));
    assert_eq!(contar(&texturado, r#"class="textura""#), 3);
    let anodizado = pintar(Superficie::Anodizado);
    assert!(anodizado.contains(r#"<linearGradient id="luz-373f43a""#));
    assert!(!anodizado.contains("textura"));
    assert_ne!(pintar(Superficie::Brillo), pintar(Superficie::Mate));
}
