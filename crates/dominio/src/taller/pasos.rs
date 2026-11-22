//! Cada ventana pasa por unos pasos (cortar, montar…). El operario los marca al empezar o al
//! terminar, como prefiera; «Pausar» y «Deshacer» corrigen un toque por error.

use thiserror::Error;

/// Propuesta de PANTALLAS 13; se cambian en Ajustes > Taller (decisión 5 del plan de H4).
pub const PASOS_POR_DEFECTO: [&str; 6] = [
    "Cortar",
    "Montar marco",
    "Montar hojas",
    "Acristalar",
    "Accesorios y cierre",
    "Revisar",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDePaso {
    Pendiente,
    EnCurso,
    Hecho,
}

impl EstadoDePaso {
    pub fn desde_texto(texto: &str) -> Option<EstadoDePaso> {
        match texto {
            "pendiente" => Some(EstadoDePaso::Pendiente),
            "en_curso" => Some(EstadoDePaso::EnCurso),
            "hecho" => Some(EstadoDePaso::Hecho),
            _ => None,
        }
    }

    pub fn como_texto(self) -> &'static str {
        match self {
            EstadoDePaso::Pendiente => "pendiente",
            EstadoDePaso::EnCurso => "en_curso",
            EstadoDePaso::Hecho => "hecho",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccionDePaso {
    Empezar,
    Terminar,
    Pausar,
    Deshacer,
}

impl AccionDePaso {
    pub fn desde_texto(texto: &str) -> Option<AccionDePaso> {
        match texto {
            "empezar" => Some(AccionDePaso::Empezar),
            "terminar" => Some(AccionDePaso::Terminar),
            "pausar" => Some(AccionDePaso::Pausar),
            "deshacer" => Some(AccionDePaso::Deshacer),
            _ => None,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Ese paso ya está {}: recarga la pantalla.", .0.como_texto().replace('_', " "))]
pub struct PasoNoPermitido(pub EstadoDePaso);

/// Se puede terminar sin haber empezado; deshacer devuelve un paso hecho a «en curso».
pub fn aplicar_accion(
    estado: EstadoDePaso,
    accion: AccionDePaso,
) -> Result<EstadoDePaso, PasoNoPermitido> {
    use AccionDePaso as A;
    use EstadoDePaso as E;
    match (estado, accion) {
        (E::Pendiente, A::Empezar) => Ok(E::EnCurso),
        (E::Pendiente | E::EnCurso, A::Terminar) => Ok(E::Hecho),
        (E::EnCurso, A::Pausar) => Ok(E::Pendiente),
        (E::Hecho, A::Deshacer) => Ok(E::EnCurso),
        _ => Err(PasoNoPermitido(estado)),
    }
}

/// Un paso por renglón, sin vacíos ni repetidos; sin ninguno, los de por defecto.
pub fn leer_pasos(texto: &str) -> Vec<String> {
    let mut pasos: Vec<String> = Vec::new();
    for renglon in texto.lines().map(str::trim).filter(|r| !r.is_empty()) {
        let paso: String = renglon.chars().take(60).collect();
        if !pasos.iter().any(|p| p.eq_ignore_ascii_case(&paso)) {
            pasos.push(paso);
        }
    }
    if pasos.is_empty() {
        return PASOS_POR_DEFECTO.iter().map(|p| (*p).to_owned()).collect();
    }
    pasos
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use AccionDePaso as A;
    use EstadoDePaso as E;

    #[test]
    fn se_puede_terminar_sin_empezar_y_deshacer_lo_hecho() {
        assert_eq!(aplicar_accion(E::Pendiente, A::Empezar), Ok(E::EnCurso));
        assert_eq!(aplicar_accion(E::Pendiente, A::Terminar), Ok(E::Hecho));
        assert_eq!(aplicar_accion(E::EnCurso, A::Terminar), Ok(E::Hecho));
        assert_eq!(aplicar_accion(E::EnCurso, A::Pausar), Ok(E::Pendiente));
        assert_eq!(aplicar_accion(E::Hecho, A::Deshacer), Ok(E::EnCurso));
    }

    #[test]
    fn rechaza_lo_que_ya_no_toca() {
        assert!(aplicar_accion(E::Hecho, A::Terminar).is_err());
        assert!(aplicar_accion(E::EnCurso, A::Empezar).is_err());
        let error = aplicar_accion(E::Hecho, A::Empezar).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Ese paso ya está hecho: recarga la pantalla."
        );
    }

    #[test]
    fn lee_los_pasos_de_ajustes() {
        assert_eq!(
            leer_pasos(" Cortar \n\ncortar\nSoldar\n"),
            ["Cortar", "Soldar"]
        );
        assert_eq!(leer_pasos("  ").len(), PASOS_POR_DEFECTO.len());
        for texto in ["pendiente", "en_curso", "hecho"] {
            assert_eq!(
                EstadoDePaso::desde_texto(texto).unwrap().como_texto(),
                texto
            );
        }
        assert_eq!(AccionDePaso::desde_texto("pausar"), Some(A::Pausar));
        assert_eq!(AccionDePaso::desde_texto("x"), None);
    }
}
