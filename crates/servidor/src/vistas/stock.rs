//! Pantallas del stock (PANTALLAS 9): saldos con retales, movimientos y lista de la compra.

use askama::Template;
use dominio::stock::formatear_cantidad;
use dominio::unidades::Longitud;

use super::{GrupoMenu, Opcion, UsuarioEnPantalla};
use crate::aplicacion::stock::compra::LineaDeCompra;
use crate::bd::consultas::stock::{FilaMovimiento, FilaSaldo};

/// Un artículo en un color: lo entero (barras o unidades) y sus retales.
pub struct FilaStock {
    pub articulo: i64,
    pub referencia: String,
    pub descripcion: String,
    pub color: String,
    pub unidad: String,
    pub enteras: String,
    pub retales: Vec<String>,
    pub minimo: String,
    pub bajo_minimo: bool,
}

fn cm(dmm: i64) -> String {
    Longitud::desde_decimas_de_mm(dmm).formatear_cm()
}

/// Junta los saldos por artículo y color; el mínimo cuenta sobre lo que no tiene color.
pub fn filas_de_stock(saldos: &[FilaSaldo]) -> Vec<FilaStock> {
    let mut filas: Vec<FilaStock> = Vec::new();
    for s in saldos {
        let color = s.acabado.clone().unwrap_or_else(|| "Sin color".into());
        let misma = filas
            .last()
            .is_some_and(|f| f.articulo == s.articulo_id && f.color == color);
        if !misma {
            filas.push(FilaStock {
                articulo: s.articulo_id,
                referencia: s.referencia.clone(),
                descripcion: s.descripcion.clone(),
                color,
                unidad: s.unidad.clone(),
                enteras: "0".into(),
                retales: Vec::new(),
                minimo: if s.minimo_mil > 0 {
                    formatear_cantidad(s.minimo_mil)
                } else {
                    String::new()
                },
                bajo_minimo: false,
            });
        }
        let Some(fila) = filas.last_mut() else {
            continue;
        };
        match s.longitud_dmm {
            None => {
                fila.enteras = formatear_cantidad(s.cantidad_mil);
                fila.bajo_minimo =
                    s.acabado_id.is_none() && s.minimo_mil > 0 && s.cantidad_mil < s.minimo_mil;
            }
            Some(largo) => fila.retales.push(format!(
                "{} × {} cm",
                formatear_cantidad(s.cantidad_mil),
                cm(largo)
            )),
        }
    }
    filas
}

#[derive(Template)]
#[template(path = "stock.html")]
pub struct PaginaStock {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub filas: Vec<FilaStock>,
    pub solo_bajo: bool,
    pub articulos: Vec<Opcion>,
    pub acabados: Vec<Opcion>,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

pub struct MovimientoEnPantalla {
    pub cuando: String,
    pub tipo: String,
    pub cantidad: String,
    pub detalle: String,
    pub motivo: String,
    pub usuario: String,
}

impl From<&FilaMovimiento> for MovimientoEnPantalla {
    fn from(m: &FilaMovimiento) -> Self {
        let color = m.acabado.clone().unwrap_or_else(|| "sin color".into());
        let detalle = match m.longitud_dmm {
            Some(l) => format!("{color} · retal de {} cm", cm(l)),
            None => color,
        };
        MovimientoEnPantalla {
            cuando: super::fecha_corta(m.cuando_en.get(..10).unwrap_or_default()),
            tipo: m.tipo.clone(),
            cantidad: formatear_cantidad(m.cantidad_mil),
            detalle,
            motivo: m.motivo.clone(),
            usuario: m.usuario.clone(),
        }
    }
}

#[derive(Template)]
#[template(path = "stock_articulo.html")]
pub struct PaginaArticuloStock {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: i64,
    pub referencia: String,
    pub descripcion: String,
    pub unidad: String,
    pub minimo: String,
    pub filas: Vec<FilaStock>,
    pub movimientos: Vec<MovimientoEnPantalla>,
    pub confirmacion: Option<String>,
}

pub struct LineaCompraEnPantalla {
    pub referencia: String,
    pub descripcion: String,
    pub color: String,
    pub unidad: String,
    pub hay: String,
    pub necesitan: String,
    pub minimo: String,
    pub comprar: String,
}

impl From<&LineaDeCompra> for LineaCompraEnPantalla {
    fn from(l: &LineaDeCompra) -> Self {
        LineaCompraEnPantalla {
            referencia: l.referencia.clone(),
            descripcion: l.descripcion.clone(),
            color: l.color.clone().unwrap_or_default(),
            unidad: l.unidad.clone(),
            hay: formatear_cantidad(l.hay_mil),
            necesitan: formatear_cantidad(l.necesitan_mil),
            minimo: formatear_cantidad(l.minimo_mil),
            comprar: formatear_cantidad(l.comprar_mil),
        }
    }
}

#[derive(Template)]
#[template(path = "stock_compra.html")]
pub struct PaginaCompra {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub lineas: Vec<LineaCompraEnPantalla>,
}
