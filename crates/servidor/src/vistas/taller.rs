//! Lo que el taller ve de una ventana, en mm (decisión 1 del plan de H4) y sin precios: la
//! pantalla del operario y la orden de taller impresa (design/03-hoja-taller.png).

use askama::Template;
use dominio::texto::nombre_legible;
use dominio::unidades::Longitud;

use super::{UsuarioEnPantalla, VentanaEnPantalla, titulo_y_medidas};
use crate::aplicacion::trabajos::congelado::VentanaCongelada;
use crate::aplicacion::trabajos::consultar::VentanaDeTrabajo;
use crate::bd::consultas::trabajos::FilaTrabajo;

pub fn mm(dmm: i64) -> String {
    Longitud::desde_decimas_de_mm(dmm).formatear_mm()
}

pub struct PiezaEnHoja {
    pub numero: u32,
    pub nombre: String,
    pub cantidad: u32,
    pub medida: String,
    pub corte: String,
}

fn pieza_en_hoja(p: &crate::aplicacion::resumen_de_linea::PiezaResumida) -> PiezaEnHoja {
    PiezaEnHoja {
        numero: p.numero,
        nombre: nombre_legible(&p.perfil),
        cantidad: p.cantidad,
        medida: mm(p.longitud_dmm),
        corte: p.corte.map(|g| format!("{g}°")).unwrap_or_default(),
    }
}

/// La hoja de taller va en mm, como las piezas; lo que falta en la serie sale «?».
fn chapa_en_hoja(c: &crate::aplicacion::resumen_de_linea::ChapaResumida) -> (String, String) {
    let medida = |dmm: Option<i64>| dmm.map_or_else(|| "?".to_owned(), mm);
    let nombre = c
        .nombre
        .clone()
        .unwrap_or_else(|| nombre_legible(&c.articulo));
    let texto = format!(
        "{nombre}: {} × {} × {} mm",
        c.cantidad,
        medida(c.ancho_dmm),
        medida(c.alto_dmm)
    );
    let longitud = |dmm: Option<i64>| dmm.map(dominio::unidades::Longitud::desde_decimas_de_mm);
    let dibujo = dominio::dibujo::dibujar_chapa(longitud(c.ancho_dmm), longitud(c.alto_dmm));
    (texto, dibujo)
}

pub struct HojaDeVentana {
    pub titulo: String,
    pub serie: String,
    pub acabado: String,
    pub medidas: String,
    pub dibujo: String,
    pub piezas: Vec<PiezaEnHoja>,
    /// Vacío mientras la serie no tenga la medida del cristal: se imprime «PENDIENTE de medir».
    pub cristal: Vec<String>,
    /// Chapas del parteluces: (texto, dibujo técnico a escala).
    pub chapas: Vec<(String, String)>,
    pub accesorios: Vec<(String, String)>,
    pub opciones: Vec<String>,
}

impl HojaDeVentana {
    pub fn desde(descripcion: &str, c: &VentanaCongelada) -> HojaDeVentana {
        let r = &c.resumen;
        HojaDeVentana {
            titulo: titulo_y_medidas(descripcion).0,
            serie: nombre_legible(&r.serie),
            acabado: c.acabado.clone(),
            medidas: format!("{} × {}", mm(c.ancho_dmm), mm(c.alto_dmm)),
            dibujo: r.dibujo.clone(),
            piezas: r.piezas.iter().map(pieza_en_hoja).collect(),
            cristal: r
                .vidrios
                .iter()
                .map(|v| {
                    format!(
                        "{}{} × {} × {} mm",
                        v.nombre
                            .as_ref()
                            .map(|n| format!("{n}: "))
                            .unwrap_or_default(),
                        v.cantidad,
                        mm(v.ancho_dmm),
                        mm(v.alto_dmm)
                    )
                })
                .collect(),
            chapas: r.chapas.iter().map(chapa_en_hoja).collect(),
            accesorios: r
                .accesorios
                .iter()
                .map(|(a, n)| (nombre_legible(a), n.clone()))
                .collect(),
            opciones: descripcion
                .split(" · ")
                .filter(|t| t.contains(": "))
                .map(str::to_owned)
                .collect(),
        }
    }
}

pub struct TarjetaVentana {
    pub id: i64,
    pub codigo: String,
    pub cliente: String,
    pub titulo: String,
    pub medidas: String,
    pub dibujo: String,
    pub hechos: usize,
    pub total: usize,
    pub falta_algo: Option<String>,
    pub medida_cambiada: bool,
}

impl TarjetaVentana {
    pub fn de(v: &VentanaDeTrabajo, t: &FilaTrabajo) -> TarjetaVentana {
        TarjetaVentana {
            id: v.fila.id,
            codigo: format!("{} · ventana {}", t.codigo(), v.fila.orden),
            cliente: t.cliente_nombre.clone(),
            titulo: titulo_y_medidas(&v.fila.descripcion).0,
            medidas: format!(
                "{} × {} mm",
                mm(v.congelado.ancho_dmm),
                mm(v.congelado.alto_dmm)
            ),
            dibujo: v.congelado.resumen.dibujo.clone(),
            hechos: v.hechos(),
            total: v.pasos.len(),
            falta_algo: v.fila.falta_algo.clone(),
            medida_cambiada: v.fila.medida_cambiada,
        }
    }
}

#[derive(Template)]
#[template(path = "mi_dia.html")]
pub struct PaginaMiDia {
    pub usuario: UsuarioEnPantalla,
    pub tarjetas: Vec<TarjetaVentana>,
    /// Solo con el permiso de ver todo el taller (ADR-0024).
    pub resto: Vec<TarjetaVentana>,
    /// Las tareas sueltas que le ha mandado el dueño (ADR-0025).
    pub tareas: Vec<super::TarjetaApunte>,
    /// Sus apuntes y tareas sin cerrar, para el contador del botón «Apuntes».
    pub apuntes_pendientes: usize,
}

#[derive(Template)]
#[template(path = "mi_dia_ventana.html")]
pub struct PaginaVentanaOperario {
    pub usuario: UsuarioEnPantalla,
    pub codigo: String,
    pub cliente: String,
    pub ventana: VentanaEnPantalla,
    pub hoja: HojaDeVentana,
    pub anterior: Option<i64>,
    pub siguiente: Option<i64>,
    pub aviso: Option<String>,
}

pub struct Comparacion {
    pub medidas_antes: String,
    pub medidas_despues: String,
    pub antes: Vec<PiezaEnHoja>,
    pub despues: Vec<PiezaEnHoja>,
}

impl Comparacion {
    pub fn de(antes: &VentanaCongelada, despues: &VentanaCongelada) -> Comparacion {
        let a = HojaDeVentana::desde(&antes.descripcion, antes);
        let d = HojaDeVentana::desde(&despues.descripcion, despues);
        Comparacion {
            medidas_antes: a.medidas,
            medidas_despues: d.medidas,
            antes: a.piezas,
            despues: d.piezas,
        }
    }
}

#[derive(Template)]
#[template(path = "medidas.html")]
pub struct PaginaMedidas {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [super::GrupoMenu],
    pub seccion: &'static str,
    pub trabajo: i64,
    pub ventana: i64,
    pub version: i64,
    pub titulo: String,
    pub revision: i64,
    pub ancho: String,
    pub alto: String,
    pub opciones: Vec<super::OpcionEnPantalla>,
    pub motivo: String,
    pub reiniciar: bool,
    pub pasos_hechos: bool,
    pub comparacion: Option<Comparacion>,
    pub aviso: Option<String>,
}
