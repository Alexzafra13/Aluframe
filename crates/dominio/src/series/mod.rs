mod calculo;
mod calculo_rellenos;
mod lector;
mod leer_partes;
mod leer_posicion;
mod leer_rellenos;
mod leer_serie;
mod leer_tipologia;
mod modelo;
mod resultado;
mod validacion;
mod valor;

#[cfg(test)]
mod pruebas;

pub use crate::informe::{Informe, Problema};
pub use calculo::{ErrorCalculo, calcular_ventana, calcular_ventana_con_opciones};
pub use modelo::*;
pub use resultado::{
    AccesorioCalculado, ChapaCalculada, ParteLucesCalculado, PiezaCalculada, ResultadoVentana,
    VidrioCalculado,
};
pub use valor::{CONFIRMAR, Dato, Valor};

/// Lee y valida un archivo de serie: con errores no hay serie; los avisos acompañan a la serie.
pub fn importar_serie(raiz: &Valor) -> Result<(Serie, Informe), Informe> {
    let mut lector = lector::Lector::new();
    let serie = leer_serie::leer_cabecera_y_tipologias(&mut lector, raiz);
    let mut informe = lector.informe;
    match serie {
        Some(serie) if !informe.tiene_errores() => {
            validacion::validar_serie(&serie, &mut informe);
            if informe.tiene_errores() {
                Err(informe)
            } else {
                Ok((serie, informe))
            }
        }
        _ => Err(informe),
    }
}
