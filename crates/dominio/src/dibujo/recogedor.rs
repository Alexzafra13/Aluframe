//! La cinta de la persiana y su recogedor (ADR-0032), como en la foto del propietario: la cinta
//! baja por dentro de la guía del lado elegido y entra por arriba en el recogedor, una caja
//! alargada al pie de la guía con un tornillo abajo. La cinta es gris con la ventana blanca y
//! negra con color, para que se distinga.

use std::fmt::Write;

use super::DatosDeDibujo;
use super::geometria::{Lado, Plano, Rect};

const COLOR_CINTA_GRIS: &str = "#8B939B";
const COLOR_CINTA_NEGRA: &str = "#15191D";

pub fn dibujar(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let Some(p) = &plano.persiana else { return };
    let guia = match p.cinta {
        Lado::Derecha => p.guias[1],
        Lado::Izquierda => p.guias[0],
    };
    let caja = caja_en(guia);
    cinta(s, guia, caja.y + guia.ancho / 2, datos);
    recogedor(s, caja, guia.ancho, datos);
}

/// Al pie de la guía; por debajo del número del marco lateral, que va al 72 % del alto.
fn caja_en(guia: Rect) -> Rect {
    let ancho = guia.ancho * 17 / 20;
    let alto = (guia.ancho * 6).min(guia.alto / 4);
    Rect {
        x: guia.x + (guia.ancho - ancho) / 2,
        y: guia.abajo() - guia.ancho / 3 - alto,
        ancho,
        alto,
    }
}

/// Una tira tejida: el color de la cinta con una trama fina encima.
fn cinta(s: &mut String, guia: Rect, hasta: i64, datos: &DatosDeDibujo) {
    let color = if datos.color.es_blanco() {
        COLOR_CINTA_GRIS
    } else {
        COLOR_CINTA_NEGRA
    };
    let ancho = (guia.ancho * 2 / 5).max(1);
    let x = guia.x + (guia.ancho - ancho) / 2;
    let paso = (guia.ancho / 10).max(2);
    let _ = write!(
        s,
        r##"<defs><pattern id="trama-cinta-{paso}" width="{paso}" height="{paso}" patternUnits="userSpaceOnUse"><path d="M0 0 H{paso}" stroke="#fff" stroke-opacity=".35" stroke-width="{}" fill="none"/></pattern></defs><rect class="persiana-cinta" x="{x}" y="{}" width="{ancho}" height="{}" fill="{color}"/><rect x="{x}" y="{}" width="{ancho}" height="{}" fill="url(#trama-cinta-{paso})"/>"##,
        (paso / 3).max(1),
        guia.y,
        hasta - guia.y,
        guia.y,
        hasta - guia.y
    );
}

/// La caja con un canto de luz, la boca oscura por donde entra la cinta y el tornillo.
fn recogedor(s: &mut String, caja: Rect, guia: i64, datos: &DatosDeDibujo) {
    let borde = datos.color.borde();
    let boca = Rect {
        x: caja.x + caja.ancho / 6,
        y: caja.y + guia / 3,
        ancho: caja.ancho * 2 / 3,
        alto: (guia / 5).max(1),
    };
    let _ = write!(
        s,
        r##"<g class="persiana-recogedor"><rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{borde}"/><path d="M{} {} V{}" stroke="#fff" stroke-opacity=".45" stroke-width="2" fill="none"/><rect x="{}" y="{}" width="{}" height="{}" fill="#15191D"/><circle cx="{}" cy="{}" r="{}" fill="#C9CDD1" stroke="#5C636B"/></g>"##,
        caja.x,
        caja.y,
        caja.ancho,
        caja.alto,
        (guia / 12).max(1),
        datos.color.hoja(),
        caja.x + caja.ancho / 8,
        caja.y + guia / 2,
        caja.abajo() - guia / 3,
        boca.x,
        boca.y,
        boca.ancho,
        boca.alto,
        caja.x + caja.ancho / 2,
        caja.abajo() - guia / 2,
        (guia / 8).max(1)
    );
}
