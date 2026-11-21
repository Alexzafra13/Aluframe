use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, Weekday};

pub fn formatear_utc(instante: OffsetDateTime) -> String {
    let segundos = instante
        .to_offset(UtcOffset::UTC)
        .replace_nanosecond(0)
        .unwrap_or(instante);
    segundos
        .format(&Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

pub fn leer_utc(texto: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(texto, &Rfc3339).ok()
}

pub fn a_hora_de_madrid(instante: OffsetDateTime) -> OffsetDateTime {
    let horas = if es_horario_de_verano(instante) { 2 } else { 1 };
    let desfase = UtcOffset::from_hms(horas, 0, 0).unwrap_or(UtcOffset::UTC);
    instante.to_offset(desfase)
}

/// Fecha del día en Madrid como `AAAA-MM-DD`, más los días indicados.
pub fn fecha_de_madrid(instante: OffsetDateTime, mas_dias: i64) -> String {
    let fecha = a_hora_de_madrid(instante).date() + time::Duration::days(mas_dias);
    let formato = format_description!("[year]-[month]-[day]");
    fecha.format(formato).unwrap_or_default()
}

pub fn anio_de_madrid(instante: OffsetDateTime) -> i64 {
    i64::from(a_hora_de_madrid(instante).year())
}

pub fn formatear_para_pantalla(instante: OffsetDateTime) -> String {
    let formato = format_description!("[day]/[month]/[year] [hour]:[minute]");
    a_hora_de_madrid(instante)
        .format(formato)
        .unwrap_or_default()
}

/// Regla de la UE: del último domingo de marzo al último domingo de octubre, a la 01:00 UTC.
fn es_horario_de_verano(instante: OffsetDateTime) -> bool {
    let utc = instante.to_offset(UtcOffset::UTC);
    let anio = utc.year();
    match (
        cambio_de_hora(anio, Month::March),
        cambio_de_hora(anio, Month::October),
    ) {
        (Some(inicio), Some(fin)) => utc >= inicio && utc < fin,
        _ => false,
    }
}

fn cambio_de_hora(anio: i32, mes: Month) -> Option<OffsetDateTime> {
    let dia_31 = Date::from_calendar_date(anio, mes, 31).ok()?;
    let retroceso = i64::from(dia_31.weekday().number_days_from_sunday());
    let domingo = dia_31 - time::Duration::days(retroceso);
    debug_assert_eq!(domingo.weekday(), Weekday::Sunday);
    let una_en_punto = Time::from_hms(1, 0, 0).ok()?;
    Some(PrimitiveDateTime::new(domingo, una_en_punto).assume_utc())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn en_invierno_madrid_va_una_hora_por_delante() {
        let hora = a_hora_de_madrid(datetime!(2026-01-15 10:00 UTC));
        assert_eq!(hora.hour(), 11);
    }

    #[test]
    fn en_verano_madrid_va_dos_horas_por_delante() {
        let hora = a_hora_de_madrid(datetime!(2026-07-15 10:00 UTC));
        assert_eq!(hora.hour(), 12);
    }

    #[test]
    fn cambia_justo_a_la_una_utc_del_ultimo_domingo_de_marzo() {
        assert_eq!(a_hora_de_madrid(datetime!(2026-03-29 00:59 UTC)).hour(), 1);
        assert_eq!(a_hora_de_madrid(datetime!(2026-03-29 01:00 UTC)).hour(), 3);
    }

    #[test]
    fn vuelve_al_horario_de_invierno_el_ultimo_domingo_de_octubre() {
        assert_eq!(a_hora_de_madrid(datetime!(2026-10-25 00:59 UTC)).hour(), 2);
        assert_eq!(a_hora_de_madrid(datetime!(2026-10-25 01:00 UTC)).hour(), 2);
        assert_eq!(
            a_hora_de_madrid(datetime!(2026-10-25 01:00 UTC))
                .offset()
                .whole_hours(),
            1
        );
    }

    #[test]
    fn formatea_y_lee_en_utc_sin_fracciones() {
        let instante = datetime!(2026-11-21 09:30:15.123 UTC);
        let texto = formatear_utc(instante);
        assert_eq!(texto, "2026-11-21T09:30:15Z");
        assert_eq!(leer_utc(&texto), Some(datetime!(2026-11-21 09:30:15 UTC)));
    }

    #[test]
    fn muestra_la_fecha_en_formato_espanol() {
        assert_eq!(
            formatear_para_pantalla(datetime!(2026-11-21 09:05 UTC)),
            "21/11/2026 10:05"
        );
    }
}
