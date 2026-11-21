use super::{obligatorio, porcentaje, texto};
use crate::clientes::ErrorDeCampo;
use crate::nif::Nif;
use crate::texto::limpiar_opcional;
use crate::unidades::Porcentaje;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntradaDeProveedor {
    pub nombre: String,
    pub nif: String,
    pub contacto: String,
    pub condiciones_pago: String,
    pub descuento_habitual: String,
    pub notas: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosDeProveedor {
    pub nombre: String,
    pub nif: Option<Nif>,
    pub contacto: Option<String>,
    pub condiciones_pago: Option<String>,
    pub descuento_habitual: Porcentaje,
    pub notas: Option<String>,
}

pub fn validar_proveedor(
    entrada: &EntradaDeProveedor,
) -> Result<DatosDeProveedor, Vec<ErrorDeCampo>> {
    let mut errores = Vec::new();
    let nombre = obligatorio(
        "nombre",
        &entrada.nombre,
        (120, "escribe el nombre del proveedor"),
        &mut errores,
    );
    let nif = limpiar_opcional(&entrada.nif).and_then(|t| match Nif::desde_texto(&t) {
        Ok(nif) => Some(nif),
        Err(e) => {
            errores.push(super::error("nif", e.to_string()));
            None
        }
    });
    let datos = DatosDeProveedor {
        nombre,
        nif,
        contacto: texto("contacto", &entrada.contacto, 200, &mut errores),
        condiciones_pago: texto(
            "condiciones_pago",
            &entrada.condiciones_pago,
            200,
            &mut errores,
        ),
        descuento_habitual: porcentaje(
            "descuento_habitual",
            &entrada.descuento_habitual,
            10_000,
            &mut errores,
        ),
        notas: texto("notas", &entrada.notas, 2_000, &mut errores),
    };
    if errores.is_empty() {
        Ok(datos)
    } else {
        Err(errores)
    }
}
