use proptest::prelude::*;

use super::*;
use crate::unidades::Decimal;

const PARAMETROS: [&str; 6] = [
    "descuento_marco_horizontal",
    "descuento_hoja_horizontal",
    "descuento_hoja_4_hojas",
    "descuento_hoja_ancho",
    "descuento_inversor",
    "descuento_hoja_vertical",
];

fn es_variable(nombre: &str) -> bool {
    nombre == "ancho" || nombre == "alto" || PARAMETROS.contains(&nombre)
}

fn d(texto: &str) -> Decimal {
    Decimal::desde_texto(texto).unwrap()
}

fn valores(ancho: &str, alto: &str) -> Valores {
    let mut valores = Valores::new();
    valores.insert("ancho".into(), d(ancho));
    valores.insert("alto".into(), d(alto));
    for (nombre, valor) in PARAMETROS
        .iter()
        .zip(["4.1", "2", "3", "4.85", "10.3", "5.3"])
    {
        valores.insert((*nombre).into(), d(valor));
    }
    valores
}

fn calcular(texto: &str, ancho: &str, alto: &str) -> Result<Decimal, ErrorFormula> {
    Formula::analizar(texto, &es_variable)?.evaluar(&valores(ancho, alto))
}

fn error_de(texto: &str) -> ErrorFormula {
    Formula::analizar(texto, &es_variable).unwrap_err()
}

#[test]
fn calcula_las_formulas_reales_de_las_series() {
    assert_eq!(
        calcular("ancho - descuento_marco_horizontal", "150", "120"),
        Ok(d("145.9"))
    );
    assert_eq!(
        calcular("ancho / 2 - descuento_hoja_horizontal", "150", "120"),
        Ok(d("73"))
    );
    assert_eq!(
        calcular("ancho / 2 - descuento_hoja_horizontal", "151", "120"),
        Ok(d("73.5"))
    );
    assert_eq!(
        calcular("(ancho - descuento_hoja_4_hojas) / 4", "303", "120"),
        Ok(d("75"))
    );
    assert_eq!(
        calcular("(ancho - descuento_hoja_ancho) / 2", "120", "140"),
        Ok(d("57.575"))
    );
    assert_eq!(
        calcular("alto - descuento_inversor", "120", "140"),
        Ok(d("129.7"))
    );
}

#[test]
fn las_bisagras_dependen_del_alto() {
    let bisagras = "2 * si(alto > 140, 3, 2)";
    assert_eq!(calcular(bisagras, "120", "140"), Ok(d("4")));
    assert_eq!(calcular(bisagras, "120", "140.1"), Ok(d("6")));
}

#[test]
fn respeta_la_precedencia_y_la_asociatividad() {
    assert_eq!(calcular("2 + 3 * 4", "1", "1"), Ok(d("14")));
    assert_eq!(calcular("(2 + 3) * 4", "1", "1"), Ok(d("20")));
    assert_eq!(calcular("10 - 4 - 3", "1", "1"), Ok(d("3")));
    assert_eq!(calcular("100 / 4 / 5", "1", "1"), Ok(d("5")));
    assert_eq!(calcular("-ancho + 200", "150", "1"), Ok(d("50")));
    assert_eq!(calcular("-2 * 3", "1", "1"), Ok(d("-6")));
}

#[test]
fn calcula_las_funciones_permitidas() {
    assert_eq!(calcular("minimo(3, 1, 2)", "1", "1"), Ok(d("1")));
    assert_eq!(calcular("maximo(ancho, alto)", "150", "120"), Ok(d("150")));
    assert_eq!(calcular("redondear(2.5)", "1", "1"), Ok(d("3")));
    assert_eq!(calcular("techo(2.1)", "1", "1"), Ok(d("3")));
    assert_eq!(calcular("suelo(2.9)", "1", "1"), Ok(d("2")));
    assert_eq!(calcular("suelo(0 - 2.1)", "1", "1"), Ok(d("-3")));
    assert_eq!(calcular("si(ancho == 150, 1, 0)", "150", "1"), Ok(d("1")));
    assert_eq!(calcular("si(ancho <= 100, 1, 0)", "150", "1"), Ok(d("0")));
}

#[test]
fn rechaza_nombres_y_funciones_desconocidos() {
    assert_eq!(
        error_de("anchura - 2"),
        ErrorFormula::VariableDesconocida("anchura".into())
    );
    assert_eq!(
        error_de("raiz(4)"),
        ErrorFormula::FuncionDesconocida("raiz".into())
    );
    assert_eq!(
        error_de("minimo + 1"),
        ErrorFormula::VariableDesconocida("minimo".into())
    );
}

#[test]
fn comprueba_el_numero_de_argumentos() {
    assert!(matches!(
        error_de("redondear(1, 2)"),
        ErrorFormula::Aridad {
            funcion: "redondear",
            ..
        }
    ));
    assert!(matches!(
        error_de("minimo(1)"),
        ErrorFormula::Aridad {
            funcion: "minimo",
            ..
        }
    ));
    assert!(matches!(
        error_de("si(alto > 1, 2)"),
        ErrorFormula::Aridad { funcion: "si", .. }
    ));
}

#[test]
fn solo_admite_comparaciones_dentro_de_si() {
    assert_eq!(error_de("alto > 3"), ErrorFormula::ComparacionFueraDeSi);
    assert_eq!(
        error_de("minimo(alto > 1, 2)"),
        ErrorFormula::ComparacionFueraDeSi
    );
    assert_eq!(error_de("(alto > 1)"), ErrorFormula::ComparacionFueraDeSi);
    assert_eq!(error_de("si(alto, 1, 2)"), ErrorFormula::SiSinComparacion);
}

#[test]
fn rechaza_formulas_mal_formadas() {
    assert_eq!(error_de(""), ErrorFormula::Vacia);
    assert_eq!(error_de("   "), ErrorFormula::Vacia);
    assert_eq!(error_de("ancho -"), ErrorFormula::FinInesperado);
    assert!(matches!(
        error_de("ancho 2"),
        ErrorFormula::Inesperado { posicion: 7, .. }
    ));
    assert!(matches!(error_de("(ancho"), ErrorFormula::FinInesperado));
    assert!(matches!(
        error_de("ancho)"),
        ErrorFormula::Inesperado { .. }
    ));
}

#[test]
fn limita_la_longitud_y_la_profundidad() {
    let larga = format!("ancho{}", " + 1".repeat(60));
    assert_eq!(
        error_de(&larga),
        ErrorFormula::DemasiadoLarga { maximo: 200 }
    );
    let profunda = format!("{}1{}", "(".repeat(20), ")".repeat(20));
    assert_eq!(
        error_de(&profunda),
        ErrorFormula::DemasiadoProfunda { maximo: 16 }
    );
    let aceptable = format!("{}1{}", "(".repeat(10), ")".repeat(10));
    assert!(Formula::analizar(&aceptable, &es_variable).is_ok());
}

#[test]
fn la_division_por_cero_es_un_error_al_evaluar() {
    let resultado = calcular("ancho / (alto - alto)", "150", "120");
    assert_eq!(
        resultado,
        Err(ErrorFormula::Calculo(
            crate::unidades::ErrorUnidad::DivisionPorCero
        ))
    );
}

#[test]
fn lista_las_variables_que_usa() {
    let formula =
        Formula::analizar("(ancho - descuento_hoja_ancho) / 2 + ancho", &es_variable).unwrap();
    assert_eq!(formula.variables(), vec!["ancho", "descuento_hoja_ancho"]);
    assert_eq!(
        formula.texto(),
        "(ancho - descuento_hoja_ancho) / 2 + ancho"
    );
}

type Estrategia = BoxedStrategy<Expresion>;

fn expresion_aleatoria() -> impl Strategy<Value = Expresion> {
    let hoja = prop_oneof![
        (0i64..1000).prop_map(|n| Expresion::Numero(Decimal::desde_entero(n))),
        prop::sample::select(vec!["ancho", "alto", "descuento_inversor"])
            .prop_map(|n| Expresion::Variable(n.to_owned())),
    ];
    hoja.prop_recursive(4, 24, 3, |interior| {
        let interior = interior.boxed();
        prop_oneof![
            interior
                .clone()
                .prop_map(|e| Expresion::Negativo(Box::new(e))),
            operacion_aleatoria(interior.clone()),
            llamada_aleatoria(interior.clone()),
            si_aleatorio(interior),
        ]
    })
}

fn operacion_aleatoria(interior: Estrategia) -> Estrategia {
    let operador = prop::sample::select(vec![
        Operador::Suma,
        Operador::Resta,
        Operador::Producto,
        Operador::Cociente,
    ]);
    (operador, interior.clone(), interior)
        .prop_map(|(operador, a, b)| Expresion::Operacion {
            operador,
            izquierda: Box::new(a),
            derecha: Box::new(b),
        })
        .boxed()
}

fn llamada_aleatoria(interior: Estrategia) -> Estrategia {
    let funcion = prop::sample::select(vec![Funcion::Minimo, Funcion::Maximo]);
    (funcion, prop::collection::vec(interior, 2..4))
        .prop_map(|(funcion, argumentos)| Expresion::Llamada {
            funcion,
            argumentos,
        })
        .boxed()
}

fn si_aleatorio(interior: Estrategia) -> Estrategia {
    (
        interior.clone(),
        interior.clone(),
        interior.clone(),
        interior,
    )
        .prop_map(|(a, b, c, e)| Expresion::Si {
            condicion: Condicion {
                comparador: Comparador::Mayor,
                izquierda: Box::new(a),
                derecha: Box::new(b),
            },
            entonces: Box::new(c),
            si_no: Box::new(e),
        })
        .boxed()
}

proptest! {
    #[test]
    fn cualquier_texto_da_resultado_o_error_sin_romperse(texto in "[a-z0-9_+*/(),.<>= -]{0,80}") {
        if let Ok(formula) = Formula::analizar(&texto, &es_variable) {
            let _ = formula.evaluar(&valores("150", "120"));
        }
    }

    #[test]
    fn imprimir_y_volver_a_leer_da_la_misma_formula(expresion in expresion_aleatoria()) {
        let texto = expresion.to_string();
        prop_assume!(texto.chars().count() <= LONGITUD_MAXIMA);
        let leida = Formula::analizar(&texto, &es_variable).unwrap();
        prop_assert_eq!(leida.expresion(), &expresion);
    }

    #[test]
    fn evaluar_nunca_desborda_en_silencio(expresion in expresion_aleatoria()) {
        let _ = super::evaluacion::evaluar(&expresion, &valores("150", "120"));
    }
}
