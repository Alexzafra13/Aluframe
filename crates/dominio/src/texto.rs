//! Normalización de textos para buscar y para reconocer cabeceras de archivos.

/// Minúsculas, sin tildes ni diéresis y con los espacios repetidos reducidos a uno:
/// «Peñas  Ávila» y «penas avila» se encuentran igual.
pub fn normalizar_para_buscar(texto: &str) -> String {
    let mut resultado = String::with_capacity(texto.len());
    for palabra in texto.split_whitespace() {
        if !resultado.is_empty() {
            resultado.push(' ');
        }
        resultado.extend(
            palabra
                .chars()
                .flat_map(char::to_lowercase)
                .map(quitar_tilde),
        );
    }
    resultado
}

/// Una cabecera de columna como clave: «Descripción » y «precio tarifa» pasan a
/// `descripcion` y `precio_tarifa`.
pub fn normalizar_clave(texto: &str) -> String {
    normalizar_para_buscar(texto).replace(' ', "_")
}

/// Un texto escrito por el usuario: sin espacios a los lados y vacío como ausente.
pub fn limpiar_opcional(texto: &str) -> Option<String> {
    let limpio = texto.trim();
    (!limpio.is_empty()).then(|| limpio.to_owned())
}

/// Un código de los datos como se lee en pantalla: «MARCO_LATERAL» es «Marco lateral».
pub fn nombre_legible(codigo: &str) -> String {
    let texto = separar_palabras(codigo).to_lowercase();
    let texto = texto
        .split(' ')
        .map(con_tilde)
        .collect::<Vec<_>>()
        .join(" ");
    let mut letras = texto.chars();
    letras
        .next()
        .map(|p| p.to_uppercase().chain(letras).collect())
        .unwrap_or_default()
}

/// Los códigos van sin tildes (ASCII, `CLAUDE.md`); estas palabras del taller la recuperan al
/// mostrarse. Una palabra nueva con tilde en un código se añade aquí.
fn con_tilde(palabra: &str) -> &str {
    match palabra {
        "si" => "sí",
        "cajon" => "cajón",
        "guia" => "guía",
        "camara" => "cámara",
        "angulo" => "ángulo",
        "union" => "unión",
        otra => otra,
    }
}

/// El guion bajo separa palabras, salvo entre dos cifras, donde es la coma decimal que un código
/// no puede llevar: `TORNILLO_4_8x25` es «Tornillo 4,8x25».
fn separar_palabras(codigo: &str) -> String {
    let letras: Vec<char> = codigo.chars().collect();
    let es_cifra = |i: Option<usize>| {
        i.and_then(|i| letras.get(i))
            .is_some_and(char::is_ascii_digit)
    };
    letras
        .iter()
        .enumerate()
        .map(|(i, &c)| match c {
            '_' if es_cifra(i.checked_sub(1)) && es_cifra(Some(i + 1)) => ',',
            '_' => ' ',
            otra => otra,
        })
        .collect()
}

fn quitar_tilde(letra: char) -> char {
    match letra {
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        otra => otra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busca_sin_distinguir_tildes_ni_mayusculas() {
        assert_eq!(normalizar_para_buscar("  Peñas   ÁVILA "), "penas avila");
        assert_eq!(normalizar_para_buscar("Güell"), "guell");
        assert_eq!(normalizar_para_buscar(""), "");
    }

    #[test]
    fn lee_los_codigos_de_los_datos_como_nombres() {
        assert_eq!(nombre_legible("ALUPROM_21"), "Aluprom 21");
        assert_eq!(nombre_legible("climalit"), "Climalit");
        assert_eq!(nombre_legible("si"), "Sí");
        assert_eq!(nombre_legible("GUIA_LATERAL"), "Guía lateral");
        assert_eq!(nombre_legible("CAJON_50"), "Cajón 50");
        assert_eq!(nombre_legible("TORNILLO_4_8x25"), "Tornillo 4,8x25");
        assert_eq!(nombre_legible("CIERRE_4_HOJAS"), "Cierre 4 hojas");
        assert_eq!(nombre_legible(""), "");
    }

    #[test]
    fn convierte_cabeceras_en_claves() {
        assert_eq!(normalizar_clave("Descripción "), "descripcion");
        assert_eq!(normalizar_clave("Precio  Tarifa"), "precio_tarifa");
    }

    #[test]
    fn un_texto_en_blanco_es_ausente() {
        assert_eq!(limpiar_opcional("   "), None);
        assert_eq!(limpiar_opcional(" Zafra "), Some("Zafra".into()));
    }
}
