//! El acabado en el SVG (ADR-0019). Va en atributos `fill` y `stroke`, no en `style`: la
//! política de seguridad de la página (CSP) no deja aplicar estilos escritos en el HTML.

use super::ColorDeDibujo;

/// Imitación madera: un patrón con vetas suaves sobre el color base. El identificador lleva el
/// color, así dos dibujos de la misma página no se pisan el patrón aunque sean distintos.
pub fn definiciones(color: &ColorDeDibujo) -> String {
    if !color.madera {
        return String::new();
    }
    let base = color.perfil();
    format!(
        r##"<defs><pattern id="veta-{id}" patternUnits="userSpaceOnUse" width="1600" height="420"><rect width="1600" height="420" fill="{base}"/><path d="M0 70 C400 40 800 110 1600 60 M0 190 C500 230 900 150 1600 200 M0 330 C350 300 1100 370 1600 320" fill="none" stroke="#000" stroke-opacity=".16" stroke-width="14"/><path d="M0 250 C600 270 1000 230 1600 255" fill="none" stroke="#fff" stroke-opacity=".12" stroke-width="10"/></pattern></defs>"##,
        id = identificador(color)
    )
}

/// Atributos del marco: su color (o la veta) y su borde.
pub fn atributos_de_marco(color: &ColorDeDibujo) -> String {
    atributos(color, color.perfil())
}

/// Atributos de una hoja: un punto más clara que el marco (o la misma veta).
pub fn atributos_de_hoja(color: &ColorDeDibujo) -> String {
    atributos(color, &color.hoja())
}

/// Grupo que envuelve cierres, manillas y bisagras: lo heredan de aquí.
pub fn grupo_de_herrajes(color: &ColorDeDibujo) -> String {
    format!(r#"<g class="herrajes" fill="{}">"#, color.herraje())
}

fn atributos(color: &ColorDeDibujo, liso: &str) -> String {
    let relleno = if color.madera {
        format!("url(#veta-{})", identificador(color))
    } else {
        liso.to_owned()
    };
    format!(r#" fill="{relleno}" stroke="{}""#, color.borde())
}

fn identificador(color: &ColorDeDibujo) -> &str {
    color.perfil().trim_start_matches('#')
}
