//! Dibujo esquemático de una ventana en SVG, calculado con las mismas piezas que la hoja de
//! corte (FORMATO_SERIES, apartado 9). Se usa en pantalla y en las hojas impresas.

mod acabado;
mod color;
mod geometria;
mod marcadores;
mod svg;

#[cfg(test)]
mod pruebas;

use std::collections::BTreeMap;

use crate::series::{Dibujo, ResultadoVentana, Tipologia};
use crate::unidades::Longitud;

pub use color::{BLANCO_9016, ColorDeDibujo, Herrajes, Superficie, leer_color_hex};
pub use geometria::Lado;

/// Lo que necesita el dibujo, ya resuelto a partir de la serie, la línea y el cálculo.
#[derive(Debug, Clone)]
pub struct DatosDeDibujo {
    pub dibujo: Dibujo,
    pub ancho: Longitud,
    pub alto: Longitud,
    /// (número, perfil) de cada pieza, en el orden de la hoja de corte.
    pub piezas: Vec<(u32, String)>,
    pub bisagras_por_hoja: u32,
    pub hoja_cremona: Option<Lado>,
    pub titulo: String,
    /// El del acabado de la línea; blanco si no se eligió.
    pub color: ColorDeDibujo,
}

impl DatosDeDibujo {
    /// Bisagras por hoja = cantidad del accesorio indicado ÷ hojas; la cremona, de la opción.
    pub fn desde_calculo(
        tipologia: &Tipologia,
        resultado: &ResultadoVentana,
        opciones: &BTreeMap<String, String>,
        (ancho, alto): (Longitud, Longitud),
    ) -> DatosDeDibujo {
        let dibujo = tipologia.dibujo.clone();
        let hojas = u32::from(dibujo.hojas.max(1));
        let bisagras = dibujo.bisagras.as_ref().map_or(0, |codigo| {
            let total: u32 = resultado
                .accesorios_sumados()
                .iter()
                .filter(|(articulo, _)| articulo == codigo)
                .filter_map(|(_, cantidad)| cantidad.a_entero())
                .filter_map(|n| u32::try_from(n).ok())
                .sum();
            total / hojas
        });
        let hoja_cremona = dibujo.cremona.as_ref().and_then(|codigo| {
            let valor = opciones.get(codigo).cloned().or_else(|| {
                tipologia
                    .opciones
                    .iter()
                    .find(|o| &o.codigo == codigo)
                    .map(|o| o.por_defecto.clone())
            });
            valor.as_deref().and_then(Lado::desde_texto)
        });
        DatosDeDibujo {
            piezas: resultado.piezas.iter().map(clave_de_posicion).collect(),
            titulo: format!(
                "{} de {} × {} cm",
                tipologia.nombre,
                ancho.formatear_cm(),
                alto.formatear_cm()
            ),
            dibujo,
            ancho,
            alto,
            bisagras_por_hoja: bisagras,
            hoja_cremona,
            color: ColorDeDibujo::default(),
        }
    }
}

/// El número va donde dice `posicion` si la serie la declara; si no, se deduce del nombre.
fn clave_de_posicion(pieza: &crate::series::PiezaCalculada) -> (u32, String) {
    let clave = pieza.posicion.as_ref().unwrap_or(&pieza.perfil);
    (pieza.numero, clave.clone())
}

/// El SVG de la ventana. Nunca falla: con medidas absurdas dibuja lo que puede.
pub fn dibujar_ventana(datos: &DatosDeDibujo) -> String {
    let plano = geometria::Plano::nuevo(datos);
    svg::escribir(&plano, datos)
}
