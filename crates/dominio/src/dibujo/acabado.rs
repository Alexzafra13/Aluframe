//! El acabado en el SVG (ADR-0019): cada perfil (marco u hoja) se pinta por capas para que se
//! parezca al aluminio de verdad: sombra, color o veta, textura, luz según la superficie (brillo,
//! mate, anodizado) y la línea del rebaje. Todo va en atributos: la política de seguridad de la
//! página (CSP) no deja aplicar estilos escritos en el HTML.

use std::fmt::Write;

use super::ColorDeDibujo;
use super::color::Superficie;
use super::geometria::Rect;

/// Degradados y patrones del acabado. Sus identificadores llevan el color y la superficie: dos
/// dibujos distintos en la misma página no se pisan.
pub fn definiciones(color: &ColorDeDibujo) -> String {
    let id = identificador(color);
    let mut s = format!("<defs>{}", luz(color.superficie, &id));
    if color.madera {
        let _ = write!(s, "{}{}", veta(color, &id, false), veta(color, &id, true));
    }
    if color.superficie == Superficie::Texturado {
        let _ = write!(
            s,
            r##"<pattern id="textura-{id}" patternUnits="userSpaceOnUse" width="90" height="90"><circle cx="12" cy="18" r="7" fill="#000" fill-opacity=".10"/><circle cx="58" cy="9" r="6" fill="#fff" fill-opacity=".14"/><circle cx="37" cy="52" r="8" fill="#000" fill-opacity=".08"/><circle cx="76" cy="66" r="6" fill="#fff" fill-opacity=".12"/><circle cx="20" cy="78" r="5" fill="#000" fill-opacity=".09"/></pattern>"##
        );
    }
    s.push_str("</defs>");
    s
}

/// Un perfil por capas. `clase` es `marco` u `hoja`; el marco además proyecta sombra.
pub fn perfil(s: &mut String, color: &ColorDeDibujo, (clase, r): (&str, Rect), grosor: i64) {
    let id = identificador(color);
    let es_hoja = clase == "hoja";
    if !es_hoja {
        let sombra = Rect {
            y: r.y + (grosor / 5).max(1),
            ..r
        };
        rect(
            s,
            "sombra",
            sombra,
            r##" fill="#15191d" fill-opacity=".12""##,
        );
    }
    let base = match (color.madera, es_hoja) {
        (true, false) => format!("url(#veta-{id}-h)"),
        (true, true) => format!("url(#veta-{id}-v)"),
        (false, false) => color.perfil().to_owned(),
        (false, true) => color.hoja(),
    };
    let borde = color.borde();
    rect(s, clase, r, &format!(r#" fill="{base}" stroke="{borde}""#));
    if color.superficie == Superficie::Texturado {
        rect(s, "textura", r, &format!(r#" fill="url(#textura-{id})""#));
    }
    rect(s, "luz", r, &format!(r#" fill="url(#luz-{id})""#));
    let rebaje = r.dentro((grosor * 2 / 5).max(1));
    let linea = format!(r#" fill="none" stroke="{borde}" stroke-opacity=".55""#);
    rect(s, "rebaje", rebaje, &linea);
}

/// Cómo refleja la luz: el brillo tiene un reflejo marcado arriba, el mate casi nada y el
/// anodizado bandas metálicas.
fn luz(superficie: Superficie, id: &str) -> String {
    let paradas: &[(u8, &str, &str)] = match superficie {
        Superficie::Brillo => &[
            (0, "#fff", ".30"),
            (30, "#fff", ".06"),
            (100, "#000", ".12"),
        ],
        Superficie::Mate => &[(0, "#fff", ".14"), (100, "#000", ".08")],
        Superficie::Texturado => &[(0, "#fff", ".10"), (100, "#000", ".10")],
        Superficie::Anodizado => &[
            (0, "#fff", ".45"),
            (30, "#000", ".10"),
            (55, "#fff", ".30"),
            (80, "#000", ".14"),
            (100, "#fff", ".20"),
        ],
    };
    let mut s = format!(r#"<linearGradient id="luz-{id}" x1="0" y1="0" x2="0" y2="1">"#);
    for (punto, color, opacidad) in paradas {
        let _ = write!(
            s,
            r#"<stop offset="{punto}%" stop-color="{color}" stop-opacity="{opacidad}"/>"#
        );
    }
    s.push_str("</linearGradient>");
    s
}

/// Vetas de madera sobre el color base: en el marco van en horizontal y en las hojas en
/// vertical, así se distinguen como en una ventana de verdad.
fn veta(color: &ColorDeDibujo, id: &str, vertical: bool) -> String {
    let (sufijo, giro, base) = if vertical {
        ("v", r#" patternTransform="rotate(90)""#, color.hoja())
    } else {
        ("h", "", color.perfil().to_owned())
    };
    format!(
        r##"<pattern id="veta-{id}-{sufijo}" patternUnits="userSpaceOnUse" width="2400" height="520"{giro}><rect width="2400" height="520" fill="{base}"/><path d="M0 60 C500 20 900 110 1500 55 S2100 30 2400 70 M0 150 C400 190 1000 120 1700 165 S2200 140 2400 150 M0 290 C600 250 1100 330 1800 280 S2250 300 2400 290 M0 420 C500 450 1200 390 1900 430 S2300 410 2400 420" fill="none" stroke="#2a160a" stroke-opacity=".30" stroke-width="30"/><path d="M0 105 C700 80 1300 140 2400 100 M0 220 C800 250 1500 190 2400 225 M0 360 C600 330 1400 390 2400 355 M0 480 C900 500 1600 460 2400 485" fill="none" stroke="#2a160a" stroke-opacity=".16" stroke-width="70"/><path d="M0 200 C700 215 1500 180 2400 205 M0 330 C900 345 1500 315 2400 335" fill="none" stroke="#fff" stroke-opacity=".12" stroke-width="50"/></pattern>"##
    )
}

fn rect(s: &mut String, clase: &str, r: Rect, extra: &str) {
    let _ = write!(
        s,
        r#"<rect class="{clase}" x="{}" y="{}" width="{}" height="{}"{extra}/>"#,
        r.x, r.y, r.ancho, r.alto
    );
}

fn identificador(color: &ColorDeDibujo) -> String {
    let madera = if color.madera { "m" } else { "" };
    format!(
        "{}{}{madera}",
        color.perfil().trim_start_matches('#'),
        color.superficie.inicial()
    )
}

/// Grupo que envuelve cierres, manillas y bisagras: lo heredan de aquí.
pub fn grupo_de_herrajes(color: &ColorDeDibujo) -> String {
    format!(r#"<g class="herrajes" fill="{}">"#, color.herraje())
}
