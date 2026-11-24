//! El parteluces en el dibujo (ADR-0030): una barra en cada hoja, la chapa de abajo rayada en
//! el color de la ventana (o el cristal), su medida acotada dentro del hueco y, cuando se sabe,
//! la altura real del parteluces a la derecha.

use std::fmt::Write;

use super::DatosDeDibujo;
use super::acabado::perfil;
use super::geometria::{Plano, Rect};
use crate::unidades::Longitud;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParteLucesEnDibujo {
    /// Medida escrita de la chapa o del cristal de abajo.
    pub abajo: Longitud,
    /// Altura real del eje desde abajo de la ventana; sin ella se aproxima y la cota lo dice.
    pub altura: Option<Longitud>,
    pub con_chapa: bool,
}

/// Altura del eje desde abajo de la ventana, en unidades del plano (décimas de mm).
fn altura_del_eje(plano: &Plano, p: &ParteLucesEnDibujo) -> i64 {
    let aproximada = || {
        p.abajo.decimas_de_mm()
            + plano.grosor_marco
            + plano.grosor_hoja
            + grosor_de_barra(plano) / 2
    };
    p.altura.map_or_else(aproximada, Longitud::decimas_de_mm)
}

/// Coordenada vertical del eje en el plano (para la barra y para el número de su pieza).
pub fn eje(plano: &Plano, p: &ParteLucesEnDibujo) -> i64 {
    plano.marco.abajo() - altura_del_eje(plano, p)
}

fn grosor_de_barra(plano: &Plano) -> i64 {
    (plano.grosor_hoja * 7 / 10).max(1)
}

pub fn dibujar(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let Some(p) = &datos.parteluces else { return };
    let barra = grosor_de_barra(plano);
    let eje = eje(plano, p);
    if p.con_chapa {
        rayado(s, &datos.color.borde(), (plano.grosor_hoja / 3).max(2));
    }
    for hoja in &plano.hojas {
        let hueco = hoja.dentro(plano.grosor_hoja);
        let y = eje.clamp(hueco.y + barra, hueco.abajo() - barra);
        if p.con_chapa {
            let chapa = Rect {
                y: y + barra / 2,
                alto: hueco.abajo() - y - barra / 2,
                ..hueco
            };
            let base = datos.color.hoja();
            let _ = write!(
                s,
                r#"<rect class="chapa" x="{}" y="{}" width="{}" height="{}" fill="{base}"/><rect class="chapa-rayado" x="{}" y="{}" width="{}" height="{}" fill="url(#rayado-chapa)"/>"#,
                chapa.x,
                chapa.y,
                chapa.ancho,
                chapa.alto,
                chapa.x,
                chapa.y,
                chapa.ancho,
                chapa.alto
            );
        }
        let travesano = Rect {
            y: y - barra / 2,
            alto: barra,
            ..hueco
        };
        perfil(s, &datos.color, ("parteluces", travesano), barra);
    }
}

/// Rayado a 45° como en los planos: distingue la chapa del cristal también al imprimir en gris.
fn rayado(s: &mut String, borde: &str, paso: i64) {
    let _ = write!(
        s,
        r#"<defs><pattern id="rayado-chapa" width="{paso}" height="{paso}" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><path d="M0 0 V{paso}" stroke="{borde}" stroke-width="{}" stroke-opacity=".45"/></pattern></defs>"#,
        (paso / 10).max(1)
    );
}

/// Dentro del hueco de abajo de la primera hoja, su medida («chapa 50»): es lo que se corta.
/// A la derecha, solo si la serie la sabe, la altura real del parteluces desde abajo.
pub fn cota(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let Some(p) = &datos.parteluces else { return };
    let Some(hoja) = plano.hojas.first() else {
        return;
    };
    let letra = plano.tamano_de_letra();
    let hueco = hoja.dentro(plano.grosor_hoja);
    let barra = grosor_de_barra(plano);
    let arriba = eje(plano, p).clamp(hueco.y + barra, hueco.abajo() - barra) + barra / 2;
    let abajo = hueco.abajo();
    let x = hueco.x + hueco.ancho / 5;
    let relleno = if p.con_chapa { "chapa" } else { "cristal" };
    let texto = format!("{relleno} {}", p.abajo.formatear_cm());
    let (xt, yt) = (x + letra / 2, (arriba + abajo) / 2 + letra * 35 / 100);
    let ancho_texto = letra * i64::try_from(texto.chars().count()).unwrap_or(10) * 62 / 100;
    let _ = write!(
        s,
        r##"<g class="capa-cotas"><path class="cota" d="M{x} {arriba} V{abajo} M{} {arriba} H{} M{} {abajo} H{}"/><rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="#FBFAF6" fill-opacity=".9"/><text class="texto-cota" x="{xt}" y="{yt}" font-size="{letra}">{texto}</text></g>"##,
        x - letra / 3,
        x + letra / 3,
        x - letra / 3,
        x + letra / 3,
        xt - letra / 5,
        yt - letra,
        ancho_texto + letra * 2 / 5,
        letra * 13 / 10,
        letra / 4,
    );
    if let Some(altura) = p.altura {
        cota_de_altura(s, plano, eje(plano, p), altura);
    }
}

fn cota_de_altura(s: &mut String, plano: &Plano, eje: i64, altura: Longitud) {
    let letra = plano.tamano_de_letra();
    let x = plano.marco.derecha() + plano.margen / 3;
    let abajo = plano.marco.abajo();
    let (xt, yt) = (x + letra, (abajo + eje) / 2);
    let _ = write!(
        s,
        r#"<g class="capa-cotas"><path class="cota" d="M{x} {eje} V{abajo} M{} {eje} H{} M{} {abajo} H{}"/><text class="texto-cota" x="{xt}" y="{yt}" font-size="{letra}" text-anchor="middle" transform="rotate(-90 {xt} {yt})">{}</text></g>"#,
        x - letra / 2,
        x + letra / 2,
        x - letra / 2,
        x + letra / 2,
        altura.formatear_cm()
    );
}
