//! Dibujo esquemático de una ventana en SVG, calculado con las mismas piezas que la hoja de
//! corte (FORMATO_SERIES, apartado 9). Se usa en pantalla y en las hojas impresas.

mod acabado;
mod chapa;
mod cierre;
mod color;
mod cotas;
mod geometria;
mod marcadores;
mod mosquitera;
mod parteluces;
mod persiana;
mod recogedor;
mod svg;

#[cfg(test)]
mod pruebas;

use std::collections::BTreeMap;

use crate::series::{Dibujo, ResultadoVentana, Tipologia};
use crate::unidades::Longitud;

pub use chapa::dibujar_chapa;
pub use color::{BLANCO_9016, ColorDeDibujo, Herrajes, Superficie, leer_color_hex};
pub use geometria::Lado;
pub use parteluces::ParteLucesEnDibujo;
pub use persiana::PersianaEnDibujo;

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
    pub parteluces: Option<ParteLucesEnDibujo>,
    /// Mosquitera: el cajón de 50 se dibuja más alto que el de 45 (ADR-0031).
    pub cajon_de_50: bool,
    /// Persiana en la ventana (ADR-0032): `ancho` y `alto` son entonces los del conjunto.
    pub persiana: Option<PersianaEnDibujo>,
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
            piezas: claves_de_posicion(resultado),
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
            parteluces: parteluces_en_dibujo(resultado),
            cajon_de_50: lleva_cajon_de_50(resultado),
            persiana: persiana_en_dibujo(resultado),
        }
    }
}

/// La serie elige el cajón con la cantidad de cada perfil (ADR-0031); el dibujo lo reconoce por
/// el nombre del perfil que salió, como los números de pieza.
fn lleva_cajon_de_50(resultado: &ResultadoVentana) -> bool {
    let calculadas = resultado.piezas.iter().map(|p| p.perfil.as_str());
    let pendientes = resultado
        .piezas_pendientes
        .iter()
        .map(|p| p.perfil.as_str());
    calculadas
        .chain(pendientes)
        .any(|perfil| perfil.contains("CAJON") && perfil.contains("50"))
}

fn persiana_en_dibujo(resultado: &ResultadoVentana) -> Option<PersianaEnDibujo> {
    resultado.persiana.as_ref().map(|p| PersianaEnDibujo {
        guia: p.guia,
        cajon: p.cajon,
        cinta: Lado::desde_texto(&p.cinta).unwrap_or(Lado::Derecha),
        ancho_ventana: p.ancho_ventana,
        alto_ventana: p.alto_ventana,
    })
}

/// Va chapa abajo si el cálculo tiene chapas; si no, cristal.
fn parteluces_en_dibujo(resultado: &ResultadoVentana) -> Option<ParteLucesEnDibujo> {
    resultado.parteluces.map(|p| ParteLucesEnDibujo {
        abajo: p.abajo,
        altura: p.altura,
        con_chapa: !resultado.chapas.is_empty(),
    })
}

/// El número va donde dice `posicion` si la serie la declara; si no, se deduce del nombre. Las
/// piezas aún sin medida también se numeran en el dibujo, como en la lista.
fn claves_de_posicion(resultado: &ResultadoVentana) -> Vec<(u32, String)> {
    let calculadas = resultado
        .piezas
        .iter()
        .map(|p| (p.numero, p.posicion.as_ref().unwrap_or(&p.perfil).clone()));
    let pendientes = resultado
        .piezas_pendientes
        .iter()
        .map(|p| (p.numero, p.posicion.as_ref().unwrap_or(&p.perfil).clone()));
    let mut todas: Vec<(u32, String)> = calculadas.chain(pendientes).collect();
    todas.sort_by_key(|(numero, _)| *numero);
    todas
}

/// El SVG de la ventana. Nunca falla: con medidas absurdas dibuja lo que puede.
pub fn dibujar_ventana(datos: &DatosDeDibujo) -> String {
    let plano = geometria::Plano::nuevo(datos);
    svg::escribir(&plano, datos)
}
