//! Orden de taller impresa: una hoja A4 por ventana y el plan de corte del trabajo (H4).

use std::collections::BTreeMap;

use askama::Template;
use dominio::corte::BarraCortada;
use dominio::taller::BarrasDePerfil;
use dominio::texto::nombre_legible;
use dominio::unidades::Longitud;

use super::{HojaDeVentana, fecha_corta, mm};
use crate::aplicacion::trabajos::orden::OrdenDeTaller;

pub struct HojaImpresa {
    pub orden: i64,
    pub revision: i64,
    pub revisada: String,
    pub hoja: HojaDeVentana,
}

pub struct BarraDibujada {
    pub svg: String,
    pub texto: String,
}

pub struct GrupoDeCorte {
    pub titulo: String,
    pub barras: Vec<BarraDibujada>,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "orden_imprimir.html")]
pub struct PaginaOrden {
    pub id: i64,
    pub codigo: String,
    pub cliente: String,
    pub direccion: String,
    pub hojas: Vec<HojaImpresa>,
    pub grupos: Vec<GrupoDeCorte>,
    pub ajustes: String,
}

/// La barra a escala en SVG (la página no admite estilos en línea): cada pieza con su ventana y
/// número del dibujo, y lo que sobra en blanco.
fn dibujar_barra(barra: &BarraCortada, grupo: &BarrasDePerfil) -> String {
    const ANCHO: f64 = 1000.0;
    let total = grupo.longitud_barra.decimas_de_mm().max(1) as f64;
    let mut x = 0.0;
    let mut partes = String::new();
    for pieza in &barra.piezas {
        let ancho = pieza.longitud.decimas_de_mm() as f64 / total * ANCHO;
        let (ventana, numero) = grupo.origen.get(&pieza.id).copied().unwrap_or_default();
        partes.push_str(&format!(
            "<rect x=\"{x:.1}\" y=\"1\" width=\"{ancho:.1}\" height=\"46\" fill=\"#E3EDF6\" stroke=\"#15191D\" stroke-width=\"1.5\"/>\
             <text x=\"{:.1}\" y=\"20\" text-anchor=\"middle\" font-size=\"13\" fill=\"#15191D\">V{ventana}·{numero}</text>\
             <text x=\"{:.1}\" y=\"38\" text-anchor=\"middle\" font-size=\"12\" fill=\"#5C636B\">{}</text>",
            x + ancho / 2.0,
            x + ancho / 2.0,
            mm(pieza.longitud.decimas_de_mm())
        ));
        x += ancho;
    }
    format!(
        "<svg class=\"barra-corte\" viewBox=\"0 0 1000 48\" role=\"img\" aria-label=\"Barra con {} piezas\">\
         <rect x=\"0\" y=\"1\" width=\"1000\" height=\"46\" fill=\"#fff\" stroke=\"#15191D\" stroke-width=\"1.5\"/>{partes}</svg>",
        barra.piezas.len()
    )
}

fn texto_de_barra(barra: &BarraCortada, (retal_minimo, retal): (Longitud, Option<i64>)) -> String {
    let piezas: Vec<String> = barra
        .piezas
        .iter()
        .map(|p| mm(p.longitud.decimas_de_mm()))
        .collect();
    let tipo = if barra.sobrante >= retal_minimo && barra.sobrante > Longitud::CERO {
        "retal"
    } else {
        "desecho"
    };
    let origen = retal.map_or_else(String::new, |l| {
        format!("Coger el retal de {} del stock: ", mm(l))
    });
    format!(
        "{origen}{} · sobrante {} ({tipo})",
        piezas.join(" + "),
        mm(barra.sobrante.decimas_de_mm())
    )
}

fn titulo_con_cuentas(titulo: &str, plan: &dominio::corte::PlanDeCorte) -> String {
    let nuevas = match plan.barras_nuevas() {
        1 => "1 barra nueva".to_owned(),
        n => format!("{n} barras nuevas"),
    };
    match plan.numero_de_barras() - plan.barras_nuevas() {
        0 => format!("{titulo} · {nuevas}"),
        1 => format!("{titulo} · {nuevas} y 1 retal del stock"),
        r => format!("{titulo} · {nuevas} y {r} retales del stock"),
    }
}

fn grupo(barras: &BarrasDePerfil, largos: &BTreeMap<u32, i64>) -> GrupoDeCorte {
    let titulo = format!(
        "{} · {} · barra de {}",
        nombre_legible(&barras.perfil),
        barras.color,
        mm(barras.longitud_barra.decimas_de_mm())
    );
    match &barras.plan {
        Ok(plan) => GrupoDeCorte {
            titulo: titulo_con_cuentas(&titulo, plan),
            barras: plan
                .barras
                .iter()
                .map(|b| BarraDibujada {
                    svg: dibujar_barra(b, barras),
                    texto: texto_de_barra(
                        b,
                        (
                            plan.retal_minimo,
                            b.retal.and_then(|id| largos.get(&id).copied()),
                        ),
                    ),
                })
                .collect(),
            error: None,
        },
        Err(error) => GrupoDeCorte {
            titulo,
            barras: Vec::new(),
            error: Some(error.to_string()),
        },
    }
}

impl PaginaOrden {
    pub fn de(orden: &OrdenDeTaller) -> PaginaOrden {
        let t = &orden.ficha.trabajo;
        let a = &orden.ajustes;
        PaginaOrden {
            id: t.id,
            codigo: t.codigo(),
            cliente: t.cliente_nombre.clone(),
            direccion: t.direccion_montaje.clone().unwrap_or_default(),
            hojas: orden
                .ficha
                .ventanas
                .iter()
                .map(|v| HojaImpresa {
                    orden: v.fila.orden,
                    revision: v.fila.revision,
                    revisada: v
                        .fila
                        .revisada_en
                        .as_deref()
                        .map(|f| fecha_corta(f.get(..10).unwrap_or(f)))
                        .unwrap_or_default(),
                    hoja: HojaDeVentana::desde(&v.fila.descripcion, &v.congelado),
                })
                .collect(),
            grupos: orden
                .plan
                .iter()
                .map(|g| grupo(g, &orden.enlace.largo_de_retal))
                .collect(),
            ajustes: format!(
                "Grosor de sierra {} mm · recorte de extremos {} mm · retal desde {} mm",
                mm(a.grosor_sierra.decimas_de_mm()),
                mm(a.recorte_extremos.decimas_de_mm()),
                mm(a.retal_minimo.decimas_de_mm())
            ),
        }
    }

    pub fn total(&self) -> usize {
        self.hojas.len()
    }
}
