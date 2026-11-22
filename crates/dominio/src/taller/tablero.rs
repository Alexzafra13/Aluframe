//! Estado de un trabajo según sus pasos y su columna en el tablero del Inicio (design/02-inicio).

use super::EstadoDePaso;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDeTrabajo {
    Pendiente,
    EnCurso,
    Terminado,
    Entregado,
    Anulado,
}

impl EstadoDeTrabajo {
    pub fn desde_texto(texto: &str) -> Option<EstadoDeTrabajo> {
        match texto {
            "pendiente" => Some(EstadoDeTrabajo::Pendiente),
            "en_curso" => Some(EstadoDeTrabajo::EnCurso),
            "terminado" => Some(EstadoDeTrabajo::Terminado),
            "entregado" => Some(EstadoDeTrabajo::Entregado),
            "anulado" => Some(EstadoDeTrabajo::Anulado),
            _ => None,
        }
    }

    pub fn como_texto(self) -> &'static str {
        match self {
            EstadoDeTrabajo::Pendiente => "pendiente",
            EstadoDeTrabajo::EnCurso => "en_curso",
            EstadoDeTrabajo::Terminado => "terminado",
            EstadoDeTrabajo::Entregado => "entregado",
            EstadoDeTrabajo::Anulado => "anulado",
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            EstadoDeTrabajo::Pendiente => "Pendiente",
            EstadoDeTrabajo::EnCurso => "En curso",
            EstadoDeTrabajo::Terminado => "Terminado",
            EstadoDeTrabajo::Entregado => "Entregado",
            EstadoDeTrabajo::Anulado => "Anulado",
        }
    }
}

/// Entregado y anulado los decide el propietario; el resto sale de los pasos.
pub fn calcular_estado(actual: EstadoDeTrabajo, pasos: &[EstadoDePaso]) -> EstadoDeTrabajo {
    if matches!(
        actual,
        EstadoDeTrabajo::Entregado | EstadoDeTrabajo::Anulado
    ) {
        return actual;
    }
    if !pasos.is_empty() && pasos.iter().all(|p| *p == EstadoDePaso::Hecho) {
        EstadoDeTrabajo::Terminado
    } else if pasos.iter().any(|p| *p != EstadoDePaso::Pendiente) {
        EstadoDeTrabajo::EnCurso
    } else {
        EstadoDeTrabajo::Pendiente
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnaDelTablero {
    PorCortar,
    EnMontaje,
    ParaEntregar,
}

/// «Por cortar» mientras el primer paso de alguna ventana no esté hecho (el corte va primero);
/// fuera del tablero lo entregado y lo anulado.
pub fn columna_del_tablero(
    estado: EstadoDeTrabajo,
    primeros_pasos: &[EstadoDePaso],
) -> Option<ColumnaDelTablero> {
    match estado {
        EstadoDeTrabajo::Entregado | EstadoDeTrabajo::Anulado => None,
        EstadoDeTrabajo::Terminado => Some(ColumnaDelTablero::ParaEntregar),
        _ if primeros_pasos.iter().any(|p| *p != EstadoDePaso::Hecho) => {
            Some(ColumnaDelTablero::PorCortar)
        }
        _ => Some(ColumnaDelTablero::EnMontaje),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use EstadoDePaso as P;
    use EstadoDeTrabajo as T;

    #[test]
    fn el_estado_sale_de_los_pasos_salvo_entregado_y_anulado() {
        assert_eq!(
            calcular_estado(T::Pendiente, &[P::Pendiente, P::Pendiente]),
            T::Pendiente
        );
        assert_eq!(
            calcular_estado(T::Pendiente, &[P::EnCurso, P::Pendiente]),
            T::EnCurso
        );
        assert_eq!(
            calcular_estado(T::EnCurso, &[P::Hecho, P::Hecho]),
            T::Terminado
        );
        assert_eq!(
            calcular_estado(T::Terminado, &[P::Hecho, P::EnCurso]),
            T::EnCurso
        );
        assert_eq!(calcular_estado(T::Entregado, &[P::Pendiente]), T::Entregado);
        assert_eq!(calcular_estado(T::Pendiente, &[]), T::Pendiente);
    }

    #[test]
    fn cada_trabajo_va_a_su_columna() {
        use ColumnaDelTablero as C;
        assert_eq!(
            columna_del_tablero(T::Pendiente, &[P::Pendiente]),
            Some(C::PorCortar)
        );
        assert_eq!(
            columna_del_tablero(T::EnCurso, &[P::Hecho, P::EnCurso]),
            Some(C::PorCortar)
        );
        assert_eq!(
            columna_del_tablero(T::EnCurso, &[P::Hecho, P::Hecho]),
            Some(C::EnMontaje)
        );
        assert_eq!(
            columna_del_tablero(T::Terminado, &[P::Hecho]),
            Some(C::ParaEntregar)
        );
        assert_eq!(columna_del_tablero(T::Entregado, &[P::Hecho]), None);
        for texto in ["pendiente", "en_curso", "terminado", "entregado", "anulado"] {
            let estado = T::desde_texto(texto).unwrap();
            assert_eq!(estado.como_texto(), texto);
            assert!(!estado.nombre().is_empty());
        }
        assert_eq!(T::desde_texto("x"), None);
    }
}
