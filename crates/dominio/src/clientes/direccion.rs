use super::datos::{ErrorDeCampo, comprobar_telefono};
use crate::texto::limpiar_opcional;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntradaDeDireccion {
    pub etiqueta: String,
    pub calle: String,
    pub cp: String,
    pub localidad: String,
    pub provincia: String,
    pub contacto_nombre: String,
    pub contacto_telefono: String,
    pub notas: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatosDeDireccion {
    pub etiqueta: Option<String>,
    pub calle: Option<String>,
    pub cp: Option<String>,
    pub localidad: Option<String>,
    pub provincia: Option<String>,
    pub contacto_nombre: Option<String>,
    pub contacto_telefono: Option<String>,
    pub notas: Option<String>,
}

impl DatosDeDireccion {
    /// «Obra Calle Mayor · Calle Mayor 3, Zafra» para listas y desplegables.
    pub fn resumen(&self) -> String {
        let lugar: Vec<&str> = [self.calle.as_deref(), self.localidad.as_deref()]
            .into_iter()
            .flatten()
            .collect();
        match (self.etiqueta.as_deref(), lugar.is_empty()) {
            (Some(etiqueta), true) => etiqueta.to_owned(),
            (Some(etiqueta), false) => format!("{etiqueta} · {}", lugar.join(", ")),
            (None, false) => lugar.join(", "),
            (None, true) => "Dirección sin datos".to_owned(),
        }
    }
}

/// Una dirección con todo en blanco no se guarda: devuelve `Ok(None)`.
pub fn validar_direccion(
    entrada: &EntradaDeDireccion,
) -> Result<Option<DatosDeDireccion>, Vec<ErrorDeCampo>> {
    let mut errores = Vec::new();
    let mut texto = |campo: &'static str, valor: &str, largo: usize| {
        let limpio = limpiar_opcional(valor);
        if limpio.as_ref().is_some_and(|v| v.chars().count() > largo) {
            errores.push(ErrorDeCampo {
                campo,
                mensaje: format!("no puede tener más de {largo} caracteres"),
            });
        }
        limpio
    };
    let datos = DatosDeDireccion {
        etiqueta: texto("etiqueta", &entrada.etiqueta, 100),
        calle: texto("calle", &entrada.calle, 200),
        cp: texto("cp", &entrada.cp, 10),
        localidad: texto("localidad", &entrada.localidad, 200),
        provincia: texto("provincia", &entrada.provincia, 200),
        contacto_nombre: texto("contacto_nombre", &entrada.contacto_nombre, 200),
        contacto_telefono: texto("contacto_telefono", &entrada.contacto_telefono, 30),
        notas: texto("notas", &entrada.notas, 2_000),
    };
    comprobar_telefono(
        "contacto_telefono",
        datos.contacto_telefono.as_deref(),
        &mut errores,
    );
    if !errores.is_empty() {
        return Err(errores);
    }
    Ok((datos != DatosDeDireccion::default()).then_some(datos))
}
