use super::*;

const DATOS: &str = "# comentario\n\
=Zafra|Cabrahiga\n\
Avenida\tDuques de Feria\t06300\t0\n\
Calle\tSevilla\t06300\t0\n\
Calle\tSevillanas\t06300\t0\n\
Finca\tLa Cabrahiga\t06300\t1\n\
=Badajoz|Gévora\n\
Avenida\tSanta Marina\t06005|06006\t0\n\
Calle\tSevilla\t06010\t0\n\
Calle\tRío Gévora\t06180\t1\n\
Calle\tSin Código\t\t0\n\
mal formada\n";

fn callejero() -> Callejero {
    let mut callejero = Callejero::default();
    callejero.anadir(DATOS, "Badajoz");
    callejero
}

fn calles(escrito: &str, localidad: &str, codigo_postal: &str) -> Vec<String> {
    let donde = DondeBuscar {
        localidad,
        codigo_postal,
    };
    callejero()
        .sugerir(escrito, donde, 10)
        .iter()
        .map(|s| format!("{} ({})", s.calle, s.localidad))
        .collect()
}

#[test]
fn con_la_localidad_escrita_solo_sugiere_sus_calles() {
    assert_eq!(
        calles("sevilla", "zafra", ""),
        ["Calle Sevilla (Zafra)", "Calle Sevillanas (Zafra)"]
    );
    assert_eq!(
        calles("duq", "ZAFRA", ""),
        ["Avenida Duques de Feria (Zafra)"]
    );
    assert_eq!(calles("rio", "Gevora", ""), ["Calle Río Gévora (Gévora)"]);
}

#[test]
fn sin_localidad_busca_en_toda_la_provincia_y_dice_el_pueblo() {
    assert_eq!(
        calles("calle sevilla", "", ""),
        [
            "Calle Sevilla (Badajoz)",
            "Calle Sevilla (Zafra)",
            "Calle Sevillanas (Zafra)"
        ]
    );
}

#[test]
fn una_localidad_que_no_esta_busca_por_su_codigo_o_no_sugiere() {
    assert_eq!(
        calles("sevilla", "Pedanía Nueva", "06010"),
        ["Calle Sevilla (Badajoz)"]
    );
    assert!(calles("sevilla", "Sevilla", "").is_empty());
    assert_eq!(calles("sevilla", "", "06300").len(), 2);
}

#[test]
fn entiende_las_abreviaturas_del_principio() {
    assert_eq!(
        calles("c/ sevilla", "Zafra", "")[0],
        "Calle Sevilla (Zafra)"
    );
    assert_eq!(calles("c/sevilla", "Zafra", "")[0], "Calle Sevilla (Zafra)");
    assert_eq!(
        calles("avda. duques", "Zafra", ""),
        ["Avenida Duques de Feria (Zafra)"]
    );
    assert_eq!(expandir_abreviaturas("pza espana"), "plaza espana");
    assert_eq!(expandir_abreviaturas("avenida"), "avenida");
}

#[test]
fn da_los_codigos_de_la_calle_y_la_provincia() {
    let callejero = callejero();
    let donde = DondeBuscar {
        localidad: "Badajoz",
        codigo_postal: "",
    };
    let marina = &callejero.sugerir("santa marina", donde, 5)[0];
    assert_eq!(marina.codigos_postales, ["06005", "06006"]);
    assert_eq!(marina.provincia, "Badajoz");
    let sin_codigo = &callejero.sugerir("sin codigo", donde, 5)[0];
    assert!(sin_codigo.codigos_postales.is_empty());
}

#[test]
fn con_menos_de_dos_letras_no_sugiere_y_respeta_el_maximo() {
    let callejero = callejero();
    assert_eq!(callejero.len(), 8);
    assert!(
        callejero
            .sugerir("s", DondeBuscar::default(), 10)
            .is_empty()
    );
    assert_eq!(
        callejero.sugerir("calle", DondeBuscar::default(), 2).len(),
        2
    );
}

#[test]
fn las_calles_con_codigo_postal_van_antes_que_las_que_no_tienen() {
    let mut callejero = Callejero::default();
    callejero.anadir(
        "=Badajoz\nCalle\tSantaren\t\t0\nCalle\tSanta Ana\t06001\t0\n",
        "Badajoz",
    );
    let donde = DondeBuscar {
        localidad: "Badajoz",
        codigo_postal: "",
    };
    let calles: Vec<String> = callejero
        .sugerir("santa", donde, 5)
        .into_iter()
        .map(|s| s.calle)
        .collect();
    assert_eq!(calles, ["Calle Santa Ana", "Calle Santaren"]);
}
