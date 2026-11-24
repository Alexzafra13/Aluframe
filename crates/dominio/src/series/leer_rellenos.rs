//! Lo que va dentro de las hojas: cristales, chapas y el parteluces que los separa (ADR-0030).

use super::lector::{Lector, leer_lista, opcional, ruta_hija};
use super::leer_partes::leer_solo_si;
use super::{Dato, ParteLucesDeSerie, PiezaDeChapa, PiezaDeVidrio, Valor, Vidrio};

pub fn leer_vidrio(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<Dato<Vidrio>> {
    let campos = lector.mapa(valor, ruta)?;
    if opcional(campos, "estado").is_some_and(Valor::es_confirmar) {
        lector.solo_campos(campos, &["estado"], ruta);
        lector.anotar_confirmar(ruta);
        return Some(Dato::Confirmar);
    }
    lector.solo_campos(campos, &["tipo", "piezas"], ruta);
    let tipo = lector
        .obligatorio(campos, "tipo", ruta)
        .and_then(|v| lector.texto(v, &ruta_hija(ruta, "tipo")));
    let lista = lector.obligatorio(campos, "piezas", ruta)?;
    let piezas = leer_lista(
        lector,
        lista,
        &ruta_hija(ruta, "piezas"),
        leer_pieza_de_vidrio,
    );
    Some(Dato::Valor(Vidrio {
        tipo: tipo?,
        piezas: piezas?,
    }))
}

/// El ancho y el alto admiten `CONFIRMAR` (el cristal de arriba de un parteluces mientras no se
/// sepa su descuento); la cantidad no.
fn leer_pieza_de_vidrio(
    lector: &mut Lector,
    elemento: &Valor,
    ruta: &str,
) -> Option<PiezaDeVidrio> {
    let campos = lector.mapa(elemento, ruta)?;
    lector.solo_campos(
        campos,
        &["nombre", "cantidad", "ancho", "alto", "solo_si"],
        ruta,
    );
    let r = |campo: &str| ruta_hija(ruta, campo);
    let mut formula = |campo: &str| {
        let valor = lector.obligatorio(campos, campo, ruta)?;
        lector.formula(valor, &r(campo))
    };
    let (cantidad, ancho, alto) = (formula("cantidad"), formula("ancho"), formula("alto"));
    let cantidad = match cantidad? {
        Dato::Valor(formula) => formula,
        Dato::Confirmar => {
            let mensaje =
                "la cantidad de cristales no admite CONFIRMAR; usa vidrio: { estado: CONFIRMAR }";
            lector.informe.error(&r("cantidad"), mensaje);
            return None;
        }
    };
    let nombre = texto_opcional(lector, opcional(campos, "nombre"), &r("nombre"))?;
    let solo_si = leer_solo_si(lector, opcional(campos, "solo_si"), &r("solo_si"))?;
    Some(PiezaDeVidrio {
        nombre,
        cantidad,
        ancho: ancho?,
        alto: alto?,
        solo_si,
    })
}

pub fn leer_chapas(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Vec<PiezaDeChapa>> {
    match valor {
        None => Some(Vec::new()),
        Some(valor) => leer_lista(lector, valor, ruta, leer_chapa),
    }
}

fn leer_chapa(lector: &mut Lector, elemento: &Valor, ruta: &str) -> Option<PiezaDeChapa> {
    let campos = lector.mapa(elemento, ruta)?;
    lector.solo_campos(
        campos,
        &["articulo", "nombre", "cantidad", "ancho", "alto", "solo_si"],
        ruta,
    );
    let r = |campo: &str| ruta_hija(ruta, campo);
    let articulo = lector
        .obligatorio(campos, "articulo", ruta)
        .and_then(|v| lector.texto(v, &r("articulo")));
    let mut formula = |campo: &str| {
        let valor = lector.obligatorio(campos, campo, ruta)?;
        lector.formula(valor, &r(campo))
    };
    let (cantidad, ancho, alto) = (formula("cantidad"), formula("ancho"), formula("alto"));
    let nombre = texto_opcional(lector, opcional(campos, "nombre"), &r("nombre"));
    let solo_si = leer_solo_si(lector, opcional(campos, "solo_si"), &r("solo_si"));
    Some(PiezaDeChapa {
        articulo: articulo?,
        nombre: nombre?,
        cantidad: cantidad?,
        ancho: ancho?,
        alto: alto?,
        solo_si: solo_si?,
    })
}

/// `parteluces: { solo_si, abajo, altura }`: solo el dibujo (ADR-0030).
pub fn leer_parteluces(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Option<ParteLucesDeSerie>> {
    let Some(valor) = valor else {
        return Some(None);
    };
    let campos = lector.mapa(valor, ruta)?;
    lector.solo_campos(campos, &["solo_si", "abajo", "altura"], ruta);
    let r = |campo: &str| ruta_hija(ruta, campo);
    let solo_si = leer_solo_si(lector, opcional(campos, "solo_si"), &r("solo_si"));
    let abajo = lector
        .obligatorio(campos, "abajo", ruta)
        .and_then(|v| lector.formula(v, &r("abajo")));
    let altura = lector
        .obligatorio(campos, "altura", ruta)
        .and_then(|v| lector.formula(v, &r("altura")));
    let Dato::Valor(abajo) = abajo? else {
        lector
            .informe
            .error(&r("abajo"), "la medida de abajo no admite CONFIRMAR");
        return None;
    };
    Some(Some(ParteLucesDeSerie {
        solo_si: solo_si?,
        abajo,
        altura: altura?,
    }))
}

fn texto_opcional(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Option<String>> {
    match valor {
        None => Some(None),
        Some(valor) => lector.texto(valor, ruta).map(Some),
    }
}
