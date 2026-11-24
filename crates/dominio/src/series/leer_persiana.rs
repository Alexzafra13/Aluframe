//! `persiana: { solo_si, guia, cajon, cinta }`: persiana montada en la ventana (ADR-0032).

use super::lector::{Lector, opcional, ruta_hija};
use super::leer_partes::leer_solo_si;
use super::{Dato, PersianaDeSerie, Valor};

pub fn leer_persiana(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Option<PersianaDeSerie>> {
    let Some(valor) = valor else {
        return Some(None);
    };
    let campos = lector.mapa(valor, ruta)?;
    lector.solo_campos(campos, &["solo_si", "guia", "cajon", "cinta"], ruta);
    let r = |campo: &str| ruta_hija(ruta, campo);
    let solo_si = leer_solo_si(lector, opcional(campos, "solo_si"), &r("solo_si"));
    let guia = lector
        .obligatorio(campos, "guia", ruta)
        .and_then(|v| lector.formula(v, &r("guia")));
    let cajon = lector
        .obligatorio(campos, "cajon", ruta)
        .and_then(|v| lector.formula(v, &r("cajon")));
    let cinta = lector
        .obligatorio(campos, "cinta", ruta)
        .and_then(|v| lector.texto(v, &r("cinta")));
    let Dato::Valor(guia) = guia? else {
        lector
            .informe
            .error(&r("guia"), "el ancho de la guía no admite CONFIRMAR");
        return None;
    };
    Some(Some(PersianaDeSerie {
        solo_si: solo_si?,
        guia,
        cajon: cajon?,
        cinta: cinta?,
    }))
}
