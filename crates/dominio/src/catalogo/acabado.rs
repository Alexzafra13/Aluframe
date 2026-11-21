use super::{obligatorio, porcentaje};
use crate::clientes::ErrorDeCampo;
use crate::unidades::Porcentaje;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntradaDeAcabado {
    pub nombre: String,
    pub es_base: bool,
    pub recargo: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosDeAcabado {
    pub nombre: String,
    pub es_base: bool,
    pub recargo: Porcentaje,
}

/// El acabado base (blanco) es el precio de tarifa: no puede llevar recargo.
pub fn validar_acabado(entrada: &EntradaDeAcabado) -> Result<DatosDeAcabado, Vec<ErrorDeCampo>> {
    let mut errores = Vec::new();
    let nombre = obligatorio(
        "nombre",
        &entrada.nombre,
        (60, "escribe el nombre del color o acabado"),
        &mut errores,
    );
    let recargo = porcentaje("recargo", &entrada.recargo, 100_000, &mut errores);
    if entrada.es_base && recargo != Porcentaje::CERO {
        errores.push(super::error(
            "recargo",
            "el acabado base es el precio de tarifa y no lleva recargo",
        ));
    }
    if errores.is_empty() {
        Ok(DatosDeAcabado {
            nombre,
            es_base: entrada.es_base,
            recargo,
        })
    } else {
        Err(errores)
    }
}
