use askama::Template;

use super::{GrupoMenu, Opcion, UsuarioEnPantalla, nombre_de_unidad, porcentaje, precio};
use crate::aplicacion::importar_tarifa::PrevisualizacionDeTarifa;
use crate::aplicacion::tarifas::texto_de_origen;
use crate::bd::consultas::tarifas::{FilaTarifa, PrecioDeTarifa};
use crate::fecha::{formatear_para_pantalla, leer_utc};

pub struct FilaDeTarifa {
    pub id: i64,
    pub proveedor: String,
    pub nombre: String,
    pub fecha_tarifa: String,
    pub importada: String,
    pub origen: &'static str,
    pub descuento: String,
    pub precios: i64,
    pub vigente: bool,
}

impl From<&FilaTarifa> for FilaDeTarifa {
    fn from(t: &FilaTarifa) -> Self {
        FilaDeTarifa {
            id: t.id,
            proveedor: t.proveedor.clone(),
            nombre: t.nombre.clone(),
            fecha_tarifa: t
                .fecha_tarifa
                .as_deref()
                .map(fecha_corta)
                .unwrap_or_default(),
            importada: leer_utc(&t.importada_en)
                .map(formatear_para_pantalla)
                .unwrap_or_default(),
            origen: texto_de_origen(&t.formato_origen),
            descuento: porcentaje(t.descuento_compra_bp),
            precios: t.precios,
            vigente: t.estado == "vigente",
        }
    }
}

/// «2026-11-21» pasa a «21/11/2026».
pub fn fecha_corta(iso: &str) -> String {
    let partes: Vec<&str> = iso.split('-').collect();
    match partes.as_slice() {
        [anio, mes, dia] => format!("{dia}/{mes}/{anio}"),
        _ => iso.to_owned(),
    }
}

#[derive(Template)]
#[template(path = "catalogo_tarifas.html")]
pub struct PaginaTarifas {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub tarifas: Vec<FilaDeTarifa>,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

pub struct PrecioEnPantalla {
    pub referencia: String,
    pub descripcion: String,
    pub unidad: &'static str,
    pub precio: String,
}

impl From<&PrecioDeTarifa> for PrecioEnPantalla {
    fn from(p: &PrecioDeTarifa) -> Self {
        PrecioEnPantalla {
            referencia: p.referencia.clone(),
            descripcion: p.descripcion.clone(),
            unidad: nombre_de_unidad(&p.unidad),
            precio: precio(Some(p.precio_pu)),
        }
    }
}

#[derive(Template)]
#[template(path = "catalogo_tarifa.html")]
pub struct PaginaTarifa {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub tarifa: FilaDeTarifa,
    pub precios: Vec<PrecioEnPantalla>,
}

/// Lo elegido en el paso 1, que viaja oculto hasta confirmar.
#[derive(Debug, Clone, Default)]
pub struct ValoresDeImportacion {
    pub proveedor: String,
    pub nombre: String,
    pub fecha: String,
    pub descuento: String,
    pub texto: String,
}

pub struct CambioEnPantalla {
    pub referencia: String,
    pub descripcion: String,
    pub anterior: String,
    pub nuevo: String,
    pub variacion: String,
    pub sube: bool,
    pub salto: bool,
}

pub struct NuevoEnPantalla {
    pub referencia: String,
    pub descripcion: String,
    pub unidad: &'static str,
    pub precio: String,
}

pub struct VistaPreviaDeTarifa {
    pub proveedor: String,
    pub nuevos: Vec<NuevoEnPantalla>,
    pub cambios: Vec<CambioEnPantalla>,
    pub iguales: usize,
    pub ausentes: Vec<String>,
    pub errores: Vec<String>,
    pub avisos: Vec<String>,
    pub sha256: String,
    pub se_puede_importar: bool,
}

impl From<PrevisualizacionDeTarifa> for VistaPreviaDeTarifa {
    fn from(p: PrevisualizacionDeTarifa) -> Self {
        let salto = dominio::tarifa::SALTO_DE_PRECIO_AVISADO;
        let c = &p.comparacion;
        VistaPreviaDeTarifa {
            se_puede_importar: p.se_puede_importar(),
            nuevos: c
                .nuevos
                .iter()
                .map(|f| NuevoEnPantalla {
                    referencia: f.referencia.clone(),
                    descripcion: f.descripcion.clone(),
                    unidad: f.unidad.nombre(),
                    precio: f.precio.formatear(),
                })
                .collect(),
            cambios: c
                .cambios
                .iter()
                .map(|k| CambioEnPantalla {
                    referencia: k.referencia.clone(),
                    descripcion: k.descripcion.clone(),
                    anterior: k
                        .anterior
                        .map_or_else(|| "sin precio".to_owned(), |a| a.formatear()),
                    nuevo: k.nuevo.formatear(),
                    variacion: k.variacion.map(|v| v.formatear(true)).unwrap_or_default(),
                    sube: k.variacion.is_some_and(|v| v.puntos_basicos() > 0),
                    salto: k.variacion.is_some_and(|v| v.absoluto() > salto),
                })
                .collect(),
            iguales: c.iguales,
            ausentes: c.ausentes.clone(),
            proveedor: p.proveedor,
            errores: p.errores,
            avisos: p.avisos,
            sha256: p.sha256,
        }
    }
}

#[derive(Template)]
#[template(path = "importar_tarifa.html")]
pub struct PaginaImportarTarifa {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub proveedores: Vec<Opcion>,
    pub valores: ValoresDeImportacion,
    pub previa: Option<VistaPreviaDeTarifa>,
    pub aviso: Option<String>,
    pub plantilla: String,
}
