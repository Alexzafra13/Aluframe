use super::*;
use crate::tarifa::{TipoArticulo, UnidadDeVenta};
use crate::unidades::{Longitud, Porcentaje};

fn campos(errores: &[crate::clientes::ErrorDeCampo]) -> Vec<&str> {
    errores.iter().map(|e| e.campo).collect()
}

#[test]
fn un_proveedor_necesita_nombre_y_valida_nif_y_descuento() {
    let bueno = EntradaDeProveedor {
        nombre: " Aluval ".into(),
        descuento_habitual: "12,5".into(),
        ..EntradaDeProveedor::default()
    };
    let datos = validar_proveedor(&bueno).unwrap();
    assert_eq!(datos.nombre, "Aluval");
    assert_eq!(
        datos.descuento_habitual,
        Porcentaje::desde_puntos_basicos(1_250)
    );
    let malo = EntradaDeProveedor {
        nif: "B1".into(),
        descuento_habitual: "120".into(),
        ..EntradaDeProveedor::default()
    };
    assert_eq!(
        campos(&validar_proveedor(&malo).unwrap_err()),
        vec!["nombre", "nif", "descuento_habitual"]
    );
    let texto = EntradaDeProveedor {
        nombre: "A".into(),
        descuento_habitual: "mucho".into(),
        ..EntradaDeProveedor::default()
    };
    assert_eq!(
        campos(&validar_proveedor(&texto).unwrap_err()),
        vec!["descuento_habitual"]
    );
}

#[test]
fn el_acabado_base_no_lleva_recargo() {
    let lacado = EntradaDeAcabado {
        nombre: "Lacado RAL 7016".into(),
        es_base: false,
        recargo: "15".into(),
    };
    assert_eq!(
        validar_acabado(&lacado).unwrap().recargo,
        Porcentaje::desde_puntos_basicos(1_500)
    );
    let blanco = EntradaDeAcabado {
        nombre: "Blanco".into(),
        es_base: true,
        recargo: "".into(),
    };
    assert!(validar_acabado(&blanco).unwrap().es_base);
    let mal = EntradaDeAcabado {
        nombre: "".into(),
        es_base: true,
        recargo: "5".into(),
    };
    assert_eq!(
        campos(&validar_acabado(&mal).unwrap_err()),
        vec!["nombre", "recargo"]
    );
}

fn articulo(unidad: &str) -> EntradaDeArticulo {
    EntradaDeArticulo {
        referencia: "AL-21.01".into(),
        descripcion: "Marco lateral".into(),
        tipo: "perfil".into(),
        unidad: unidad.into(),
        ..EntradaDeArticulo::default()
    }
}

#[test]
fn un_articulo_por_barras_necesita_su_longitud() {
    let sin = validar_articulo(&articulo("barra")).unwrap_err();
    assert_eq!(campos(&sin), vec!["longitud_barra_cm"]);
    let con = EntradaDeArticulo {
        longitud_barra_cm: "640".into(),
        stock_minimo: "4,5".into(),
        ..articulo("barra")
    };
    let datos = validar_articulo(&con).unwrap();
    assert_eq!(
        datos.longitud_barra,
        Some(Longitud::desde_decimas_de_mm(64_000))
    );
    assert_eq!(datos.stock_minimo_mil, 4_500);
    assert_eq!(datos.tipo, TipoArticulo::Perfil);
    let mal = EntradaDeArticulo {
        longitud_barra_cm: "seis".into(),
        ..articulo("barra")
    };
    assert_eq!(
        campos(&validar_articulo(&mal).unwrap_err()),
        vec!["longitud_barra_cm"]
    );
}

#[test]
fn ignora_la_longitud_si_no_es_barra_y_valida_la_caja() {
    let caja = EntradaDeArticulo {
        longitud_barra_cm: "640".into(),
        unidades_por_caja: "100".into(),
        ..articulo("caja")
    };
    let datos = validar_articulo(&caja).unwrap();
    assert_eq!(datos.unidad, UnidadDeVenta::Caja);
    assert_eq!(
        (datos.longitud_barra, datos.unidades_por_caja),
        (None, Some(100))
    );
    let mala = EntradaDeArticulo {
        unidades_por_caja: "0".into(),
        stock_minimo: "-1".into(),
        ..articulo("caja")
    };
    assert_eq!(
        campos(&validar_articulo(&mala).unwrap_err()),
        vec!["unidades_por_caja", "stock_minimo"]
    );
}

#[test]
fn exige_referencia_descripcion_tipo_y_unidad() {
    let errores = validar_articulo(&EntradaDeArticulo::default()).unwrap_err();
    assert_eq!(
        campos(&errores),
        vec!["referencia", "descripcion", "tipo", "unidad"]
    );
}
