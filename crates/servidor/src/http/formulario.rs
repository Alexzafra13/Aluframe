//! Lectura de formularios con campos repetidos (direcciones de montaje `d0_calle`, `d1_calle`…),
//! que el extractor `Form` con una estructura no admite.

use std::collections::{BTreeSet, HashMap};

use dominio::clientes::{EntradaDeCliente, EntradaDeDireccion};

pub struct Campos(HashMap<String, String>);

/// Más bloques de dirección que esto en un alta no tiene sentido y limita el trabajo.
const MAXIMO_DE_DIRECCIONES: usize = 20;

impl Campos {
    pub fn de(pares: Vec<(String, String)>) -> Campos {
        Campos(pares.into_iter().collect())
    }

    pub fn texto(&self, nombre: &str) -> String {
        self.0.get(nombre).cloned().unwrap_or_default()
    }

    pub fn marcado(&self, nombre: &str) -> bool {
        self.0.contains_key(nombre)
    }

    pub fn entero(&self, nombre: &str) -> Option<i64> {
        self.0.get(nombre).and_then(|v| v.trim().parse().ok())
    }

    pub fn cliente(&self) -> EntradaDeCliente {
        EntradaDeCliente {
            tipo: self.texto("tipo"),
            nombre: self.texto("nombre"),
            nif: self.texto("nif"),
            email: self.texto("email"),
            telefono: self.texto("telefono"),
            fiscal_calle: self.texto("fiscal_calle"),
            fiscal_cp: self.texto("fiscal_cp"),
            fiscal_localidad: self.texto("fiscal_localidad"),
            fiscal_provincia: self.texto("fiscal_provincia"),
            fiscal_pais: self.texto("fiscal_pais"),
            forma_de_pago: self.texto("forma_de_pago"),
            notas: self.texto("notas"),
        }
    }

    pub fn direccion(&self, n: usize) -> EntradaDeDireccion {
        let campo = |nombre: &str| self.texto(&format!("d{n}_{nombre}"));
        EntradaDeDireccion {
            etiqueta: campo("etiqueta"),
            calle: campo("calle"),
            cp: campo("cp"),
            localidad: campo("localidad"),
            provincia: campo("provincia"),
            contacto_nombre: campo("contacto_nombre"),
            contacto_telefono: campo("contacto_telefono"),
            notas: campo("notas"),
        }
    }

    /// Los números de bloque presentes, en orden (`d0_…`, `d3_…`).
    pub fn bloques_de_direccion(&self) -> Vec<usize> {
        let numeros: BTreeSet<usize> = self
            .0
            .keys()
            .filter_map(|clave| clave.strip_prefix('d')?.split_once('_')?.0.parse().ok())
            .filter(|n| *n < MAXIMO_DE_DIRECCIONES)
            .collect();
        numeros.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn campos(pares: &[(&str, &str)]) -> Campos {
        Campos::de(
            pares
                .iter()
                .map(|(a, b)| ((*a).into(), (*b).into()))
                .collect(),
        )
    }

    #[test]
    fn reune_los_bloques_de_direccion() {
        let c = campos(&[
            ("d3_calle", "B"),
            ("d0_calle", "A"),
            ("d99_calle", "X"),
            ("dato", "y"),
        ]);
        assert_eq!(c.bloques_de_direccion(), vec![0, 3]);
        assert_eq!(c.direccion(3).calle, "B");
        assert_eq!(c.direccion(1).calle, "");
    }

    #[test]
    fn lee_casillas_y_enteros() {
        let c = campos(&[("misma", "1"), ("version", " 4 ")]);
        assert!(c.marcado("misma"));
        assert!(!c.marcado("otra"));
        assert_eq!(c.entero("version"), Some(4));
        assert_eq!(c.entero("misma_no"), None);
    }
}
