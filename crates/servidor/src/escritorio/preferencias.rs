//! Dónde trabaja la aplicación de escritorio: en este equipo o contra el servidor del taller
//! (ADR-0015). Se guarda en un archivo pequeño junto a los datos de la aplicación.

use std::path::Path;

use serde::Deserialize;

pub const ARCHIVO_DE_PREFERENCIAS: &str = "escritorio.toml";
const PUERTO_DEL_SERVIDOR: u16 = crate::config::PUERTO_POR_DEFECTO;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conexion {
    EsteEquipo,
    Servidor,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Preferencias {
    /// Sin elegir todavía: la aplicación pregunta al abrirse.
    pub conexion: Option<Conexion>,
    /// La última dirección escrita, aunque ahora se trabaje en este equipo.
    pub servidor: Option<String>,
}

#[derive(Deserialize)]
struct EnArchivo {
    conexion: Option<String>,
    servidor: Option<String>,
}

impl Preferencias {
    /// Un archivo dañado o ausente equivale a no haber elegido: se vuelve a preguntar.
    pub fn cargar(ruta: &Path) -> Preferencias {
        let Ok(texto) = std::fs::read_to_string(ruta) else {
            return Preferencias::default();
        };
        let Ok(leido) = toml::from_str::<EnArchivo>(&texto) else {
            return Preferencias::default();
        };
        let servidor = leido.servidor.and_then(|s| validar_direccion(&s).ok());
        let conexion = match leido.conexion.as_deref() {
            Some("este_equipo") => Some(Conexion::EsteEquipo),
            Some("servidor") if servidor.is_some() => Some(Conexion::Servidor),
            _ => None,
        };
        Preferencias { conexion, servidor }
    }

    pub fn guardar(&self, ruta: &Path) -> std::io::Result<()> {
        let mut texto = String::new();
        match self.conexion {
            Some(Conexion::EsteEquipo) => texto.push_str("conexion = \"este_equipo\"\n"),
            Some(Conexion::Servidor) => texto.push_str("conexion = \"servidor\"\n"),
            None => {}
        }
        // `validar_direccion` ya descarta comillas y barras: se puede escribir tal cual.
        if let Some(servidor) = &self.servidor {
            texto.push_str(&format!("servidor = \"{servidor}\"\n"));
        }
        std::fs::write(ruta, texto)
    }

    /// La dirección a la que conectarse, si se eligió trabajar contra el servidor.
    pub fn servidor_elegido(&self) -> Option<&str> {
        match self.conexion {
            Some(Conexion::Servidor) => self.servidor.as_deref(),
            _ => None,
        }
    }
}

/// Admite lo que escribiría el dueño («192.168.1.100», «http://taller:9090/») y devuelve una
/// dirección completa sin barra final; sin puerto se usa el del servidor (9090).
pub fn validar_direccion(texto: &str) -> Result<String, String> {
    let limpio = texto.trim();
    let prohibido = |c: char| c.is_whitespace() || c.is_control() || "\"'\\<>`".contains(c);
    if limpio.is_empty() || limpio.chars().any(prohibido) {
        return Err("Escribe la dirección del servidor, por ejemplo 192.168.1.100".into());
    }
    let (esquema, resto) = match limpio.split_once("://") {
        Some((esquema, resto)) => (esquema.to_ascii_lowercase(), resto),
        None => ("http".to_owned(), limpio),
    };
    let resto = resto.trim_end_matches('/');
    if !matches!(esquema.as_str(), "http" | "https") {
        return Err("La dirección debe empezar por http:// o https://".into());
    }
    let anfitrion = resto.split('/').next().unwrap_or_default();
    if anfitrion.is_empty() || anfitrion.starts_with(':') {
        return Err("Falta el nombre o la IP del servidor.".into());
    }
    let con_puerto = anfitrion.contains(':') || anfitrion.contains('[');
    let puerto = if con_puerto {
        String::new()
    } else {
        format!(":{PUERTO_DEL_SERVIDOR}")
    };
    let ruta = resto.strip_prefix(anfitrion).unwrap_or_default();
    Ok(format!("{esquema}://{anfitrion}{puerto}{ruta}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completa_la_direccion_que_escribiria_el_dueno() {
        assert_eq!(
            validar_direccion("192.168.1.100").unwrap(),
            "http://192.168.1.100:9090"
        );
        assert_eq!(
            validar_direccion(" http://taller:8080/ ").unwrap(),
            "http://taller:8080"
        );
        assert_eq!(
            validar_direccion("HTTPS://alumframe.ts.net").unwrap(),
            "https://alumframe.ts.net:9090"
        );
    }

    #[test]
    fn rechaza_direcciones_vacias_raras_o_con_comillas() {
        for mala in [
            "",
            "   ",
            "ftp://x",
            "http://",
            "http://:9090",
            "a\"b",
            "a b",
        ] {
            assert!(validar_direccion(mala).is_err(), "{mala}");
        }
    }

    #[test]
    fn guarda_y_vuelve_a_leer_la_eleccion() {
        let ruta = std::env::temp_dir().join(format!("pref-{}.toml", std::process::id()));
        let preferencias = Preferencias {
            conexion: Some(Conexion::Servidor),
            servidor: Some("http://192.168.1.100:9090".into()),
        };
        preferencias.guardar(&ruta).unwrap();
        assert_eq!(Preferencias::cargar(&ruta), preferencias);
        assert_eq!(
            Preferencias::cargar(&ruta).servidor_elegido(),
            Some("http://192.168.1.100:9090")
        );
        let _ = std::fs::remove_file(ruta);
    }

    #[test]
    fn un_archivo_danado_o_ausente_vuelve_a_preguntar() {
        let ruta = std::env::temp_dir().join(format!("pref-mala-{}.toml", std::process::id()));
        std::fs::write(&ruta, "conexion = [").unwrap();
        assert_eq!(Preferencias::cargar(&ruta).conexion, None);
        let _ = std::fs::remove_file(&ruta);
        assert_eq!(Preferencias::cargar(&ruta), Preferencias::default());
    }
}
