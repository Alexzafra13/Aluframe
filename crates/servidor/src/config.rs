use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

pub const PUERTO_POR_DEFECTO: u16 = 9090;
pub const NOMBRE_ARCHIVO_CONFIGURACION: &str = "alumframe.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modo {
    Servidor,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Configuracion {
    pub modo: Modo,
    pub puerto: u16,
    pub carpeta_datos: PathBuf,
    pub abrir_navegador: Option<bool>,
    pub permitir_datos_demo: bool,
    /// Carpetas donde buscar, además de las unidades y montajes habituales, un disco con la
    /// carpeta `AlumFrame-copias` (por ejemplo, un disco montado en `/srv/copias`).
    pub discos_de_copia: Vec<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ErrorConfiguracion {
    #[error("No se pudo leer {ruta}: {causa}")]
    Lectura {
        ruta: PathBuf,
        causa: std::io::Error,
    },
    #[error("El archivo de configuración no es válido: {0}")]
    Formato(#[from] toml::de::Error),
    #[error("La variable {variable} no es válida: «{valor}»")]
    Variable {
        variable: &'static str,
        valor: String,
    },
}

impl Default for Configuracion {
    fn default() -> Self {
        Configuracion {
            modo: modo_por_defecto(),
            puerto: PUERTO_POR_DEFECTO,
            carpeta_datos: PathBuf::from("datos"),
            abrir_navegador: None,
            permitir_datos_demo: false,
            discos_de_copia: Vec::new(),
        }
    }
}

fn modo_por_defecto() -> Modo {
    if cfg!(windows) {
        Modo::Local
    } else {
        Modo::Servidor
    }
}

impl Configuracion {
    pub fn desde_toml(texto: &str) -> Result<Configuracion, ErrorConfiguracion> {
        Ok(toml::from_str(texto)?)
    }

    pub fn cargar(
        archivo: &Path,
        leer_variable: impl Fn(&str) -> Option<String>,
    ) -> Result<Configuracion, ErrorConfiguracion> {
        let base = match std::fs::read_to_string(archivo) {
            Ok(texto) => Configuracion::desde_toml(&texto)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Configuracion::default(),
            Err(causa) => {
                return Err(ErrorConfiguracion::Lectura {
                    ruta: archivo.to_path_buf(),
                    causa,
                });
            }
        };
        base.aplicar_variables(leer_variable)
    }

    pub fn aplicar_variables(
        mut self,
        leer_variable: impl Fn(&str) -> Option<String>,
    ) -> Result<Configuracion, ErrorConfiguracion> {
        if let Some(valor) = leer_variable("ALUMFRAME_MODO") {
            self.modo = match valor.as_str() {
                "servidor" => Modo::Servidor,
                "local" => Modo::Local,
                _ => return Err(variable_no_valida("ALUMFRAME_MODO", valor)),
            };
        }
        if let Some(valor) = leer_variable("ALUMFRAME_PUERTO") {
            self.puerto = valor
                .parse()
                .map_err(|_| variable_no_valida("ALUMFRAME_PUERTO", valor))?;
        }
        if let Some(valor) = leer_variable("ALUMFRAME_CARPETA_DATOS") {
            self.carpeta_datos = PathBuf::from(valor);
        }
        if let Some(valor) = leer_variable("ALUMFRAME_ABRIR_NAVEGADOR") {
            self.abrir_navegador = Some(valor == "si");
        }
        if let Some(valor) = leer_variable("ALUMFRAME_DATOS_DEMO") {
            self.permitir_datos_demo = valor == "si";
        }
        if let Some(valor) = leer_variable("ALUMFRAME_DISCOS_COPIA") {
            self.discos_de_copia = std::env::split_paths(&valor).collect();
        }
        Ok(self)
    }

    pub fn direccion_de_escucha(&self) -> IpAddr {
        match self.modo {
            Modo::Servidor => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            Modo::Local => IpAddr::V4(Ipv4Addr::LOCALHOST),
        }
    }

    pub fn debe_abrir_navegador(&self) -> bool {
        self.abrir_navegador.unwrap_or(self.modo == Modo::Local)
    }

    pub fn ruta_base_de_datos(&self) -> PathBuf {
        self.carpeta_datos.join("alumframe.db")
    }

    pub fn carpeta_copias(&self) -> PathBuf {
        self.carpeta_datos.join("copias")
    }
}

fn variable_no_valida(variable: &'static str, valor: String) -> ErrorConfiguracion {
    ErrorConfiguracion::Variable { variable, valor }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sin_variables(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn usa_el_puerto_9090_por_defecto() {
        assert_eq!(Configuracion::default().puerto, 9090);
    }

    #[test]
    fn lee_el_archivo_toml() {
        let config = Configuracion::desde_toml(
            "modo = \"local\"\npuerto = 8000\ncarpeta_datos = \"/srv/alumframe\"",
        )
        .unwrap();
        assert_eq!(config.modo, Modo::Local);
        assert_eq!(config.puerto, 8000);
        assert_eq!(config.carpeta_datos, PathBuf::from("/srv/alumframe"));
    }

    #[test]
    fn rechaza_claves_desconocidas() {
        assert!(Configuracion::desde_toml("puertto = 1").is_err());
    }

    #[test]
    fn las_variables_de_entorno_mandan_sobre_el_archivo() {
        let config = Configuracion::default()
            .aplicar_variables(|nombre| (nombre == "ALUMFRAME_PUERTO").then(|| "7000".into()))
            .unwrap();
        assert_eq!(config.puerto, 7000);
    }

    #[test]
    fn rechaza_un_puerto_no_numerico() {
        let resultado = Configuracion::default()
            .aplicar_variables(|nombre| (nombre == "ALUMFRAME_PUERTO").then(|| "abc".into()));
        assert!(resultado.is_err());
    }

    #[test]
    fn el_modo_local_solo_escucha_en_este_equipo_y_abre_el_navegador() {
        let config = Configuracion {
            modo: Modo::Local,
            ..Configuracion::default()
        }
        .aplicar_variables(sin_variables)
        .unwrap();
        assert_eq!(
            config.direccion_de_escucha(),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
        assert!(config.debe_abrir_navegador());
    }

    #[test]
    fn el_modo_servidor_escucha_en_la_red_local_sin_abrir_navegador() {
        let config = Configuracion {
            modo: Modo::Servidor,
            ..Configuracion::default()
        };
        assert_eq!(
            config.direccion_de_escucha(),
            IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        );
        assert!(!config.debe_abrir_navegador());
    }
}
