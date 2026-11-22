//! Ajustes > Copias (H7).

use askama::Template;
use dominio::copias::MomentoLocal;

use super::{GrupoMenu, UsuarioEnPantalla};
use crate::aplicacion::copias::pantalla::{CopiaGuardada, EstadoDeLasCopias, LugarDeCopias};
use crate::fecha::{formatear_para_pantalla, leer_utc};

/// Días desde el 1-1-1970 al día juliano de `time`.
const DIA_JULIANO_DE_1970: i64 = 2_440_588;

pub struct CopiaEnPantalla {
    pub nombre: String,
    pub cuando: String,
    pub tamano: String,
}

pub struct LugarEnPantalla {
    pub titulo: String,
    pub disco: String,
    pub copias: Vec<CopiaEnPantalla>,
}

#[derive(Template)]
#[template(path = "ajustes_copias.html")]
pub struct PaginaCopias {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub pestana: &'static str,
    pub ultima: String,
    pub discos_de_la_ultima: String,
    pub aviso: Option<String>,
    pub lugares: Vec<LugarEnPantalla>,
    pub sin_preparar: Vec<String>,
    pub confirmacion: Option<String>,
    pub error: Option<String>,
}

impl PaginaCopias {
    pub fn nueva(
        usuario: UsuarioEnPantalla,
        estado: &EstadoDeLasCopias,
        (confirmacion, error): (Option<String>, Option<String>),
    ) -> PaginaCopias {
        let registro = &estado.registro;
        PaginaCopias {
            usuario,
            menu: super::MENU,
            seccion: "ajustes",
            pestana: "copias",
            ultima: registro
                .ultima_completa
                .as_deref()
                .and_then(leer_utc)
                .map_or_else(|| "ninguna todavía".to_owned(), formatear_para_pantalla),
            discos_de_la_ultima: registro.discos.join(", "),
            aviso: registro.aviso.clone(),
            lugares: estado.lugares.iter().map(lugar_en_pantalla).collect(),
            sin_preparar: estado.sin_preparar.clone(),
            confirmacion,
            error,
        }
    }
}

fn lugar_en_pantalla(lugar: &LugarDeCopias) -> LugarEnPantalla {
    LugarEnPantalla {
        titulo: if lugar.disco.is_empty() {
            "En este equipo".to_owned()
        } else {
            format!("En el disco {}", lugar.disco)
        },
        disco: lugar.disco.clone(),
        copias: lugar.copias.iter().map(copia_en_pantalla).collect(),
    }
}

fn copia_en_pantalla(copia: &CopiaGuardada) -> CopiaEnPantalla {
    CopiaEnPantalla {
        nombre: copia.nombre.clone(),
        cuando: momento_legible(copia.momento),
        tamano: tamano_legible(copia.bytes),
    }
}

fn momento_legible(momento: MomentoLocal) -> String {
    let fecha = i32::try_from(momento.dias + DIA_JULIANO_DE_1970)
        .ok()
        .and_then(|juliano| time::Date::from_julian_day(juliano).ok());
    let (hora, minuto) = (momento.minuto / 60, momento.minuto % 60);
    fecha.map_or_else(String::new, |f| {
        format!(
            "{:02}/{:02}/{} {hora:02}:{minuto:02}",
            f.day(),
            u8::from(f.month()),
            f.year()
        )
    })
}

fn tamano_legible(bytes: u64) -> String {
    let kb = bytes.div_ceil(1024);
    if kb < 1024 {
        format!("{kb} KB")
    } else {
        let decimas = kb * 10 / 1024;
        format!("{},{} MB", decimas / 10, decimas % 10)
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn muestra_la_fecha_y_el_tamano_en_espanol() {
        let momento = MomentoLocal::nuevo((2026, 11, 22), (14, 5));
        assert_eq!(momento_legible(momento), "22/11/2026 14:05");
        assert_eq!(tamano_legible(300 * 1024), "300 KB");
        assert_eq!(tamano_legible(1536 * 1024), "1,5 MB");
    }
}
