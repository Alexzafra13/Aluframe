//! Hoja A4 de la factura y del albarán, con la misma cabecera que el presupuesto. Una factura
//! emitida se imprime desde su fotografía congelada; un borrador, con sus datos actuales.

use askama::Template;
use dominio::stock::formatear_cantidad;
use dominio::unidades::{Porcentaje, PrecioUnitario};
use serde_json::Value;

use super::{euros, fecha_corta};
use crate::aplicacion::datos_de_empresa::CabeceraDeEmpresa;

pub struct LineaFacturaImpresa {
    pub titulo: String,
    pub detalles: Vec<String>,
    pub cantidad: String,
    pub precio: String,
    pub descuento: String,
    pub importe: String,
}

#[derive(Template)]
#[template(path = "factura_imprimir.html")]
pub struct PaginaFacturaImpresa {
    pub id: i64,
    pub tipo: &'static str,
    pub codigo: String,
    pub fecha: String,
    pub borrador: bool,
    pub rectifica: String,
    pub motivo: String,
    pub empresa: CabeceraDeEmpresa,
    pub cliente: Vec<String>,
    pub lineas: Vec<LineaFacturaImpresa>,
    pub bases: Vec<(String, String, String)>,
    pub descuento: String,
    pub base: String,
    pub total: String,
    pub vencimiento: String,
    pub notas: String,
}

fn texto(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_owned()
}

fn entero(v: &Value) -> i64 {
    v.as_i64().unwrap_or_default()
}

fn linea(l: &Value) -> LineaFacturaImpresa {
    let descripcion = texto(&l["descripcion"]);
    let mut tramos = descripcion.split(" · ").map(str::to_owned);
    let descuento = entero(&l["descuento_bp"]);
    LineaFacturaImpresa {
        titulo: tramos.next().unwrap_or_default(),
        detalles: tramos.collect(),
        cantidad: formatear_cantidad(entero(&l["cantidad_mil"])),
        precio: PrecioUnitario::desde_diezmilesimas(entero(&l["precio_unitario_pu"])).formatear(),
        descuento: if descuento == 0 {
            String::new()
        } else {
            Porcentaje::desde_puntos_basicos(descuento).formatear(false)
        },
        importe: euros(entero(&l["base_cent"])),
    }
}

fn lista(v: &Value) -> Vec<String> {
    v.as_array().into_iter().flatten().map(texto).collect()
}

fn empresa_de(e: &Value, logo: Option<String>) -> CabeceraDeEmpresa {
    CabeceraDeEmpresa {
        logo,
        titulo: texto(&e["titulo"]),
        razon_social: e["razon_social"].as_str().map(str::to_owned),
        lineas: lista(&e["lineas"]),
        iban: e["iban"].as_str().map(str::to_owned),
        pie: e["pie"].as_str().map(str::to_owned),
    }
}

fn cliente_de(c: &Value) -> Vec<String> {
    let mut cliente = vec![texto(&c["nombre"])];
    if let Some(nif) = c["nif"].as_str() {
        cliente.push(format!("NIF {nif}"));
    }
    if let Some(t) = c["telefono"].as_str() {
        cliente.push(format!("Tel. {t}"));
    }
    let direccion = lista(&c["direccion"]);
    if !direccion.is_empty() {
        cliente.push(direccion.join(", "));
    }
    cliente
}

fn bases_de(t: &Value) -> Vec<(String, String, String)> {
    t["bases"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|b| {
            let tipo = Porcentaje::desde_puntos_basicos(entero(&b["tipo_bp"])).formatear(false);
            (
                tipo,
                euros(entero(&b["base_cent"])),
                euros(entero(&b["cuota_cent"])),
            )
        })
        .collect()
}

fn porcentaje_o_nada(bp: i64) -> String {
    if bp == 0 {
        String::new()
    } else {
        Porcentaje::desde_puntos_basicos(bp).formatear(false)
    }
}

/// Todo sale de la fotografía (o de su equivalente para un borrador).
pub fn hoja_desde_fotografia(
    id: i64,
    foto: &Value,
    (borrador, logo): (bool, Option<String>),
    (vencimiento, notas): (String, String),
) -> PaginaFacturaImpresa {
    let t = &foto["totales"];
    let rectificativa = texto(&foto["tipo"]) == "rectificativa";
    PaginaFacturaImpresa {
        id,
        tipo: if rectificativa {
            "Factura rectificativa"
        } else {
            "Factura"
        },
        codigo: texto(&foto["codigo"]),
        fecha: fecha_corta(&texto(&foto["fecha"])),
        borrador,
        rectifica: texto(&foto["rectifica"]),
        motivo: texto(&foto["motivo_rectificacion"]),
        empresa: empresa_de(&foto["empresa"], logo),
        cliente: cliente_de(&foto["cliente"]),
        lineas: foto["lineas"]
            .as_array()
            .into_iter()
            .flatten()
            .map(linea)
            .collect(),
        bases: bases_de(t),
        descuento: porcentaje_o_nada(entero(&t["descuento_bp"])),
        base: euros(entero(&t["base_cent"])),
        total: euros(entero(&t["total_cent"])),
        vencimiento: if vencimiento.is_empty() {
            vencimiento
        } else {
            fecha_corta(&vencimiento)
        },
        notas,
    }
}

#[derive(Template)]
#[template(path = "albaran_imprimir.html")]
pub struct PaginaAlbaranImpreso {
    pub trabajo_id: i64,
    pub codigo: String,
    pub fecha: String,
    pub trabajo: String,
    pub cliente: String,
    pub direccion: String,
    pub ventanas: Vec<String>,
    pub empresa: CabeceraDeEmpresa,
}

impl PaginaAlbaranImpreso {
    pub fn desde(
        trabajo_id: i64,
        foto: &Value,
        empresa: CabeceraDeEmpresa,
    ) -> PaginaAlbaranImpreso {
        PaginaAlbaranImpreso {
            trabajo_id,
            codigo: texto(&foto["codigo"]),
            fecha: fecha_corta(&texto(&foto["fecha"])),
            trabajo: texto(&foto["trabajo"]),
            cliente: texto(&foto["cliente"]),
            direccion: texto(&foto["direccion_montaje"]),
            ventanas: foto["ventanas"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|v| texto(&v["descripcion"]))
                .collect(),
            empresa,
        }
    }
}
