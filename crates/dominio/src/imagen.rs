//! Logotipo de la empresa: solo PNG o JPEG, reconocidos por sus primeros bytes y no por el
//! nombre del archivo. SVG no se admite: puede llevar código y la hoja se abre en el navegador.

use thiserror::Error;

/// Un logotipo mayor no aporta nada en un A4 y engorda la base y las copias.
pub const TAMANO_MAXIMO: usize = 1024 * 1024;

const FIRMA_PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const FIRMA_JPEG: [u8; 3] = [0xFF, 0xD8, 0xFF];
const ALFABETO: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDeImagen {
    Png,
    Jpeg,
}

impl TipoDeImagen {
    pub fn tipo_mime(self) -> &'static str {
        match self {
            TipoDeImagen::Png => "image/png",
            TipoDeImagen::Jpeg => "image/jpeg",
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErrorDeImagen {
    #[error("el archivo está vacío")]
    Vacia,
    #[error("el logotipo no puede pasar de 1 MB; guárdalo más pequeño")]
    Grande,
    #[error("el logotipo tiene que ser una imagen PNG o JPG")]
    TipoNoAdmitido,
}

pub fn reconocer_imagen(bytes: &[u8]) -> Result<TipoDeImagen, ErrorDeImagen> {
    if bytes.is_empty() {
        return Err(ErrorDeImagen::Vacia);
    }
    if bytes.len() > TAMANO_MAXIMO {
        return Err(ErrorDeImagen::Grande);
    }
    if bytes.starts_with(&FIRMA_PNG) {
        Ok(TipoDeImagen::Png)
    } else if bytes.starts_with(&FIRMA_JPEG) {
        Ok(TipoDeImagen::Jpeg)
    } else {
        Err(ErrorDeImagen::TipoNoAdmitido)
    }
}

/// La imagen como dirección `data:` para incrustarla en la hoja sin pedir otro archivo.
pub fn convertir_en_direccion_de_datos(bytes: &[u8]) -> Result<String, ErrorDeImagen> {
    let tipo = reconocer_imagen(bytes)?;
    Ok(format!(
        "data:{};base64,{}",
        tipo.tipo_mime(),
        codificar_base64(bytes)
    ))
}

/// Base64 estándar con relleno (RFC 4648); son pocas líneas y evitan una dependencia.
pub fn codificar_base64(bytes: &[u8]) -> String {
    let mut texto = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for trozo in bytes.chunks(3) {
        let b = [
            trozo[0],
            trozo.get(1).copied().unwrap_or(0),
            trozo.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= trozo.len() {
                let indice = ((n >> (18 - 6 * i)) & 0x3F) as usize;
                texto.push(char::from(ALFABETO[indice]));
            } else {
                texto.push('=');
            }
        }
    }
    texto
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn codifica_base64_como_la_norma() {
        assert_eq!(codificar_base64(b""), "");
        assert_eq!(codificar_base64(b"f"), "Zg==");
        assert_eq!(codificar_base64(b"fo"), "Zm8=");
        assert_eq!(codificar_base64(b"foo"), "Zm9v");
        assert_eq!(codificar_base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(codificar_base64(&[0xFF, 0xFE]), "//4=");
    }

    #[test]
    fn reconoce_png_y_jpeg_por_sus_primeros_bytes() {
        let mut png = FIRMA_PNG.to_vec();
        png.push(0);
        assert_eq!(reconocer_imagen(&png), Ok(TipoDeImagen::Png));
        assert_eq!(
            reconocer_imagen(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Ok(TipoDeImagen::Jpeg)
        );
        let direccion = convertir_en_direccion_de_datos(&png).unwrap();
        assert!(direccion.starts_with("data:image/png;base64,iVBORw0KGgo"));
    }

    #[test]
    fn rechaza_svg_vacios_y_grandes() {
        assert_eq!(
            reconocer_imagen(b"<svg xmlns='http://www.w3.org/2000/svg'/>"),
            Err(ErrorDeImagen::TipoNoAdmitido)
        );
        assert_eq!(reconocer_imagen(b""), Err(ErrorDeImagen::Vacia));
        let mut grande = FIRMA_PNG.to_vec();
        grande.resize(TAMANO_MAXIMO + 1, 0);
        assert_eq!(reconocer_imagen(&grande), Err(ErrorDeImagen::Grande));
    }
}
