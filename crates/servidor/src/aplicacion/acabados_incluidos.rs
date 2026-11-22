//! Acabados (colores) que vienen con el programa (`data/acabados.yaml`, ADR-0019): se cargan al
//! arrancar, como las series. Las correcciones llegan con cada versión.

use dominio::catalogo::{AcabadoIncluido, leer_acabados_incluidos};

use crate::bd::consultas::acabados;
use crate::bd::consultas::auditoria::{Apunte, registrar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;
use crate::series::leer_yaml;

const ARCHIVO: &str = include_str!("../../../../data/acabados.yaml");

/// Los acabados del archivo, o por qué no se pueden leer.
pub fn leer_archivo_de_acabados() -> Result<Vec<AcabadoIncluido>, String> {
    leer_acabados_incluidos(&leer_yaml(ARCHIVO)?)
}

/// Devuelve cuántos acabados ha creado. Si el archivo tuviera errores no se carga nada y el
/// programa arranca igual: se avisa en el registro (las pruebas impiden publicarlo así).
pub async fn cargar_acabados_incluidos(estado: &EstadoApp) -> Result<usize, ErrorApp> {
    let lista = match leer_archivo_de_acabados() {
        Ok(lista) => lista,
        Err(error) => {
            tracing::error!("data/acabados.yaml no se carga: {error}");
            return Ok(0);
        }
    };
    let ahora = formatear_utc(estado.ahora());
    estado
        .bd
        .escribir(move |tx| {
            let mut creados = 0;
            for acabado in &lista {
                if acabados::guardar_incluido(tx, acabado)? {
                    creados += 1;
                }
            }
            if creados > 0 {
                let apunte = Apunte {
                    cuando_en: &ahora,
                    usuario_id: None,
                    accion: "cargar_acabados_incluidos",
                    entidad: "acabado",
                    entidad_id: None,
                };
                registrar(tx, &apunte)?;
            }
            Ok(creados)
        })
        .await
}
