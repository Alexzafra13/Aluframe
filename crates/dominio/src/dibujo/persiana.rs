//! Persiana en la ventana (ADR-0032), como en la foto que aportó el propietario: el cajón arriba
//! a todo el ancho, una guía a cada lado, la persiana medio bajada con las lamas detrás del
//! cristal y la cinta por la guía del lado elegido. Cajón y guías, en el color de la ventana.
//! Suelta en pared (ADR-0033), cajón y guías de L van por fuera del hueco.

use std::fmt::Write;

use super::DatosDeDibujo;
use super::acabado::perfil;
use super::geometria::{Lado, Plano, Rect};
use crate::series::MontajeDePersiana;
use crate::unidades::Longitud;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersianaEnDibujo {
    pub montaje: MontajeDePersiana,
    /// Sin el ancho de la guía (el ala de la L en pared), el dibujo lo aproxima y no lo acota.
    pub guia: Option<Longitud>,
    /// Sin el alto del cajón, el dibujo lo aproxima y no lo acota.
    pub cajon: Option<Longitud>,
    pub cinta: Lado,
    pub ancho_ventana: Longitud,
    pub alto_ventana: Option<Longitud>,
}

/// Dónde va cada parte dentro del contorno (la medida del conjunto).
#[derive(Debug, Clone, Copy)]
pub struct PersianaEnPlano {
    pub cajon: Rect,
    pub guias: [Rect; 2],
    pub cinta: Lado,
}

/// Lo que se ve de persiana bajada: la parte de arriba del hueco, como en la foto.
const BAJADA_POR_CIEN: i64 = 30;

/// Solo para dibujar mientras falte el dato, sin cota: el ala de la guía de L (80 o 100 mm).
const GUIA_APROXIMADA: i64 = 800;

/// Reparte la medida escrita. En la ventana es el conjunto: guías y cajón quedan dentro y el
/// resto es el marco. En pared es el hueco: guías y cajón quedan por fuera y el contorno crece.
/// Devuelve (partes, hueco o marco, contorno).
pub fn repartir(medida: Rect, p: &PersianaEnDibujo) -> (PersianaEnPlano, Rect, Rect) {
    let aproximado = medida.alto * 15 / 100;
    let cajon = p.cajon.map_or(aproximado, Longitud::decimas_de_mm);
    let guia = p.guia.map_or(GUIA_APROXIMADA, Longitud::decimas_de_mm);
    let (contorno, guia, cajon) = match p.montaje {
        MontajeDePersiana::Ventana => (
            medida,
            guia.clamp(1, medida.ancho / 4),
            cajon.clamp(1, medida.alto / 2),
        ),
        MontajeDePersiana::Pared => {
            let (guia, cajon) = (guia.clamp(1, medida.ancho), cajon.clamp(1, medida.alto));
            let contorno = Rect {
                ancho: medida.ancho + 2 * guia,
                alto: medida.alto + cajon,
                ..medida
            };
            (contorno, guia, cajon)
        }
    };
    let guia_en = |x| Rect {
        x,
        y: contorno.y + cajon,
        ancho: guia,
        alto: contorno.alto - cajon,
    };
    let dentro = Rect {
        x: contorno.x + guia,
        y: contorno.y + cajon,
        ancho: contorno.ancho - 2 * guia,
        alto: contorno.alto - cajon,
    };
    let partes = PersianaEnPlano {
        cajon: Rect {
            alto: cajon,
            ..contorno
        },
        guias: [guia_en(contorno.x), guia_en(contorno.derecha() - guia)],
        cinta: p.cinta,
    };
    (partes, dentro, contorno)
}

/// Cajón y guías, antes que el marco: la ventana va entre ellos. Las tapas del cajón tienen el
/// ancho de las guías y caen sobre ellas, así se leen como una sola pieza.
pub fn dibujar_cajon_y_guias(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let Some(p) = &plano.persiana else { return };
    let color = &datos.color;
    for guia in p.guias {
        s.push_str(r#"<g class="persiana-guia">"#);
        perfil(s, color, ("marco", guia), guia.ancho);
        s.push_str("</g>");
    }
    s.push_str(r#"<g class="persiana-cajon">"#);
    perfil(s, color, ("marco", p.cajon), p.cajon.alto / 4);
    let borde = color.borde();
    let registro = p.cajon.y + p.cajon.alto * 72 / 100;
    let _ = write!(
        s,
        r#"<path d="M{} {registro} H{}" fill="none" stroke="{borde}" stroke-opacity=".55"/>"#,
        p.guias[0].derecha(),
        p.guias[1].x
    );
    for guia in p.guias {
        let _ = write!(
            s,
            r#"<rect class="persiana-tapa" x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{borde}"/>"#,
            guia.x,
            p.cajon.y,
            guia.ancho,
            p.cajon.alto,
            color.hoja()
        );
    }
    s.push_str("</g>");
}

/// Las lamas se ven detrás del cristal de cada hoja, en la parte de arriba.
pub fn dibujar_lamas(s: &mut String, plano: &Plano) {
    if plano.persiana.is_none() {
        return;
    }
    let bajada = plano.marco.y + plano.marco.alto * BAJADA_POR_CIEN / 100;
    let paso = (plano.marco.alto / 25).max(1);
    s.push_str(r#"<g class="persiana-lamas">"#);
    for hoja in &plano.hojas {
        let cristal = hoja.dentro(plano.grosor_hoja);
        let alto = bajada.min(cristal.abajo()) - cristal.y;
        if alto <= 0 {
            continue;
        }
        let _ = write!(
            s,
            r##"<rect x="{}" y="{}" width="{}" height="{alto}" fill="#ECEDEE" stroke="#8B939B"/>"##,
            cristal.x, cristal.y, cristal.ancho
        );
        let mut d = String::new();
        let mut y = cristal.y + paso;
        while y < cristal.y + alto {
            let _ = write!(d, "M{} {y} H{}", cristal.x, cristal.derecha());
            y += paso;
        }
        let _ = write!(
            s,
            r##"<path d="{d}" fill="none" stroke="#8B939B" stroke-opacity=".7"/><path d="M{} {} H{}" fill="none" stroke="#15191D" stroke-width="2"/>"##,
            cristal.x,
            cristal.y + alto,
            cristal.derecha()
        );
    }
    s.push_str("</g>");
}
