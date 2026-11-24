//! Cotas del dibujo: el ancho abajo y el alto a la izquierda. Con persiana (ADR-0032), fuera va
//! la medida del conjunto y dentro la del marco, que es lo que se corta.

use std::fmt::Write;

use super::DatosDeDibujo;
use super::geometria::Plano;

pub fn escribir(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let c = plano.contorno;
    let m = plano.margen;
    let letra = plano.tamano_de_letra();
    s.push_str(r#"<g class="capa-cotas">"#);
    let fila_del_conjunto = match &datos.persiana {
        Some(_) => c.abajo() + m * 2 / 3,
        None => c.abajo() + m / 3,
    };
    let ancho = datos.ancho.formatear_cm();
    horizontal(s, (c.x, c.derecha(), fila_del_conjunto), &ancho, letra);
    vertical(
        s,
        (c.y, c.abajo(), c.x - m / 3),
        &datos.alto.formatear_cm(),
        letra,
    );
    if let Some(p) = &datos.persiana {
        let marco = plano.marco;
        let texto = format!("marco {}", p.ancho_ventana.formatear_cm());
        horizontal(
            s,
            (marco.x, marco.derecha(), c.abajo() + m / 3),
            &texto,
            letra,
        );
        if let Some(alto) = p.alto_ventana {
            let x = c.derecha() + m * 6 / 10;
            let texto = format!("marco {}", alto.formatear_cm());
            vertical(s, (marco.y, marco.abajo(), x), &texto, letra);
        }
    }
    s.push_str("</g>");
}

fn horizontal(s: &mut String, (desde, hasta, y): (i64, i64, i64), texto: &str, letra: i64) {
    let medio = letra / 2;
    let _ = write!(
        s,
        r#"<path class="cota" d="M{desde} {y} H{hasta} M{desde} {} V{} M{hasta} {} V{}"/><text class="texto-cota" x="{}" y="{}" font-size="{letra}" text-anchor="middle">{texto}</text>"#,
        y - medio,
        y + medio,
        y - medio,
        y + medio,
        (desde + hasta) / 2,
        y + letra * 13 / 10
    );
}

fn vertical(s: &mut String, (desde, hasta, x): (i64, i64, i64), texto: &str, letra: i64) {
    let medio = letra / 2;
    let (xt, yt) = (x - medio, (desde + hasta) / 2);
    let _ = write!(
        s,
        r#"<path class="cota" d="M{x} {desde} V{hasta} M{} {desde} H{} M{} {hasta} H{}"/><text class="texto-cota" x="{xt}" y="{yt}" font-size="{letra}" text-anchor="middle" transform="rotate(-90 {xt} {yt})">{texto}</text>"#,
        x - medio,
        x + medio,
        x - medio,
        x + medio
    );
}
