//! Color con el que se dibuja la ventana (ADR-0019): el del acabado elegido, blanco si no hay.

/// Color de los cierres y manillas: blancos con la ventana blanca y negros con cualquier color,
/// que es lo que se monta en el taller (propietario, 2026-11-22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Herrajes {
    Blanco,
    Negro,
}

impl Herrajes {
    pub fn desde_texto(texto: &str) -> Option<Herrajes> {
        match texto {
            "blanco" => Some(Herrajes::Blanco),
            "negro" => Some(Herrajes::Negro),
            _ => None,
        }
    }

    pub fn como_texto(self) -> &'static str {
        match self {
            Herrajes::Blanco => "blanco",
            Herrajes::Negro => "negro",
        }
    }

    fn hex(self) -> &'static str {
        match self {
            Herrajes::Blanco => "#f4f4f2",
            Herrajes::Negro => "#15191d",
        }
    }
}

/// Cómo está acabada la superficie: cambia el reflejo en el dibujo (y, más adelante, el precio).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Superficie {
    Brillo,
    Mate,
    Texturado,
    Anodizado,
}

impl Superficie {
    pub fn desde_texto(texto: &str) -> Option<Superficie> {
        match texto {
            "brillo" => Some(Superficie::Brillo),
            "mate" => Some(Superficie::Mate),
            "texturado" => Some(Superficie::Texturado),
            "anodizado" => Some(Superficie::Anodizado),
            _ => None,
        }
    }

    pub fn como_texto(self) -> &'static str {
        match self {
            Superficie::Brillo => "brillo",
            Superficie::Mate => "mate",
            Superficie::Texturado => "texturado",
            Superficie::Anodizado => "anodizado",
        }
    }

    pub fn inicial(self) -> char {
        match self {
            Superficie::Brillo => 'b',
            Superficie::Mate => 'm',
            Superficie::Texturado => 't',
            Superficie::Anodizado => 'a',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorDeDibujo {
    /// `#rrggbb` en minúsculas (ya validado con `leer_color_hex`).
    perfil: String,
    /// Imitación madera: se dibuja con veta.
    pub madera: bool,
    pub herrajes: Herrajes,
    pub superficie: Superficie,
}

/// Blanco RAL 9016 (brillo), el que se usa por defecto en el taller.
pub const BLANCO_9016: &str = "#f2f2ed";

impl Default for ColorDeDibujo {
    fn default() -> Self {
        ColorDeDibujo {
            perfil: BLANCO_9016.to_owned(),
            madera: false,
            herrajes: Herrajes::Blanco,
            superficie: Superficie::Brillo,
        }
    }
}

impl ColorDeDibujo {
    /// `None` si el color no es `#rrggbb`.
    pub fn nuevo(perfil: &str, madera: bool, herrajes: Herrajes) -> Option<ColorDeDibujo> {
        Some(ColorDeDibujo {
            perfil: leer_color_hex(perfil)?,
            madera,
            herrajes,
            superficie: Superficie::Brillo,
        })
    }

    pub fn con_superficie(self, superficie: Superficie) -> ColorDeDibujo {
        ColorDeDibujo { superficie, ..self }
    }

    pub fn perfil(&self) -> &str {
        &self.perfil
    }

    pub fn herraje(&self) -> &str {
        self.herrajes.hex()
    }

    /// Las hojas, un punto más claras que el marco, para distinguirlas en el dibujo.
    pub fn hoja(&self) -> String {
        mezclar(&self.perfil, "#ffffff", 88)
    }

    /// Borde de marco y hojas: el mismo tono, oscurecido (en negro sale gris, para que se vea).
    pub fn borde(&self) -> String {
        mezclar(&self.perfil, "#5c636b", 55)
    }
}

/// `peso` % del primer color y el resto del segundo. Ambos ya son `#rrggbb` válidos.
fn mezclar(a: &str, b: &str, peso: u32) -> String {
    let canal = |hex: &str, i: usize| {
        hex.get(1 + 2 * i..3 + 2 * i)
            .and_then(|c| u32::from_str_radix(c, 16).ok())
            .unwrap_or(0)
    };
    let mezcla: Vec<String> = (0..3)
        .map(|i| (canal(a, i) * peso + canal(b, i) * (100 - peso) + 50) / 100)
        .map(|v| format!("{v:02x}"))
        .collect();
    format!("#{}", mezcla.concat())
}

/// Acepta `#RRGGBB` (mayúsculas o minúsculas) y lo devuelve en minúsculas.
pub fn leer_color_hex(texto: &str) -> Option<String> {
    let cifras = texto.trim().strip_prefix('#')?;
    let valido = cifras.len() == 6 && cifras.bytes().all(|b| b.is_ascii_hexdigit());
    valido.then(|| format!("#{}", cifras.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solo_acepta_colores_de_seis_cifras_hexadecimales() {
        assert_eq!(leer_color_hex("#0F4336").as_deref(), Some("#0f4336"));
        for malo in ["0f4336", "#0f43", "#0f43zz", "rojo", "#0f43366", "url(#x)"] {
            assert_eq!(leer_color_hex(malo), None, "{malo}");
        }
    }

    #[test]
    fn por_defecto_es_blanco_con_cierres_blancos() {
        let color = ColorDeDibujo::default();
        assert_eq!(color.perfil(), BLANCO_9016);
        assert_eq!(color.herrajes, Herrajes::Blanco);
        assert_eq!(Herrajes::desde_texto("negro"), Some(Herrajes::Negro));
        assert_eq!(Herrajes::desde_texto("dorado"), None);
    }

    #[test]
    fn las_hojas_son_algo_mas_claras_y_el_borde_algo_mas_oscuro() {
        let negro = ColorDeDibujo::nuevo("#000000", false, Herrajes::Negro).unwrap();
        assert_eq!(negro.hoja(), "#1f1f1f");
        assert_eq!(negro.borde(), "#292d30");
        assert_eq!(mezclar("#ffffff", "#000000", 100), "#ffffff");
    }
}
