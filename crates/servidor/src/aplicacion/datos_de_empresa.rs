//! Datos de la empresa para la cabecera y el pie de los documentos. Sale lo que esté escrito:
//! para que algo no aparezca, se deja en blanco.

use dominio::clientes::normalizar_telefono;
use dominio::iban::normalizar_iban;
use dominio::nif::Nif;

use super::ajustes_presupuesto::{AjustesDePresupuesto, TextosDeAjustes};

/// Cabecera de la hoja impresa: el logotipo o, sin él, el nombre en grande.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CabeceraDeEmpresa {
    pub logo: Option<String>,
    pub titulo: String,
    /// La razón social, cuando el nombre comercial es otro.
    pub razon_social: Option<String>,
    pub lineas: Vec<String>,
    pub iban: Option<String>,
    pub pie: Option<String>,
}

impl CabeceraDeEmpresa {
    pub fn desde(textos: &TextosDeAjustes, logo: Option<String>) -> CabeceraDeEmpresa {
        let v = |clave: &str| Some(textos.valor(clave).trim().to_owned()).filter(|t| !t.is_empty());
        let razon = v("empresa.nombre");
        let comercial = v("empresa.nombre_comercial");
        let titulo = comercial
            .clone()
            .or_else(|| razon.clone())
            .unwrap_or_default();
        let razon_social = razon.filter(|r| comercial.as_ref().is_some_and(|c| c != r));
        let poblacion = unir(&[v("empresa.cp"), v("empresa.localidad")], " ");
        let poblacion = match (poblacion, v("empresa.provincia")) {
            (Some(p), Some(prov)) if !p.contains(&prov) => Some(format!("{p} ({prov})")),
            (Some(p), _) => Some(p),
            (None, prov) => prov,
        };
        let telefonos = unir(
            &[
                v("empresa.telefono").map(|t| format!("Tel. {t}")),
                v("empresa.movil").map(|m| format!("Móvil {m}")),
            ],
            " · ",
        );
        let lineas = [
            v("empresa.nif").map(|n| format!("NIF {n}")),
            v("empresa.direccion"),
            poblacion,
            telefonos,
            unir(&[v("empresa.email"), v("empresa.web")], " · "),
        ];
        CabeceraDeEmpresa {
            logo,
            titulo,
            razon_social,
            lineas: lineas.into_iter().flatten().collect(),
            iban: v("empresa.iban"),
            pie: v("empresa.pie"),
        }
    }

    pub fn es_vacia(&self) -> bool {
        self.logo.is_none() && self.titulo.is_empty() && self.lineas.is_empty()
    }
}

fn unir(partes: &[Option<String>], separador: &str) -> Option<String> {
    let presentes: Vec<&str> = partes.iter().flatten().map(String::as_str).collect();
    (!presentes.is_empty()).then(|| presentes.join(separador))
}

impl AjustesDePresupuesto {
    /// Los datos de la empresa en renglones, como se copian al aceptar un presupuesto.
    pub fn empresa(&self) -> Vec<String> {
        let cabecera = CabeceraDeEmpresa::desde(&self.textos, None);
        let mut datos: Vec<String> = Some(cabecera.titulo)
            .filter(|t| !t.is_empty())
            .into_iter()
            .chain(cabecera.razon_social)
            .chain(cabecera.lineas)
            .collect();
        datos.extend(cabecera.iban.map(|i| format!("IBAN {i}")));
        datos
    }
}

/// Comprueba NIF, teléfonos e IBAN y los deja escritos de forma uniforme.
pub fn normalizar_empresa(textos: &mut TextosDeAjustes) -> Result<(), String> {
    let nif = textos.valor("empresa.nif").trim().to_owned();
    if !nif.is_empty() {
        let valido = Nif::desde_texto(&nif).map_err(|e| format!("NIF de la empresa: {e}"))?;
        textos.cambiar("empresa.nif", valido.como_texto().to_owned());
    }
    for (clave, nombre) in [("empresa.telefono", "Teléfono"), ("empresa.movil", "Móvil")] {
        let escrito = textos.valor(clave).trim().to_owned();
        if !escrito.is_empty() {
            let normalizado =
                normalizar_telefono(&escrito, None).map_err(|e| format!("{nombre}: {e}"))?;
            textos.cambiar(clave, normalizado);
        }
    }
    let iban = textos.valor("empresa.iban").trim().to_owned();
    if !iban.is_empty() {
        let normalizado = normalizar_iban(&iban).map_err(|e| format!("IBAN: {e}"))?;
        textos.cambiar("empresa.iban", normalizado);
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn textos(pares: &[(&str, &str)]) -> TextosDeAjustes {
        TextosDeAjustes(
            super::super::ajustes_presupuesto::CLAVES
                .iter()
                .map(|c| {
                    let valor = pares.iter().find(|(k, _)| k == c).map_or("", |(_, v)| v);
                    ((*c).to_owned(), valor.to_owned())
                })
                .collect(),
        )
    }

    #[test]
    fn sin_nombre_comercial_el_titulo_es_la_razon_social() {
        let t = textos(&[
            ("empresa.nombre", "Aluminios Pérez SL"),
            ("empresa.cp", "06300"),
            ("empresa.localidad", "Zafra"),
            ("empresa.provincia", "Badajoz"),
        ]);
        let c = CabeceraDeEmpresa::desde(&t, None);
        assert_eq!(c.titulo, "Aluminios Pérez SL");
        assert_eq!(c.razon_social, None);
        assert_eq!(c.lineas, vec!["06300 Zafra (Badajoz)"]);
    }

    #[test]
    fn con_nombre_comercial_la_razon_social_va_debajo() {
        let t = textos(&[
            ("empresa.nombre", "Pérez e Hijos SL"),
            ("empresa.nombre_comercial", "Aluminios Pérez"),
            ("empresa.telefono", "924 555 555"),
            ("empresa.movil", "612 345 678"),
        ]);
        let c = CabeceraDeEmpresa::desde(&t, None);
        assert_eq!(c.titulo, "Aluminios Pérez");
        assert_eq!(c.razon_social.as_deref(), Some("Pérez e Hijos SL"));
        assert_eq!(c.lineas, vec!["Tel. 924 555 555 · Móvil 612 345 678"]);
        assert!(!c.es_vacia());
        assert!(CabeceraDeEmpresa::desde(&textos(&[]), None).es_vacia());
    }

    #[test]
    fn normaliza_nif_telefonos_e_iban_y_rechaza_los_mal_escritos() {
        let mut t = textos(&[
            ("empresa.nif", "b-12345674"),
            ("empresa.movil", "612345678"),
            ("empresa.iban", "es9121000418450200051332"),
        ]);
        normalizar_empresa(&mut t).unwrap();
        assert_eq!(t.valor("empresa.nif"), "B12345674");
        assert_eq!(t.valor("empresa.movil"), "612 345 678");
        assert_eq!(t.valor("empresa.iban"), "ES91 2100 0418 4502 0005 1332");
        let mut mal = textos(&[("empresa.iban", "ES00 1234")]);
        assert!(
            normalizar_empresa(&mut mal)
                .unwrap_err()
                .starts_with("IBAN")
        );
    }
}
