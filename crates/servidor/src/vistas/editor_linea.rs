use askama::Template;
use dominio::series::OpcionDeLinea;
use dominio::unidades::Longitud;

use super::{GrupoMenu, Opcion, UsuarioEnPantalla};
use crate::aplicacion::calculo_de_ventana::CalculoDeVentana;

/// «MARCO_LATERAL» se lee «Marco lateral», como en la hoja de taller.
pub fn nombre_legible(codigo: &str) -> String {
    let texto = codigo.replace('_', " ").to_lowercase();
    let mut letras = texto.chars();
    letras
        .next()
        .map(|p| p.to_uppercase().chain(letras).collect())
        .unwrap_or_default()
}

pub struct PestanaDeLinea {
    pub orden: i64,
    pub titulo: String,
    pub completa: bool,
}

pub struct OpcionEnPantalla {
    pub codigo: String,
    pub nombre: String,
    pub valores: Vec<String>,
    pub elegido: String,
}

impl OpcionEnPantalla {
    pub fn de(opcion: &OpcionDeLinea, elegido: Option<&String>) -> OpcionEnPantalla {
        OpcionEnPantalla {
            codigo: opcion.codigo.clone(),
            nombre: opcion.nombre.clone(),
            valores: opcion.valores.clone(),
            elegido: elegido
                .cloned()
                .unwrap_or_else(|| opcion.por_defecto.clone()),
        }
    }
}

/// Lo que se escribe en el editor, tal cual, para volver a mostrarlo sin perder nada.
#[derive(Debug, Clone, Default)]
pub struct ValoresDeVentana {
    pub seleccion: String,
    pub ancho: String,
    pub alto: String,
    pub cantidad: String,
    pub acabado: String,
    pub vidrio: String,
    pub mano_de_obra: String,
    pub descuento: String,
    pub iva: String,
    pub limites_revisados: bool,
}

pub struct PiezaEnPantalla {
    pub numero: u32,
    pub nombre: String,
    pub cantidad: u32,
    pub medida: String,
    pub corte: String,
}

#[derive(Default)]
pub struct VistaCalculo {
    pub dibujo: String,
    pub piezas: Vec<PiezaEnPantalla>,
    pub vidrios: Vec<String>,
    pub faltan: Vec<String>,
    pub fuera_de_limites: Vec<String>,
    pub perfiles: String,
    pub alternativa: String,
    pub cristal: String,
    pub accesorios: String,
    pub mano_de_obra: String,
    pub precio: String,
    pub completa: bool,
}

fn cm(dmm: i64) -> String {
    Longitud::desde_decimas_de_mm(dmm).formatear_cm()
}

fn pieza_en_pantalla(p: &dominio::series::PiezaCalculada) -> PiezaEnPantalla {
    PiezaEnPantalla {
        numero: p.numero,
        nombre: nombre_legible(&p.perfil),
        cantidad: p.cantidad,
        medida: p.longitud.formatear_cm(),
        corte: p.corte.map_or_else(|| "—".into(), |g| format!("{g}°")),
    }
}

fn vidrios(c: &CalculoDeVentana) -> Vec<String> {
    let piezas = c.resultado.vidrio.iter().flatten();
    piezas
        .map(|v| {
            format!(
                "{} × {} × {} cm",
                v.cantidad,
                cm(v.ancho.decimas_de_mm()),
                cm(v.alto.decimas_de_mm())
            )
        })
        .collect()
}

impl From<&CalculoDeVentana> for VistaCalculo {
    fn from(c: &CalculoDeVentana) -> Self {
        let m = &c.materiales;
        let alternativa = match (m.perfiles_por_barras, m.perfiles_por_metros) {
            (Some(b), Some(me)) => format!(
                "barras enteras {} · por metros {}",
                b.formatear(),
                me.formatear()
            ),
            _ => String::new(),
        };
        VistaCalculo {
            dibujo: c.dibujo.clone(),
            piezas: c.resultado.piezas.iter().map(pieza_en_pantalla).collect(),
            vidrios: vidrios(c),
            faltan: c.precio.faltan.clone(),
            fuera_de_limites: c.resultado.fuera_de_limites.clone(),
            perfiles: m.perfiles.formatear(),
            alternativa,
            cristal: m.cristal.formatear(),
            accesorios: m.accesorios.formatear(),
            mano_de_obra: String::new(),
            precio: c.precio.precio.formatear(),
            completa: c.precio.es_final(),
        }
    }
}

#[derive(Template)]
#[template(path = "editor_linea.html")]
pub struct PaginaEditorLinea {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: i64,
    pub codigo: String,
    pub cliente: String,
    pub orden: i64,
    pub pestanas: Vec<PestanaDeLinea>,
    pub series: Vec<Opcion>,
    pub acabados: Vec<Opcion>,
    pub vidrios: Vec<Opcion>,
    pub ivas: Vec<Opcion>,
    pub opciones: Vec<OpcionEnPantalla>,
    pub valores: ValoresDeVentana,
    pub calculo: Option<VistaCalculo>,
    pub aviso: Option<String>,
}

impl PaginaEditorLinea {
    pub fn accion(&self) -> String {
        if self.orden > 0 {
            format!("/presupuestos/{}/lineas/{}", self.id, self.orden)
        } else {
            format!("/presupuestos/{}/lineas/nueva", self.id)
        }
    }
}

#[derive(Template)]
#[template(path = "editor_calculo.html")]
pub struct FragmentoCalculo {
    pub calculo: Option<VistaCalculo>,
    pub aviso: Option<String>,
    pub opciones: Option<Vec<OpcionEnPantalla>>,
}
