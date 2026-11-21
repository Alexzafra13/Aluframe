use proptest::prelude::*;

use super::*;

fn filas(tabla: &TablaCsv) -> Vec<Vec<&str>> {
    tabla
        .filas
        .iter()
        .map(|f| f.campos.iter().map(String::as_str).collect())
        .collect()
}

#[test]
fn lee_el_csv_de_excel_en_espanol() {
    let texto =
        "\u{feff}Referencia;Descripción;Precio tarifa\r\nAL-1;Marco;12,50\r\nAL-2;Hoja;7\r\n";
    let tabla = leer_csv(texto).unwrap();
    assert_eq!(
        tabla.cabecera,
        vec!["referencia", "descripcion", "precio_tarifa"]
    );
    assert_eq!(
        filas(&tabla),
        vec![vec!["AL-1", "Marco", "12,50"], vec!["AL-2", "Hoja", "7"]]
    );
    assert_eq!(tabla.filas[1].linea, 3);
}

#[test]
fn detecta_la_coma_como_separador() {
    let tabla = leer_csv("a,b\n1,2\n").unwrap();
    assert_eq!(filas(&tabla), vec![vec!["1", "2"]]);
}

#[test]
fn respeta_las_comillas_y_su_escape() {
    let texto = "a;b\n\"Perfil; lacado\";\"dice \"\"hola\"\"\"\n\"dos\nlíneas\";x\nfin;y\n";
    let tabla = leer_csv(texto).unwrap();
    assert_eq!(
        filas(&tabla),
        vec![
            vec!["Perfil; lacado", "dice \"hola\""],
            vec!["dos\nlíneas", "x"],
            vec!["fin", "y"]
        ]
    );
    assert_eq!(tabla.filas[2].linea, 5);
}

#[test]
fn salta_las_lineas_en_blanco() {
    let tabla = leer_csv("a;b\n\n;;\n1;2\n\n").unwrap();
    assert_eq!(filas(&tabla), vec![vec!["1", "2"]]);
}

#[test]
fn informa_de_comillas_mal_cerradas() {
    assert_eq!(
        leer_csv("a;b\n\"abierta;1\n"),
        Err(ErrorCsv::ComillaSinCerrar(2))
    );
    assert_eq!(
        leer_csv("a;b\n\"x\"y;1\n"),
        Err(ErrorCsv::TextoTrasComillas(2))
    );
}

#[test]
fn rechaza_archivos_vacios_o_enormes() {
    assert_eq!(leer_csv(""), Err(ErrorCsv::Vacio));
    assert_eq!(leer_csv("\n\n"), Err(ErrorCsv::Vacio));
    let enorme = "a".repeat(TAMANO_MAXIMO + 1);
    assert_eq!(leer_csv(&enorme), Err(ErrorCsv::DemasiadoGrande));
}

#[test]
fn da_los_campos_por_nombre_de_columna() {
    let tabla = leer_csv("Ref;Notas;Sobra\nX; ;algo\n").unwrap();
    let fila = &tabla.filas[0];
    assert_eq!(fila.campo(tabla.columna("ref")), Some("X".into()));
    assert_eq!(fila.campo(tabla.columna("notas")), None);
    assert_eq!(fila.campo(tabla.columna("falta")), None);
    assert_eq!(
        tabla.columnas_desconocidas(&["ref", "notas"]),
        vec!["sobra"]
    );
}

proptest! {
    #[test]
    fn cualquier_texto_da_tabla_o_error_sin_romperse(texto in "\\PC{0,200}") {
        let _ = leer_csv(&texto);
    }

    #[test]
    fn lo_escrito_con_comillas_se_lee_igual(
        valores in prop::collection::vec("[^\r]{0,12}", 1..5)
    ) {
        prop_assume!(valores.iter().any(|v| !v.trim().is_empty()));
        let citar = |v: &String| format!("\"{}\"", v.replace('"', "\"\""));
        let linea: Vec<String> = valores.iter().map(citar).collect();
        let texto = format!("cabecera\n{}\n", linea.join(";"));
        let tabla = leer_csv(&texto).unwrap();
        prop_assert_eq!(&tabla.filas[0].campos, &valores);
    }
}
