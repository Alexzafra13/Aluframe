//! Dibujo técnico de una chapa (ADR-0030): el rectángulo a escala, rayado como en los planos,
//! con la cota del ancho debajo y la del alto a la derecha. Una medida que falta sale como «?».

use std::fmt::Write;

use crate::unidades::Longitud;

const ANCHO: i64 = 200;
const ALTO: i64 = 146;
/// Caja donde cabe la chapa, dejando sitio a las cotas.
const CAJA: (i64, i64, i64, i64) = (12, 8, 140, 84);
/// Proporción por defecto (ancho / alto × 100) mientras falte una medida.
const PROPORCION_SIN_MEDIDA: i64 = 160;

/// Ancho y alto del rectángulo en el dibujo: a escala dentro de la caja.
fn tamano_a_escala(ancho: Option<Longitud>, alto: Option<Longitud>) -> (i64, i64) {
    let (_, _, caja_ancho, caja_alto) = CAJA;
    let proporcion = match (ancho, alto) {
        (Some(a), Some(h)) if h.decimas_de_mm() > 0 => {
            (a.decimas_de_mm() * 100 / h.decimas_de_mm()).clamp(15, 900)
        }
        _ => PROPORCION_SIN_MEDIDA,
    };
    let (w, h) = if proporcion * caja_alto / 100 <= caja_ancho {
        (proporcion * caja_alto / 100, caja_alto)
    } else {
        (caja_ancho, caja_ancho * 100 / proporcion)
    };
    (w.max(6), h.max(6))
}

pub fn dibujar_chapa(ancho: Option<Longitud>, alto: Option<Longitud>) -> String {
    let (x0, y0, _, _) = CAJA;
    let (w, h) = tamano_a_escala(ancho, alto);
    let texto =
        |medida: Option<Longitud>| medida.map_or_else(|| "?".to_owned(), |m| m.formatear_cm());
    let mut s = String::with_capacity(1_024);
    let _ = write!(
        s,
        r##"<svg class="dibujo-chapa" viewBox="0 0 {ANCHO} {ALTO}" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Chapa de {} × {} cm"><defs><pattern id="rayado-pieza" width="9" height="9" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><path d="M0 0 V9" stroke="#5C636B" stroke-width="1.2"/></pattern></defs><rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="#ECE9E0" stroke="#15191D" stroke-width="2"/><rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="url(#rayado-pieza)"/>"##,
        texto(ancho),
        texto(alto)
    );
    cotas(&mut s, (w, h), (texto(ancho), texto(alto)));
    s
}

/// El ancho debajo y el alto a la derecha, con sus líneas de cota.
fn cotas(s: &mut String, (w, h): (i64, i64), (ancho, alto): (String, String)) {
    let (x0, y0, _, _) = CAJA;
    let (ya, xa) = (y0 + h + 14, x0 + w + 14);
    let _ = write!(
        s,
        r##"<path d="M{x0} {ya} H{} M{x0} {} V{} M{} {} V{} M{xa} {y0} V{} M{} {y0} H{} M{} {} H{}" stroke="#5C636B" stroke-width="1.2" fill="none"/>"##,
        x0 + w,
        ya - 5,
        ya + 5,
        x0 + w,
        ya - 5,
        ya + 5,
        y0 + h,
        xa - 5,
        xa + 5,
        xa - 5,
        y0 + h,
        xa + 5
    );
    let (xt, yt) = (xa + 22, y0 + h / 2);
    let _ = write!(
        s,
        r##"<g font-family="IBM Plex Mono, monospace" font-size="24" fill="#15191D" text-anchor="middle"><text x="{}" y="{}">{}</text><text x="{xt}" y="{yt}" transform="rotate(-90 {xt} {yt})">{}</text></g></svg>"##,
        x0 + w / 2,
        ya + 26,
        ancho,
        alto
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dibuja_la_chapa_con_sus_cotas_y_marca_lo_que_falta() {
        let svg = dibujar_chapa(None, Some(Longitud::desde_decimas_de_mm(5_000)));
        assert!(svg.contains(">?<") && svg.contains(">50<"), "{svg}");
        assert!(svg.contains("Chapa de ? × 50 cm"));
    }

    #[test]
    fn la_chapa_se_dibuja_a_escala() {
        let ancho_de = |svg: &str| -> i64 {
            let resto = &svg[svg.find(r#"<rect x="12" y="8" width=""#).unwrap() + 27..];
            resto.split('"').next().unwrap().parse().unwrap()
        };
        let cuadrada = dibujar_chapa(
            Some(Longitud::desde_decimas_de_mm(5_000)),
            Some(Longitud::desde_decimas_de_mm(5_000)),
        );
        let apaisada = dibujar_chapa(
            Some(Longitud::desde_decimas_de_mm(10_000)),
            Some(Longitud::desde_decimas_de_mm(5_000)),
        );
        assert!(ancho_de(&apaisada) > ancho_de(&cuadrada));
    }
}
