//! Persiana suelta en pared (ADR-0033), como el plano que aportó el propietario: el cajón por
//! fuera, encima del hueco; las guías de L a los lados, atornilladas a la pared; la persiana
//! bajada entera, con sus lamas y la lama final con el asa; y la cinta con su recogedor en la
//! guía del lado elegido. Todo en el color elegido.

use std::fmt::Write;

use super::DatosDeDibujo;
use super::geometria::{Plano, Rect};

pub fn dibujar(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let Some(p) = &plano.persiana else { return };
    lamas(s, plano.marco, datos);
    super::persiana::dibujar_cajon_y_guias(s, plano, datos);
    for guia in p.guias {
        tornillos(s, guia);
    }
    super::recogedor::dibujar(s, plano, datos);
}

/// Unas 18 lamas en el alto, como en el plano: es un esquema, no cuenta las lamas reales.
fn paso_de_lama(hueco: Rect) -> i64 {
    (hueco.alto / 18).max(1)
}

/// Las lamas llenan el hueco; abajo, la lama final, algo más alta, con el asa.
fn lamas(s: &mut String, hueco: Rect, datos: &DatosDeDibujo) {
    let paso = paso_de_lama(hueco);
    let final_alto = paso * 3 / 2;
    let fondo = datos.color.hoja();
    let borde = datos.color.borde();
    let _ = write!(
        s,
        r#"<g class="persiana-lamas"><rect x="{}" y="{}" width="{}" height="{}" fill="{fondo}" stroke="{borde}"/>"#,
        hueco.x, hueco.y, hueco.ancho, hueco.alto
    );
    let (mut juntas, mut brillos) = (String::new(), String::new());
    let brillo = (paso / 8).max(1);
    let mut y = hueco.y + paso;
    while y < hueco.abajo() - final_alto {
        let _ = write!(juntas, "M{} {y} H{}", hueco.x, hueco.derecha());
        let _ = write!(brillos, "M{} {} H{}", hueco.x, y + brillo, hueco.derecha());
        y += paso;
    }
    // Cada junta, una línea oscura con un brillo debajo: así se lee el relieve de la lama.
    let _ = write!(
        s,
        r##"<path d="{juntas}" fill="none" stroke="{borde}" stroke-width="1.5"/><path d="{brillos}" fill="none" stroke="#fff" stroke-opacity=".6"/>"##
    );
    let final_y = hueco.abajo() - final_alto;
    let _ = write!(
        s,
        r#"<rect x="{}" y="{final_y}" width="{}" height="{final_alto}" fill="{}" stroke="{borde}"/></g>"#,
        hueco.x,
        hueco.ancho,
        datos.color.perfil()
    );
    asa(s, hueco, (final_y, final_alto));
}

/// El asa de la lama final, en el centro, como en el plano.
fn asa(s: &mut String, hueco: Rect, (y, alto): (i64, i64)) {
    let ancho = (hueco.ancho / 8).max(1);
    let alto_asa = (alto / 2).max(1);
    let _ = write!(
        s,
        r##"<rect class="persiana-asa" x="{}" y="{}" width="{ancho}" height="{alto_asa}" rx="{}" fill="#5C636B" stroke="#15191D"/>"##,
        hueco.x + (hueco.ancho - ancho) / 2,
        y + (alto - alto_asa) / 2,
        alto_asa / 2
    );
}

/// Dos tornillos por guía, cerca de los extremos: la guía de L se atornilla a la pared.
fn tornillos(s: &mut String, guia: Rect) {
    let radio = (guia.ancho / 10).max(1);
    let x = guia.x + guia.ancho / 2;
    for y in [guia.y + guia.ancho / 2, guia.abajo() - guia.ancho / 2] {
        let _ = write!(
            s,
            r##"<circle class="tornillo" cx="{x}" cy="{y}" r="{radio}" fill="#C9CDD1" stroke="#5C636B"/>"##
        );
    }
}
