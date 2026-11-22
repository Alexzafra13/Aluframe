use super::*;

fn momento(dia: u8, hora: u8, minuto: u8) -> MomentoLocal {
    MomentoLocal::nuevo((2026, 11, dia), (hora, minuto))
}

#[test]
fn cuenta_los_dias_como_el_calendario() {
    assert_eq!(dias_desde_1970(1970, 1, 1), 0);
    assert_eq!(dias_desde_1970(2000, 3, 1), 11_017);
    assert_eq!(
        dias_desde_1970(2024, 2, 29) + 1,
        dias_desde_1970(2024, 3, 1)
    );
    // El 23-11-2026 es lunes: empieza semana nueva.
    assert_eq!(momento(22, 0, 0).semana() + 1, momento(23, 0, 0).semana());
    assert_eq!(momento(23, 0, 0).semana(), momento(29, 23, 59).semana());
}

#[test]
fn el_nombre_se_escribe_y_se_lee() {
    let nombre = nombre_de_copia((2026, 11, 22), (14, 5));
    assert_eq!(nombre, "alumframe-2026-11-22-1405.alumframe");
    assert_eq!(leer_nombre_de_copia(&nombre), Some(momento(22, 14, 5)));
    for ajeno in [
        "alumframe-2026-11-22.alumframe",
        "antes-de-importar-2026-11-22-1405.alumframe",
        "alumframe-2026-13-22-1405.alumframe",
        "alumframe-2026-11-22-2460.alumframe",
        "alumframe-2026-11-22-14x5.alumframe",
        "alumframe-2026-11-22-1405.alumframe.tmp",
    ] {
        assert_eq!(leer_nombre_de_copia(ajeno), None, "{ajeno}");
    }
}

#[test]
fn toca_a_las_dos_y_al_arrancar_si_falto() {
    let ayer_a_las_dos = Some(momento(21, 14, 0));
    assert!(!toca_copia(ayer_a_las_dos, None, momento(22, 13, 59)));
    assert!(toca_copia(ayer_a_las_dos, None, momento(22, 14, 0)));
    assert!(!toca_copia(
        Some(momento(22, 14, 1)),
        None,
        momento(22, 18, 0)
    ));
    // Apagado el día 22 entero: al encender el 23 por la mañana se hace la que faltó.
    assert!(toca_copia(ayer_a_las_dos, None, momento(23, 8, 0)));
    assert!(toca_copia(None, None, momento(23, 8, 0)));
    // La copia al cerrar a las 19:00 cubre la de la mañana siguiente, no la de las 14:00.
    assert!(!toca_copia(
        Some(momento(22, 19, 0)),
        None,
        momento(23, 9, 0)
    ));
    assert!(toca_copia(
        Some(momento(22, 19, 0)),
        None,
        momento(23, 14, 0)
    ));
}

#[test]
fn tras_un_fallo_espera_antes_de_reintentar() {
    let fallo = Some(momento(22, 14, 0));
    assert!(!toca_copia(None, fallo, momento(22, 14, 29)));
    assert!(toca_copia(None, fallo, momento(22, 14, 30)));
}

#[test]
fn al_cerrar_copia_salvo_si_es_reciente() {
    assert!(toca_copia_al_cerrar(None, momento(22, 19, 0)));
    assert!(!toca_copia_al_cerrar(
        Some(momento(22, 18, 55)),
        momento(22, 19, 0)
    ));
    assert!(toca_copia_al_cerrar(
        Some(momento(22, 18, 50)),
        momento(22, 19, 0)
    ));
}

fn copia_del(dias_atras: i64, hora: u8) -> String {
    // Desde el domingo 22-11-2026 hacia atrás.
    let dias = dias_desde_1970(2026, 11, 22) - dias_atras;
    let (anio, mes, dia) = fecha_de(dias);
    nombre_de_copia((anio, mes, dia), (hora, 0))
}

/// Inversa de `dias_desde_1970`, solo para construir los casos de prueba.
fn fecha_de(dias: i64) -> (i32, u8, u8) {
    let mut fecha = (2025, 1, 1);
    while dias_desde_1970(fecha.0, fecha.1, fecha.2) != dias {
        fecha = siguiente(fecha);
    }
    fecha
}

fn siguiente((anio, mes, dia): (i32, u8, u8)) -> (i32, u8, u8) {
    if dia < dias_del_mes(anio, mes) {
        (anio, mes, dia + 1)
    } else if mes == 12 {
        (anio + 1, 1, 1)
    } else {
        (anio, mes + 1, 1)
    }
}

fn dias_del_mes(anio: i32, mes: u8) -> u8 {
    let (a, m) = if mes == 12 {
        (anio + 1, 1)
    } else {
        (anio, mes + 1)
    };
    u8::try_from(dias_desde_1970(a, m, 1) - dias_desde_1970(anio, mes, 1)).unwrap()
}

#[test]
fn conserva_siete_diarias_y_cuatro_semanales() {
    let mut nombres: Vec<String> = (0..60).map(|d| copia_del(d, 14)).collect();
    nombres.push(copia_del(0, 19));
    nombres.push("antes-de-importar-x.alumframe".into());
    let sobran = copias_que_sobran(&nombres);
    let quedan: Vec<&String> = nombres.iter().filter(|n| !sobran.contains(n)).collect();
    assert_eq!(quedan.len(), 7 + 4 + 1, "{quedan:?}");
    assert!(
        quedan.contains(&&copia_del(0, 19)),
        "la más reciente del día"
    );
    assert!(
        sobran.contains(&copia_del(0, 14)),
        "la anterior del mismo día sobra"
    );
    assert!(quedan.contains(&&copia_del(6, 14)));
    assert!(!sobran.iter().any(|n| n.starts_with("antes-de")));
    // Las semanales son la más reciente de cada semana anterior a las diarias: el domingo.
    for semanas_atras in 1..=4 {
        assert!(
            quedan.contains(&&copia_del(7 * semanas_atras, 14)),
            "{semanas_atras}"
        );
    }
}

#[test]
fn con_pocas_copias_no_borra_ninguna() {
    let nombres: Vec<String> = [0, 3, 20].iter().map(|d| copia_del(*d, 14)).collect();
    assert!(copias_que_sobran(&nombres).is_empty());
}
