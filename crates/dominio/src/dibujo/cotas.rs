//! Cotas del dibujo: el ancho abajo y el alto a la izquierda, la medida total. En el dibujo del
//! taller, con persiana en la ventana (ADR-0032) se acota además el marco, que es lo que se corta;
//! en pared (ADR-0033), el hueco y, cuando se sabe, el total con guías y cajón.

use std::fmt::Write;

use super::geometria::Plano;
use super::{DatosDeDibujo, PersianaEnDibujo};
use crate::series::MontajeDePersiana;

pub fn escribir(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    s.push_str(r#"<g class="capa-cotas">"#);
    match datos
        .persiana
        .as_ref()
        .filter(|p| p.montaje == MontajeDePersiana::Pared)
    {
        Some(p) => de_persiana_en_pared(s, plano, (datos, p)),
        None => de_ventana(s, plano, datos),
    }
    s.push_str("</g>");
}

/// Sin persiana, la medida escrita. Con persiana en la ventana, además la del marco.
fn de_ventana(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let c = plano.contorno;
    let m = plano.margen;
    let letra = plano.tamano_de_letra();
    let con_marco = datos.para_taller && datos.persiana.is_some();
    let fila_del_conjunto = if con_marco {
        c.abajo() + m * 2 / 3
    } else {
        c.abajo() + m / 3
    };
    let ancho = datos.ancho.formatear_cm();
    horizontal(s, (c.x, c.derecha(), fila_del_conjunto), &ancho, letra);
    vertical(
        s,
        (c.y, c.abajo(), c.x - m / 3),
        &datos.alto.formatear_cm(),
        letra,
    );
    if let Some(p) = datos.persiana.as_ref().filter(|_| con_marco) {
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
}

/// En pared (ADR-0033), dentro el hueco, que es lo que se escribe; fuera el total con guías y
/// cajón, solo cuando la serie los sepa.
fn de_persiana_en_pared(
    s: &mut String,
    plano: &Plano,
    (datos, p): (&DatosDeDibujo, &PersianaEnDibujo),
) {
    let (c, hueco) = (plano.contorno, plano.marco);
    let (m, letra) = (plano.margen, plano.tamano_de_letra());
    let texto = |l: crate::unidades::Longitud| format!("hueco {}", l.formatear_cm());
    let fila = c.abajo() + m / 3;
    horizontal(
        s,
        (hueco.x, hueco.derecha(), fila),
        &texto(datos.ancho),
        letra,
    );
    vertical(
        s,
        (hueco.y, hueco.abajo(), c.x - m / 3),
        &texto(datos.alto),
        letra,
    );
    let Some(guia) = p.guia.filter(|_| datos.para_taller) else {
        return;
    };
    if let Some(total) = datos.ancho.sumar(guia).and_then(|a| a.sumar(guia)) {
        let fila = c.abajo() + m * 2 / 3;
        horizontal(s, (c.x, c.derecha(), fila), &total.formatear_cm(), letra);
    }
    if let Some(total) = p.cajon.and_then(|cajon| datos.alto.sumar(cajon)) {
        let x = c.derecha() + m * 6 / 10;
        vertical(s, (c.y, c.abajo(), x), &total.formatear_cm(), letra);
    }
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
