//! Mosquitera enrollable (ADR-0031), como en la vista frontal del catálogo: cajón arriba, dos
//! guías, la malla y el terminal con su tirador. Los perfiles van en el color de la ventana.
//! Las medidas de los perfiles se agrandan un poco para que se vean en ventanas grandes.

use std::fmt::Write;

use super::DatosDeDibujo;
use super::acabado::{grupo_de_herrajes, perfil};
use super::geometria::{Plano, Rect};

/// Cajón, guía y terminal en el dibujo (décimas de mm): el real (45 o 50, 25 y 30 mm) o, en
/// mosquiteras grandes, una fracción de la medida menor para que no desaparezcan.
struct Proporciones {
    cajon: i64,
    guia: i64,
    terminal: i64,
    tapa: i64,
}

impl Proporciones {
    fn de(plano: &Plano, datos: &DatosDeDibujo) -> Proporciones {
        let menor = plano.marco.ancho.min(plano.marco.alto);
        let cajon_45 = (menor / 9).max(450);
        Proporciones {
            cajon: if datos.cajon_de_50 {
                cajon_45 * 10 / 9
            } else {
                cajon_45
            },
            guia: (menor / 20).max(250),
            terminal: (menor / 14).max(300),
            tapa: (menor / 40).max(80),
        }
    }
}

/// Dónde va cada parte de la mosquitera dentro del contorno.
struct Partes {
    cajon: Rect,
    guias: [Rect; 2],
    terminal: Rect,
    malla: Rect,
}

impl Partes {
    fn de(m: Rect, p: &Proporciones) -> Partes {
        let guia = |x| Rect {
            x,
            y: m.y + p.cajon,
            ancho: p.guia,
            alto: m.alto - p.cajon,
        };
        let cajon = Rect { alto: p.cajon, ..m };
        let terminal = Rect {
            x: m.x + p.guia,
            y: m.abajo() - p.tapa - p.terminal,
            ancho: m.ancho - 2 * p.guia,
            alto: p.terminal,
        };
        let malla = Rect {
            y: cajon.abajo(),
            alto: terminal.y - cajon.abajo(),
            ..terminal
        };
        Partes {
            cajon,
            guias: [guia(m.x), guia(m.derecha() - p.guia)],
            terminal,
            malla,
        }
    }
}

pub fn dibujar(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let p = Proporciones::de(plano, datos);
    let partes = Partes::de(plano.marco, &p);
    let color = &datos.color;
    dibujar_malla(s, partes.malla, &color.borde(), (p.guia / 3).max(20));
    for guia in partes.guias {
        perfil(s, color, ("marco", guia), p.guia);
    }
    perfil(s, color, ("marco", partes.cajon), p.cajon);
    perfil(s, color, ("terminal", partes.terminal), p.terminal);
    tapas(s, (partes.cajon, partes.guias), p.tapa);
    s.push_str(&grupo_de_herrajes(color));
    tirador(s, partes.terminal);
    s.push_str("</g>");
}

fn tirador(s: &mut String, terminal: Rect) {
    let _ = write!(
        s,
        r#"<rect class="manilla" x="{}" y="{}" width="{}" height="{}" rx="{}"/>"#,
        terminal.x + terminal.ancho / 2 - terminal.ancho / 10,
        terminal.y + terminal.alto / 4,
        terminal.ancho / 5,
        terminal.alto / 2,
        terminal.alto / 4
    );
}

/// La malla: un fondo gris claro con una trama fina, como una tela mosquitera.
fn dibujar_malla(s: &mut String, malla: Rect, borde: &str, paso: i64) {
    let _ = write!(
        s,
        r##"<defs><pattern id="trama-malla" width="{paso}" height="{paso}" patternUnits="userSpaceOnUse"><path d="M0 0 H{paso} M0 0 V{paso}" stroke="{borde}" stroke-width="{}" stroke-opacity=".5" fill="none"/></pattern></defs><rect class="malla" x="{}" y="{}" width="{}" height="{}" fill="#C9CDD1"/><rect x="{}" y="{}" width="{}" height="{}" fill="url(#trama-malla)"/>"##,
        (paso / 8).max(1),
        malla.x,
        malla.y,
        malla.ancho,
        malla.alto,
        malla.x,
        malla.y,
        malla.ancho,
        malla.alto
    );
}

/// Las tapas de plástico: a los lados del cajón y al pie de las guías.
fn tapas(s: &mut String, (cajon, guias): (Rect, [Rect; 2]), tapa: i64) {
    let mut piezas = vec![
        Rect {
            ancho: tapa,
            ..cajon
        },
        Rect {
            x: cajon.derecha() - tapa,
            ancho: tapa,
            ..cajon
        },
    ];
    piezas.extend(guias.iter().map(|g| Rect {
        y: g.abajo() - tapa,
        alto: tapa,
        ..*g
    }));
    for t in piezas {
        let _ = write!(
            s,
            r##"<rect class="tapa" x="{}" y="{}" width="{}" height="{}" fill="#5C636B"/>"##,
            t.x, t.y, t.ancho, t.alto
        );
    }
}
