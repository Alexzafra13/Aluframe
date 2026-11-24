//! Logotipos de los fabricantes (`data/marcas/`), incluidos en el ejecutable como las siluetas
//! (ADR-0029). Acompañan al nombre de la serie para reconocerla de un vistazo.

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../data/marcas/"]
struct Marcas;

/// El logotipo `archivo.png`, si existe. Solo PNG: nada más se sirve de ahí.
pub fn leer_logo(archivo: &str) -> Option<Vec<u8>> {
    if !archivo.ends_with(".png") || archivo.contains('/') {
        return None;
    }
    Marcas::get(archivo).map(|a| a.data.into_owned())
}

/// Dirección del logotipo del fabricante de una serie (`ALUVAL` → `/marcas/aluval.png`).
pub fn url_de_logo(fabricante: &str) -> Option<String> {
    let archivo = format!("{}.png", fabricante.to_lowercase());
    Marcas::get(&archivo).map(|_| format!("/marcas/{archivo}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_fabricante_con_logotipo_tiene_su_direccion() {
        assert_eq!(url_de_logo("ALUVAL").as_deref(), Some("/marcas/aluval.png"));
        assert_eq!(url_de_logo("SOPENA"), None);
    }

    #[test]
    fn solo_sirve_png_de_la_carpeta_de_marcas() {
        assert!(leer_logo("aluval.png").is_some());
        assert!(leer_logo("aluval.svg").is_none());
        assert!(leer_logo("../perfiles/aluval/21210.png").is_none());
    }
}
