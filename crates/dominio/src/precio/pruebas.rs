use std::collections::BTreeMap;

use proptest::prelude::*;

use super::*;
use crate::series::{
    AccesorioCalculado, AplicaA, ResultadoVentana, UnidadCantidad, VidrioCalculado,
};
use crate::unidades::Decimal;

fn pu(texto: &str) -> PrecioUnitario {
    PrecioUnitario::desde_texto_es(texto).unwrap()
}

fn euros(centimos: i64) -> Dinero {
    Dinero::desde_centimos(centimos)
}

pub(super) fn bp(valor: i64) -> Porcentaje {
    Porcentaje::desde_puntos_basicos(valor)
}

fn articulo(referencia: &str, unidad: UnidadDeVenta, precio: &str) -> ArticuloConPrecio {
    ArticuloConPrecio {
        referencia: referencia.into(),
        unidad,
        precio: pu(precio),
        descuento_compra: Porcentaje::CERO,
        longitud_barra: (unidad == UnidadDeVenta::Barra)
            .then(|| Longitud::desde_decimas_de_mm(60_000)),
        unidades_por_caja: (unidad == UnidadDeVenta::Caja).then_some(100),
        con_acabado: false,
    }
}

pub(super) fn catalogo() -> BTreeMap<String, ArticuloConPrecio> {
    let mut perfil = articulo("P", UnidadDeVenta::Barra, "20");
    perfil.con_acabado = true;
    let mut kit = articulo("K", UnidadDeVenta::Unidad, "9");
    kit.descuento_compra = bp(1_000);
    [
        ("PERFIL", perfil),
        ("VIDRIO", articulo("V", UnidadDeVenta::MetroCuadrado, "15")),
        ("KIT", kit),
        ("TORNILLO", articulo("T", UnidadDeVenta::Caja, "8")),
    ]
    .into_iter()
    .map(|(c, a)| (c.to_owned(), a))
    .collect()
}

fn accesorio(codigo: &str, cantidad: i64) -> AccesorioCalculado {
    AccesorioCalculado {
        articulo: codigo.into(),
        cantidad: Decimal::desde_entero(cantidad),
        aplica_a: AplicaA::Ventana,
        unidad_cantidad: UnidadCantidad::Unidad,
    }
}

pub(super) fn resultado() -> ResultadoVentana {
    ResultadoVentana {
        serie: "S".into(),
        tipologia: "T".into(),
        ancho: Decimal::desde_entero(150),
        alto: Decimal::desde_entero(120),
        piezas: Vec::new(),
        accesorios: vec![accesorio("KIT", 1), accesorio("TORNILLO", 17)],
        vidrio: Some(vec![VidrioCalculado {
            tipo: "VIDRIO".into(),
            nombre: None,
            cantidad: 2,
            ancho: Longitud::desde_decimas_de_mm(6_800),
            alto: Longitud::desde_decimas_de_mm(11_100),
        }]),
        chapas: Vec::new(),
        parteluces: None,
        pendientes: Vec::new(),
        fuera_de_limites: Vec::new(),
    }
}

pub(super) fn perfiles() -> BTreeMap<String, PerfilUsado> {
    let usado = PerfilUsado {
        barras: 2,
        longitud_barra: Longitud::desde_decimas_de_mm(60_000),
        cortado: Longitud::desde_decimas_de_mm(80_000),
    };
    BTreeMap::from([("PERFIL".to_owned(), usado)])
}

#[test]
fn calcula_el_coste_de_los_materiales() {
    let coste = calcular_coste_de_materiales(
        &resultado(),
        &perfiles(),
        &catalogo(),
        (Some(ModoPerfiles::BarrasEnteras), bp(1_500)),
    );
    assert!(coste.faltan.is_empty(), "{:?}", coste.faltan);
    assert_eq!(
        coste.perfiles,
        euros(4_600),
        "2 barras × 20 € + 15 % de color"
    );
    assert_eq!(
        coste.perfiles_por_metros,
        Some(euros(3_066)),
        "800 cm de 600 cm = 1,333 barras (en milésimas) × 20 € + 15 %"
    );
    assert_eq!(coste.cristal, euros(2_265), "1,51 m² × 15 €");
    assert_eq!(
        coste.accesorios,
        euros(810 + 136),
        "kit con 10 % de descuento y 17 tornillos de una caja de 100"
    );
}

#[test]
fn los_perfiles_por_metros_suman_la_merma() {
    let coste = calcular_coste_de_materiales(
        &resultado(),
        &perfiles(),
        &catalogo(),
        (
            Some(ModoPerfiles::MetrosConMerma(bp(1_000))),
            Porcentaje::CERO,
        ),
    );
    assert_eq!(
        coste.perfiles,
        euros(2_934),
        "880 cm de 600 cm = 1,467 barras × 20 €"
    );
    assert_eq!(coste.perfiles_por_barras, Some(euros(4_000)));
}

#[test]
fn aplica_margen_mano_de_obra_cantidad_y_descuento() {
    let coste = calcular_coste_de_materiales(
        &resultado(),
        &perfiles(),
        &catalogo(),
        (Some(ModoPerfiles::BarrasEnteras), bp(1_500)),
    );
    let linea = DatosDeLinea {
        cantidad: 2,
        recargo_acabado: bp(1_500),
        mano_de_obra: euros(2_500),
        descuento: bp(500),
    };
    let sobre_coste = ReglasDePrecio {
        modo_perfiles: None,
        margen: Some((ModoMargen::SobreCoste, bp(3_000))),
    };
    let precio = calcular_precio_de_linea(&coste, &sobre_coste, &linea);
    assert!(precio.es_final());
    assert_eq!(precio.coste_unitario, euros(7_811));
    assert_eq!(precio.venta_unitaria, euros(10_154 + 2_500));
    assert_eq!(
        (precio.precio, precio.descuento),
        (euros(24_043), euros(1_265))
    );
    let sobre_venta = ReglasDePrecio {
        modo_perfiles: None,
        margen: Some((ModoMargen::SobreVenta, bp(3_000))),
    };
    assert_eq!(
        calcular_precio_de_linea(&coste, &sobre_venta, &linea).venta_unitaria,
        euros(11_159 + 2_500)
    );
}

#[test]
fn sin_reglas_ni_precios_no_hay_precio_final_y_dice_por_que() {
    let mut r = resultado();
    r.vidrio = None;
    r.pendientes.push("vidrio: CONFIRMAR".into());
    let mut sin_kit = catalogo();
    sin_kit.remove("KIT");
    let coste = calcular_coste_de_materiales(&r, &perfiles(), &sin_kit, (None, Porcentaje::CERO));
    let linea = DatosDeLinea {
        cantidad: 1,
        recargo_acabado: Porcentaje::CERO,
        mano_de_obra: Dinero::CERO,
        descuento: Porcentaje::CERO,
    };
    let precio = calcular_precio_de_linea(
        &coste,
        &ReglasDePrecio {
            modo_perfiles: None,
            margen: None,
        },
        &linea,
    );
    assert!(!precio.es_final());
    for motivo in [
        "vidrio: CONFIRMAR",
        "elige cómo se cobran los perfiles",
        "falta el cristal",
        "falta el precio de KIT",
        "elige el margen",
    ] {
        assert!(
            precio.faltan.iter().any(|f| f.contains(motivo)),
            "falta el motivo «{motivo}»: {:?}",
            precio.faltan
        );
    }
}

#[test]
fn un_margen_del_cien_por_cien_sobre_la_venta_no_es_valido() {
    let coste = calcular_coste_de_materiales(
        &resultado(),
        &perfiles(),
        &catalogo(),
        (Some(ModoPerfiles::BarrasEnteras), Porcentaje::CERO),
    );
    let reglas = ReglasDePrecio {
        modo_perfiles: None,
        margen: Some((ModoMargen::SobreVenta, bp(10_000))),
    };
    let linea = DatosDeLinea {
        cantidad: 1,
        recargo_acabado: Porcentaje::CERO,
        mano_de_obra: Dinero::CERO,
        descuento: Porcentaje::CERO,
    };
    assert!(!calcular_precio_de_linea(&coste, &reglas, &linea).es_final());
}

#[test]
fn calcula_bases_cuotas_y_total_por_tipo_de_iva() {
    let lineas = [
        LineaParaTotales {
            importe: euros(24_043),
            tipo_iva: bp(2_100),
        },
        LineaParaTotales {
            importe: euros(10_000),
            tipo_iva: bp(1_000),
        },
    ];
    let sin = calcular_totales(&lineas, Porcentaje::CERO).unwrap();
    assert_eq!(
        (sin.base, sin.iva, sin.total),
        (euros(34_043), euros(5_049 + 1_000), euros(40_092))
    );
    let con = calcular_totales(&lineas, bp(1_000)).unwrap();
    assert_eq!(con.bases[1].base, euros(21_639));
    assert_eq!(con.bases[1].cuota, euros(4_544));
    assert_eq!((con.descuento, con.total), (euros(3_404), euros(36_083)));
    assert_eq!(
        calcular_totales(&[], Porcentaje::CERO).unwrap().total,
        Dinero::CERO
    );
}

proptest! {
    #[test]
    fn las_partes_siempre_suman_el_total(
        importes in prop::collection::vec((0i64..10_000_000, prop::sample::select(vec![0i64, 400, 1_000, 2_100])), 0..20),
        descuento in 0i64..10_000,
    ) {
        let lineas: Vec<_> = importes.iter().map(|(i, t)| LineaParaTotales { importe: euros(*i), tipo_iva: bp(*t) }).collect();
        let t = calcular_totales(&lineas, bp(descuento)).unwrap();
        prop_assert_eq!(t.total.centimos(), t.base.centimos() + t.iva.centimos());
        prop_assert_eq!(t.suma_de_lineas.centimos(), t.base.centimos() + t.descuento.centimos());
        prop_assert_eq!(t.base.centimos(), t.bases.iter().map(|b| b.base.centimos()).sum::<i64>());
        prop_assert!(t.descuento.centimos() >= 0 && t.iva.centimos() >= 0);
    }

    #[test]
    fn el_precio_nunca_baja_del_coste_con_margen_y_sin_descuento(coste in 0i64..100_000_000, margen in 0i64..9_999) {
        let materiales = CosteDeMateriales { perfiles: euros(coste), ..CosteDeMateriales::default() };
        let linea = DatosDeLinea { cantidad: 1, recargo_acabado: Porcentaje::CERO, mano_de_obra: Dinero::CERO, descuento: Porcentaje::CERO };
        for modo in [ModoMargen::SobreCoste, ModoMargen::SobreVenta] {
            let reglas = ReglasDePrecio { modo_perfiles: None, margen: Some((modo, bp(margen))) };
            let precio = calcular_precio_de_linea(&materiales, &reglas, &linea);
            prop_assert!(precio.precio.centimos() >= coste);
        }
    }
}
