use serde::{Deserialize, Serialize};

use super::ErrorAcceso;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rol {
    Propietario,
    Operario,
}

impl Rol {
    pub fn como_texto(self) -> &'static str {
        match self {
            Rol::Propietario => "propietario",
            Rol::Operario => "operario",
        }
    }

    pub fn desde_texto(texto: &str) -> Result<Rol, ErrorAcceso> {
        match texto {
            "propietario" => Ok(Rol::Propietario),
            "operario" => Ok(Rol::Operario),
            otro => Err(ErrorAcceso::RolDesconocido(otro.to_owned())),
        }
    }

    pub fn es_propietario(self) -> bool {
        self == Rol::Propietario
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convierte_ida_y_vuelta_cada_rol() {
        for rol in [Rol::Propietario, Rol::Operario] {
            assert_eq!(Rol::desde_texto(rol.como_texto()), Ok(rol));
        }
    }

    #[test]
    fn rechaza_rol_desconocido() {
        assert_eq!(
            Rol::desde_texto("admin"),
            Err(ErrorAcceso::RolDesconocido("admin".into()))
        );
    }
}
