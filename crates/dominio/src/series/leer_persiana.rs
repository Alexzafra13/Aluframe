//! `persiana: { solo_si, montaje, guia, cajon, cinta }`: persiana montada en la ventana
//! (ADR-0032) o suelta en pared con guías de L (ADR-0033).

use super::lector::{Lector, opcional, ruta_hija};
use super::leer_partes::leer_solo_si;
use super::{MontajeDePersiana, PersianaDeSerie, Valor};

pub fn leer_persiana(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Option<PersianaDeSerie>> {
    let Some(valor) = valor else {
        return Some(None);
    };
    let campos = lector.mapa(valor, ruta)?;
    lector.solo_campos(
        campos,
        &["solo_si", "montaje", "guia", "cajon", "cinta"],
        ruta,
    );
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
    let montaje = leer_montaje(lector, opcional(campos, "montaje"), &r("montaje"))?;
    let guia = guia?;
    if montaje == MontajeDePersiana::Ventana && guia.es_confirmar() {
        lector.informe.error(
            &r("guia"),
            "en la ventana el ancho de la guía no admite CONFIRMAR: de él sale el marco",
        );
        return None;
    }
    Some(Some(PersianaDeSerie {
        solo_si: solo_si?,
        montaje,
        guia,
        cajon: cajon?,
        cinta: cinta?,
    }))
}

fn leer_montaje(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<MontajeDePersiana> {
    let Some(valor) = valor else {
        return Some(MontajeDePersiana::Ventana);
    };
    match lector.texto(valor, ruta)?.as_str() {
        "ventana" => Some(MontajeDePersiana::Ventana),
        "pared" => Some(MontajeDePersiana::Pared),
        _ => {
            lector.informe.error(ruta, "debe ser ventana o pared");
            None
        }
    }
}
