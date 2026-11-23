//! Cómo se ve un presupuesto según su validez (ADR-0027). Caducado y archivado se calculan con
//! la fecha de hoy, nunca se guardan: no hace falta ninguna tarea que pase por la noche.

/// Un presupuesto caducado sin aceptar sale de la lista principal medio año después.
pub const DIAS_HASTA_ARCHIVAR: i64 = 182;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situacion {
    Borrador,
    Enviado,
    Aceptado,
    Rechazado,
    /// Pasó su validez sin aceptarse: para aceptarlo hay que renovarlo (precios de hoy).
    Caducado,
    /// Caducado o rechazado hace más de medio año: fuera de la lista principal, nunca borrado.
    Archivado,
}

impl Situacion {
    pub fn nombre(self) -> &'static str {
        match self {
            Situacion::Borrador => "Borrador",
            Situacion::Enviado => "Enviado",
            Situacion::Aceptado => "Aceptado",
            Situacion::Rechazado => "Rechazado",
            Situacion::Caducado => "Caducado",
            Situacion::Archivado => "Archivado",
        }
    }

    /// Para el estilo de la etiqueta y los filtros de la lista.
    pub fn clave(self) -> &'static str {
        match self {
            Situacion::Borrador => "borrador",
            Situacion::Enviado => "enviado",
            Situacion::Aceptado => "aceptado",
            Situacion::Rechazado => "rechazado",
            Situacion::Caducado => "caducado",
            Situacion::Archivado => "archivado",
        }
    }
}

/// `validez_hasta`, `hoy` y `archivar_antes_de` van en AAAA-MM-DD (hora de Madrid), así que se
/// comparan como texto. La validez incluye su último día. Sin fecha de validez no caduca.
pub fn situacion(
    estado: &str,
    validez_hasta: Option<&str>,
    (hoy, archivar_antes_de): (&str, &str),
) -> Situacion {
    if estado == "aceptado" {
        return Situacion::Aceptado;
    }
    let validez = validez_hasta.filter(|v| !v.is_empty());
    if validez.is_some_and(|v| v < archivar_antes_de) {
        return Situacion::Archivado;
    }
    match estado {
        "rechazado" => Situacion::Rechazado,
        "borrador" | "enviado" if validez.is_some_and(|v| v < hoy) => Situacion::Caducado,
        "borrador" => Situacion::Borrador,
        "enviado" => Situacion::Enviado,
        _ => Situacion::Caducado,
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const HOY: (&str, &str) = ("2026-11-23", "2026-05-25");

    #[test]
    fn la_validez_incluye_su_ultimo_dia() {
        assert_eq!(
            situacion("enviado", Some("2026-11-23"), HOY),
            Situacion::Enviado
        );
        assert_eq!(
            situacion("enviado", Some("2026-11-22"), HOY),
            Situacion::Caducado
        );
        assert_eq!(
            situacion("borrador", Some("2026-11-01"), HOY),
            Situacion::Caducado
        );
        assert_eq!(situacion("borrador", None, HOY), Situacion::Borrador);
        assert_eq!(situacion("enviado", Some(""), HOY), Situacion::Enviado);
    }

    #[test]
    fn a_los_seis_meses_se_archiva_pero_un_aceptado_nunca() {
        assert_eq!(
            situacion("enviado", Some("2026-05-24"), HOY),
            Situacion::Archivado
        );
        assert_eq!(
            situacion("enviado", Some("2026-05-25"), HOY),
            Situacion::Caducado
        );
        assert_eq!(
            situacion("rechazado", Some("2026-01-10"), HOY),
            Situacion::Archivado
        );
        assert_eq!(
            situacion("rechazado", Some("2026-11-01"), HOY),
            Situacion::Rechazado,
            "un rechazado no caduca: ya tiene respuesta"
        );
        assert_eq!(
            situacion("aceptado", Some("2020-01-01"), HOY),
            Situacion::Aceptado
        );
    }

    #[test]
    fn cada_situacion_tiene_nombre_y_clave() {
        assert_eq!(Situacion::Caducado.nombre(), "Caducado");
        assert_eq!(Situacion::Archivado.clave(), "archivado");
        assert_eq!(situacion("caducado", None, HOY), Situacion::Caducado);
    }
}
