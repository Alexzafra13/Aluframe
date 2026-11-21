use proptest::prelude::*;

use super::*;
use crate::unidades::{Longitud, Porcentaje, PrecioUnitario};

const CABECERA: &str = "referencia;descripcion;unidad;precio_tarifa;longitud_barra;tipo\n";

fn pu(texto: &str) -> PrecioUnitario {
    PrecioUnitario::desde_texto_es(texto).unwrap()
}

fn leer(filas: &str) -> (Vec<FilaDeTarifa>, crate::informe::Informe) {
    leer_tarifa(&format!("{CABECERA}{filas}"))
}

#[test]
fn lee_una_tarifa_correcta() {
    let (filas, informe) =
        leer("AL-1;Marco lateral;barra;23,4567;640;perfil\nTOR-1;Tornillo;caja;8,5;;\n");
    assert!(!informe.tiene_errores(), "{:?}", informe.errores);
    assert_eq!(filas.len(), 2);
    assert_eq!(filas[0].precio, pu("23,4567"));
    assert_eq!(
        filas[0].longitud_barra,
        Some(Longitud::desde_decimas_de_mm(64_000))
    );
    assert_eq!(filas[0].tipo, Some(TipoArticulo::Perfil));
    assert_eq!(filas[1].unidad, UnidadDeVenta::Caja);
    assert_eq!(filas[1].longitud_barra, None);
    assert_eq!(filas[1].tipo, None);
}

#[test]
fn informa_de_todos_los_errores_de_las_filas() {
    let (filas, informe) = leer(
        "A;Uno;barra;0;640\nB;Dos;saco;5\nC;Tres;metro;\nA;Cuatro;metro;1\n;Cinco;metro;1\nD;Seis;barra;5\nE;Siete;metro;1,23456\nF;Ocho;metro;2;;cosa\n",
    );
    let mensajes: Vec<String> = informe.errores.iter().map(ToString::to_string).collect();
    let tiene = |texto: &str| mensajes.iter().any(|m| m.contains(texto));
    assert!(
        tiene("línea 2: el precio «0» debe ser mayor que cero"),
        "{mensajes:?}"
    );
    assert!(tiene("línea 3: unidad desconocida «saco»"));
    assert!(tiene("línea 4: falta el precio"));
    assert!(tiene("línea 6: falta la referencia"));
    assert!(tiene("línea 7: falta la longitud de barra"));
    assert!(tiene("línea 8: precio: «1,23456» tiene más de 4 decimales"));
    assert!(tiene("línea 9: tipo desconocido «cosa»"));
    assert_eq!(filas.len(), 1, "solo la fila A de la línea 5 es válida");
}

#[test]
fn las_referencias_duplicadas_son_un_error() {
    let (_, informe) = leer("A;Uno;metro;1\nA;Otra;metro;2\n");
    assert_eq!(
        informe.errores[0].to_string(),
        "línea 3: la referencia A ya aparece en la línea 2"
    );
}

#[test]
fn exige_las_columnas_de_la_plantilla() {
    let (_, informe) = leer_tarifa("referencia;precio\nA;1\n");
    assert!(
        informe.errores[0]
            .mensaje
            .contains("faltan columnas: descripcion, unidad, precio_tarifa")
    );
    assert!(informe.avisos[0].mensaje.contains("«precio» no se usa"));
}

#[test]
fn un_archivo_sin_filas_o_roto_es_un_error() {
    assert!(leer_tarifa(CABECERA).1.tiene_errores());
    assert!(leer_tarifa("").1.tiene_errores());
    assert!(leer("\"A;Uno;metro;1\n").1.tiene_errores());
}

#[test]
fn la_longitud_fuera_de_las_barras_solo_avisa() {
    let (filas, informe) = leer("A;Uno;metro;1;640\nB;Dos;barra;1;-5\nC;Tres;barra;1;640,005\n");
    assert_eq!(filas.len(), 1);
    assert!(
        informe.avisos[0]
            .mensaje
            .contains("solo se usa si la unidad es «barra»")
    );
    assert!(informe.errores[0].mensaje.contains("mayor que cero"));
    assert!(informe.errores[1].mensaje.contains("no es exacta"));
}

#[test]
fn admite_abreviaturas_de_unidad() {
    for (texto, unidad) in [
        ("ml", UnidadDeVenta::Metro),
        ("m²", UnidadDeVenta::MetroCuadrado),
        ("Ud.", UnidadDeVenta::Unidad),
        ("BARRAS", UnidadDeVenta::Barra),
    ] {
        assert_eq!(UnidadDeVenta::desde_texto(texto), Some(unidad), "{texto}");
    }
    for unidad in UnidadDeVenta::TODAS {
        assert_eq!(
            UnidadDeVenta::desde_texto(unidad.como_texto()),
            Some(unidad)
        );
    }
    for tipo in TipoArticulo::TODOS {
        assert_eq!(TipoArticulo::desde_texto(tipo.como_texto()), Some(tipo));
    }
    assert_eq!(UnidadDeVenta::MetroCuadrado.nombre(), "m²");
}

fn vigente(referencia: &str, precio: Option<&str>) -> ArticuloVigente {
    ArticuloVigente {
        referencia: referencia.into(),
        unidad: UnidadDeVenta::Metro,
        precio: precio.map(pu),
    }
}

#[test]
fn compara_con_la_tarifa_vigente() {
    let (filas, mut informe) =
        leer("A;Uno;metro;10\nB;Dos;metro;14\nC;Tres;metro;5\nN;Nuevo;unidad;1\n");
    let vigentes = [
        vigente("A", Some("10")),
        vigente("B", Some("10")),
        vigente("C", None),
        vigente("Z", Some("3")),
        vigente("SIN", None),
    ];
    let comparacion = comparar_con_vigente(&filas, &vigentes, &mut informe);
    assert_eq!(comparacion.iguales, 1);
    assert_eq!(comparacion.nuevos.len(), 1);
    assert_eq!(comparacion.nuevos[0].referencia, "N");
    assert_eq!(comparacion.cambios.len(), 2);
    assert_eq!(
        comparacion.cambios[0].variacion,
        Some(Porcentaje::desde_puntos_basicos(4_000))
    );
    assert_eq!(comparacion.cambios[1].anterior, None);
    assert_eq!(comparacion.ausentes, vec!["Z"]);
    assert!(comparacion.hay_cambios());
    assert!(
        informe
            .avisos
            .iter()
            .any(|a| a.mensaje.contains("B: el precio cambia un +40 %"))
    );
    assert!(!informe.tiene_errores());
}

#[test]
fn un_salto_de_hasta_el_30_por_ciento_no_avisa_y_un_cambio_de_unidad_si() {
    let (filas, mut informe) = leer("A;Uno;barra;13;640\n");
    let comparacion = comparar_con_vigente(&filas, &[vigente("A", Some("10"))], &mut informe);
    assert_eq!(comparacion.cambios.len(), 1);
    assert_eq!(informe.avisos.len(), 1);
    assert!(
        informe.avisos[0]
            .mensaje
            .contains("la unidad cambia de metro a barra")
    );
}

#[test]
fn valida_los_ajustes_masivos() {
    let p = Porcentaje::desde_puntos_basicos;
    assert!(validar_ajuste(p(500)).is_ok());
    assert!(validar_ajuste(p(-9_999)).is_ok());
    assert!(validar_ajuste(p(0)).is_err());
    assert!(validar_ajuste(p(-10_000)).is_err());
    assert!(validar_ajuste(p(10_001)).is_err());
}

proptest! {
    #[test]
    fn cualquier_texto_da_filas_o_errores_sin_romperse(texto in "\\PC{0,300}") {
        let _ = leer_tarifa(&format!("{CABECERA}{texto}"));
    }

    #[test]
    fn cada_fila_valida_aparece_una_vez_con_su_precio(precios in prop::collection::vec(1u32..10_000_000, 1..30)) {
        let cuerpo: String = precios.iter().enumerate()
            .map(|(i, p)| format!("R{i};Art {i};unidad;{},{:04}\n", p / 10_000, p % 10_000))
            .collect();
        let (filas, informe) = leer(&cuerpo);
        prop_assert!(!informe.tiene_errores());
        prop_assert_eq!(filas.len(), precios.len());
        for (fila, precio) in filas.iter().zip(&precios) {
            prop_assert_eq!(fila.precio.diezmilesimas(), i64::from(*precio));
        }
    }
}
