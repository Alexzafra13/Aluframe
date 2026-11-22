//! Siluetas de los perfiles (`data/perfiles/`), incluidas en el ejecutable como las series
//! (ADR-0017). Solo sirven para reconocer cada pieza en pantalla y en la hoja de taller.

use dominio::series::Serie;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../data/perfiles/"]
struct Siluetas;

/// El contenido de `fabricante/archivo.svg`, si existe. Solo SVG: nada más se sirve de ahí.
pub fn leer_silueta(ruta: &str) -> Option<Vec<u8>> {
    if !ruta.ends_with(".svg") {
        return None;
    }
    Siluetas::get(ruta).map(|archivo| archivo.data.into_owned())
}

/// Dirección de la silueta de un perfil de la serie, o nada si la serie no la da o no existe.
pub fn url_de_silueta(serie: &Serie, perfil: &str) -> Option<String> {
    let silueta = serie.referencias.get(perfil)?.silueta.as_deref()?;
    let archivo = format!("{silueta}.svg");
    Siluetas::get(&archivo).map(|_| format!("/perfiles/{archivo}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solo_sirve_svg_de_la_carpeta_de_perfiles() {
        assert!(leer_silueta("aluval/21210.svg").is_some());
        assert!(leer_silueta("aluval/indice.txt").is_none());
        assert!(leer_silueta("aluval/no-existe.svg").is_none());
    }
}
