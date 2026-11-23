//! Apuntes: notas y tareas sueltas para lo que no es una ventana (ADR-0025). Del operario al
//! dueño («ha llamado la vecina del 3.º») o del dueño a un operario («cambiar la cerradura del
//! portal»). Pueden llevar cliente, dirección y medidas tomadas, todo opcional menos el texto.

use thiserror::Error;

use crate::texto::limpiar_opcional;

pub const TEXTO_MAXIMO: usize = 2_000;
pub const MEDIDAS_MAXIMO: usize = 2_000;
pub const CONTACTO_MAXIMO: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorApunte {
    #[error("Escribe qué hay que hacer o qué ha pasado.")]
    SinTexto,
    #[error("«{campo}» no puede tener más de {maximo} caracteres.")]
    DemasiadoLargo { campo: &'static str, maximo: usize },
    #[error("Un apunte {desde} no puede pasar a {hasta}.")]
    CambioNoPermitido {
        desde: &'static str,
        hasta: &'static str,
    },
}

/// Lo escrito en el formulario, tal cual llega.
#[derive(Debug, Clone, Default)]
pub struct EntradaDeApunte {
    pub texto: String,
    pub medidas: String,
    /// Nombre o teléfono de quien llama, si no está dado de alta como cliente.
    pub contacto: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosDeApunte {
    pub texto: String,
    pub medidas: Option<String>,
    pub contacto: Option<String>,
}

pub fn validar_apunte(entrada: &EntradaDeApunte) -> Result<DatosDeApunte, ErrorApunte> {
    let texto = limpiar_opcional(&entrada.texto).ok_or(ErrorApunte::SinTexto)?;
    comprobar_largo("Qué hay que hacer", &texto, TEXTO_MAXIMO)?;
    let medidas = limpiar_opcional(&entrada.medidas);
    if let Some(m) = &medidas {
        comprobar_largo("Medidas", m, MEDIDAS_MAXIMO)?;
    }
    let contacto = limpiar_opcional(&entrada.contacto);
    if let Some(c) = &contacto {
        comprobar_largo("Contacto", c, CONTACTO_MAXIMO)?;
    }
    Ok(DatosDeApunte {
        texto,
        medidas,
        contacto,
    })
}

fn comprobar_largo(campo: &'static str, valor: &str, maximo: usize) -> Result<(), ErrorApunte> {
    if valor.chars().count() > maximo {
        return Err(ErrorApunte::DemasiadoLargo { campo, maximo });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoApunte {
    Pendiente,
    Hecho,
    /// Ya se hizo el presupuesto con lo apuntado.
    Presupuestado,
}

impl EstadoApunte {
    pub fn como_texto(self) -> &'static str {
        match self {
            EstadoApunte::Pendiente => "pendiente",
            EstadoApunte::Hecho => "hecho",
            EstadoApunte::Presupuestado => "presupuestado",
        }
    }

    pub fn desde_texto(texto: &str) -> Option<EstadoApunte> {
        [
            EstadoApunte::Pendiente,
            EstadoApunte::Hecho,
            EstadoApunte::Presupuestado,
        ]
        .into_iter()
        .find(|e| e.como_texto() == texto)
    }

    /// Hecho se puede reabrir si se marcó por error; lo presupuestado ya no cambia: el
    /// presupuesto sigue su propio camino.
    pub fn pasar_a(self, nuevo: EstadoApunte) -> Result<EstadoApunte, ErrorApunte> {
        use EstadoApunte::{Hecho, Pendiente, Presupuestado};
        match (self, nuevo) {
            (Pendiente, Hecho | Presupuestado) | (Hecho, Pendiente) => Ok(nuevo),
            _ => Err(ErrorApunte::CambioNoPermitido {
                desde: self.como_texto(),
                hasta: nuevo.como_texto(),
            }),
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn entrada(texto: &str) -> EntradaDeApunte {
        EntradaDeApunte {
            texto: texto.into(),
            ..EntradaDeApunte::default()
        }
    }

    #[test]
    fn exige_el_texto_y_deja_lo_demas_opcional() {
        assert_eq!(validar_apunte(&entrada("   ")), Err(ErrorApunte::SinTexto));
        let datos = validar_apunte(&entrada("  Presupuesto de mosquiteras ")).unwrap();
        assert_eq!(datos.texto, "Presupuesto de mosquiteras");
        assert_eq!(datos.medidas, None);
        assert_eq!(datos.contacto, None);
    }

    #[test]
    fn guarda_medidas_y_contacto_y_rechaza_lo_demasiado_largo() {
        let con_todo = EntradaDeApunte {
            texto: "Cocina".into(),
            medidas: "Ventana cocina 120 × 100".into(),
            contacto: "Vecina del 3.º, 612 345 678".into(),
        };
        let datos = validar_apunte(&con_todo).unwrap();
        assert_eq!(datos.medidas.as_deref(), Some("Ventana cocina 120 × 100"));
        assert_eq!(
            datos.contacto.as_deref(),
            Some("Vecina del 3.º, 612 345 678")
        );
        let largo = entrada(&"a".repeat(TEXTO_MAXIMO + 1));
        assert!(matches!(
            validar_apunte(&largo),
            Err(ErrorApunte::DemasiadoLargo { .. })
        ));
        let contacto_largo = EntradaDeApunte {
            contacto: "ñ".repeat(CONTACTO_MAXIMO + 1),
            ..entrada("x")
        };
        assert!(validar_apunte(&contacto_largo).is_err());
    }

    #[test]
    fn solo_permite_los_cambios_de_estado_con_sentido() {
        use EstadoApunte::{Hecho, Pendiente, Presupuestado};
        assert_eq!(Pendiente.pasar_a(Hecho), Ok(Hecho));
        assert_eq!(Hecho.pasar_a(Pendiente), Ok(Pendiente));
        assert_eq!(Pendiente.pasar_a(Presupuestado), Ok(Presupuestado));
        for (desde, hasta) in [
            (Presupuestado, Pendiente),
            (Presupuestado, Hecho),
            (Hecho, Presupuestado),
            (Pendiente, Pendiente),
        ] {
            assert!(desde.pasar_a(hasta).is_err(), "{desde:?} → {hasta:?}");
        }
    }

    #[test]
    fn cada_estado_se_guarda_y_se_lee_por_su_nombre() {
        for estado in [
            EstadoApunte::Pendiente,
            EstadoApunte::Hecho,
            EstadoApunte::Presupuestado,
        ] {
            assert_eq!(EstadoApunte::desde_texto(estado.como_texto()), Some(estado));
        }
        assert_eq!(EstadoApunte::desde_texto("visto"), None);
    }
}
