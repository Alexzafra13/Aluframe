//! Pantallas de facturas y cobros (PANTALLAS 11).

use askama::Template;
use dominio::factura::estado_de_cobro;
use dominio::stock::formatear_cantidad;
use dominio::unidades::{Dinero, Porcentaje, PrecioUnitario};

use super::{GrupoMenu, Opcion, UsuarioEnPantalla, euros, fecha_corta};
use crate::aplicacion::facturas::FichaDeFactura;
use crate::bd::consultas::cobros::FilaCobro;
use crate::bd::consultas::facturas::FilaFactura;

/// «Borrador», «Rectificativa» o el estado de cobro calculado con los cobros.
pub fn estado_de_factura(f: &FilaFactura, hoy: &str) -> &'static str {
    match (f.estado.as_str(), f.tipo.as_str()) {
        ("borrador", _) => "Borrador",
        (_, "rectificativa") => "Rectificativa",
        _ => estado_de_cobro(
            Dinero::desde_centimos(f.total_cent),
            Dinero::desde_centimos(f.cobrado_cent),
            (f.vencimiento.as_deref(), hoy),
        )
        .nombre(),
    }
}

pub struct FilaFacturaEnLista {
    pub id: i64,
    pub codigo: String,
    pub cliente: String,
    pub fecha: String,
    pub total: String,
    pub pendiente: String,
    pub estado: &'static str,
}

impl FilaFacturaEnLista {
    pub fn de(f: &FilaFactura, hoy: &str) -> FilaFacturaEnLista {
        let fecha = f
            .fecha_emision
            .as_deref()
            .unwrap_or(&f.creado_en[..10.min(f.creado_en.len())]);
        FilaFacturaEnLista {
            id: f.id,
            codigo: f.codigo(),
            cliente: dominio::clientes::nombre_visible(f.cliente_id, f.cliente_nombre.as_deref()),
            fecha: fecha_corta(fecha),
            total: euros(f.total_cent),
            pendiente: euros((f.total_cent - f.cobrado_cent).max(0)),
            estado: estado_de_factura(f, hoy),
        }
    }
}

#[derive(Template)]
#[template(path = "facturas.html")]
pub struct PaginaFacturas {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub filas: Vec<FilaFacturaEnLista>,
    pub pendientes: bool,
    pub clientes: Vec<Opcion>,
}

pub struct LineaEnPantalla {
    pub id: i64,
    pub descripcion: String,
    pub cantidad: String,
    pub precio: String,
    pub descuento: String,
    pub iva: String,
    pub importe: String,
}

pub struct CobroEnPantalla {
    pub id: i64,
    pub fecha: String,
    pub importe: String,
    pub medio: String,
    pub nota: String,
    pub anulado: bool,
    pub motivo: String,
    pub usuario: String,
}

pub fn medio_legible(medio: &str) -> String {
    crate::aplicacion::facturas::cobros::MEDIOS
        .iter()
        .find(|(m, _)| *m == medio)
        .map_or_else(|| medio.to_owned(), |(_, t)| (*t).to_owned())
}

impl From<&FilaCobro> for CobroEnPantalla {
    fn from(c: &FilaCobro) -> Self {
        CobroEnPantalla {
            id: c.id,
            fecha: fecha_corta(&c.fecha),
            importe: euros(c.importe_cent),
            medio: medio_legible(&c.medio),
            nota: c.nota.clone().unwrap_or_default(),
            anulado: c.anulado,
            motivo: c.motivo_anulacion.clone().unwrap_or_default(),
            usuario: c.usuario.clone(),
        }
    }
}

#[derive(Template)]
#[template(path = "factura.html")]
pub struct PaginaFactura {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: i64,
    pub version: i64,
    pub codigo: String,
    pub cliente: String,
    pub cliente_id: i64,
    pub trabajo_id: Option<i64>,
    pub borrador: bool,
    pub rectificativa: bool,
    pub rectifica: Option<(i64, String)>,
    pub motivo: String,
    pub estado: &'static str,
    pub fecha_operacion: String,
    pub vencimiento: String,
    pub notas: String,
    pub descuento: String,
    pub lineas: Vec<LineaEnPantalla>,
    pub base: String,
    pub iva: String,
    pub total: String,
    pub cobrado: String,
    pub pendiente: String,
    pub pendiente_en_campo: String,
    pub cobros: Vec<CobroEnPantalla>,
    pub medios: Vec<Opcion>,
    pub ivas: Vec<Opcion>,
    pub hoy: String,
    pub falta_nif: bool,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

fn linea_en_pantalla(l: &crate::bd::consultas::facturas::FilaLineaFactura) -> LineaEnPantalla {
    LineaEnPantalla {
        id: l.id,
        descripcion: l.descripcion.clone(),
        cantidad: formatear_cantidad(l.cantidad_mil),
        precio: PrecioUnitario::desde_diezmilesimas(l.precio_unitario_pu).formatear(),
        descuento: if l.descuento_bp == 0 {
            String::new()
        } else {
            Porcentaje::desde_puntos_basicos(l.descuento_bp).formatear(false)
        },
        iva: Porcentaje::desde_puntos_basicos(l.tipo_iva_bp).formatear(false),
        importe: euros(l.base_cent),
    }
}

fn descuento_en_campo(bp: i64) -> String {
    if bp == 0 {
        String::new()
    } else {
        Porcentaje::desde_puntos_basicos(bp).formatear(false)
    }
}

pub fn opciones_de_medios() -> Vec<Opcion> {
    crate::aplicacion::facturas::cobros::MEDIOS
        .iter()
        .map(|(v, t)| Opcion {
            valor: (*v).to_owned(),
            texto: (*t).to_owned(),
        })
        .collect()
}

/// Lo pendiente como se escribe en el campo del importe («125,50»).
fn en_campo(centimos: i64) -> String {
    format!("{},{:02}", centimos / 100, centimos % 100)
}

impl PaginaFactura {
    pub fn nueva(
        usuario: UsuarioEnPantalla,
        (ficha, hoy, ivas): (&FichaDeFactura, &str, Vec<Opcion>),
        (aviso, confirmacion): (Option<String>, Option<String>),
    ) -> PaginaFactura {
        let f = &ficha.factura;
        let pendiente = (f.total_cent - f.cobrado_cent).max(0);
        PaginaFactura {
            usuario,
            menu: super::MENU,
            seccion: "facturas",
            id: f.id,
            version: f.version,
            codigo: f.codigo(),
            cliente: dominio::clientes::nombre_visible(f.cliente_id, f.cliente_nombre.as_deref()),
            cliente_id: f.cliente_id,
            trabajo_id: f.trabajo_id,
            borrador: f.estado == "borrador",
            rectificativa: f.tipo == "rectificativa",
            rectifica: ficha.rectifica.as_ref().map(|r| (r.id, r.codigo())),
            motivo: f.motivo_rectificacion.clone().unwrap_or_default(),
            estado: estado_de_factura(f, hoy),
            fecha_operacion: f.fecha_operacion.clone().unwrap_or_default(),
            vencimiento: f.vencimiento.clone().unwrap_or_default(),
            notas: f.notas.clone().unwrap_or_default(),
            descuento: descuento_en_campo(f.descuento_bp),
            lineas: ficha.lineas.iter().map(linea_en_pantalla).collect(),
            base: euros(f.base_cent),
            iva: euros(f.iva_cent),
            total: euros(f.total_cent),
            cobrado: euros(f.cobrado_cent),
            pendiente: euros(pendiente),
            pendiente_en_campo: en_campo(pendiente),
            cobros: ficha.cobros.iter().map(CobroEnPantalla::from).collect(),
            medios: opciones_de_medios(),
            ivas,
            hoy: hoy.to_owned(),
            falta_nif: ficha.cliente.nif.is_none(),
            aviso,
            confirmacion,
        }
    }
}
