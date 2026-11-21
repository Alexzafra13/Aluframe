use askama::Template;
use dominio::catalogo::{EntradaDeAcabado, EntradaDeArticulo, EntradaDeProveedor};
use dominio::tarifa::{TipoArticulo, UnidadDeVenta};
use dominio::unidades::{Longitud, Porcentaje, PrecioUnitario};

use super::{ErroresDeCampo, GrupoMenu, UsuarioEnPantalla};
use crate::bd::consultas::acabados::FilaAcabado;
use crate::bd::consultas::articulos::FilaArticulo;
use crate::bd::consultas::proveedores::FilaProveedor;

pub fn porcentaje(bp: i64) -> String {
    Porcentaje::desde_puntos_basicos(bp).formatear(false)
}

/// Un porcentaje como valor de formulario: «12,5» y vacío si es 0.
pub fn porcentaje_en_campo(bp: i64) -> String {
    if bp == 0 {
        return String::new();
    }
    porcentaje(bp).trim_end_matches(" %").to_owned()
}

pub fn precio(pu: Option<i64>) -> String {
    pu.map(|p| PrecioUnitario::desde_diezmilesimas(p).formatear())
        .unwrap_or_default()
}

pub fn nombre_de_unidad(unidad: &str) -> &'static str {
    UnidadDeVenta::desde_texto(unidad).map_or("", UnidadDeVenta::nombre)
}

pub struct Opcion {
    pub valor: String,
    pub texto: String,
}

pub fn opciones_de_tipo() -> Vec<Opcion> {
    TipoArticulo::TODOS
        .iter()
        .map(|t| Opcion {
            valor: t.como_texto().into(),
            texto: t.como_texto().into(),
        })
        .collect()
}

pub fn opciones_de_unidad() -> Vec<Opcion> {
    UnidadDeVenta::TODAS
        .iter()
        .map(|u| Opcion {
            valor: u.como_texto().into(),
            texto: u.nombre().into(),
        })
        .collect()
}

pub fn opciones_de_proveedor(proveedores: &[FilaProveedor]) -> Vec<Opcion> {
    proveedores
        .iter()
        .filter(|p| !p.archivado)
        .map(|p| Opcion {
            valor: p.id.to_string(),
            texto: p.nombre.clone(),
        })
        .collect()
}

#[derive(Template)]
#[template(path = "catalogo_proveedores.html")]
pub struct PaginaProveedores {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub proveedores: Vec<FilaProveedor>,
    pub valores: EntradaDeProveedor,
    pub errores: ErroresDeCampo,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

#[derive(Template)]
#[template(path = "catalogo_proveedor.html")]
pub struct PaginaProveedor {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: i64,
    pub version: i64,
    pub archivado: bool,
    pub valores: EntradaDeProveedor,
    pub errores: ErroresDeCampo,
    pub aviso: Option<String>,
}

pub fn entrada_de_proveedor(p: &FilaProveedor) -> EntradaDeProveedor {
    let texto = |v: &Option<String>| v.clone().unwrap_or_default();
    EntradaDeProveedor {
        nombre: p.nombre.clone(),
        nif: texto(&p.nif),
        contacto: texto(&p.contacto),
        condiciones_pago: texto(&p.condiciones_pago),
        descuento_habitual: porcentaje_en_campo(p.descuento_habitual_bp),
        notas: texto(&p.notas),
    }
}

#[derive(Template)]
#[template(path = "catalogo_acabados.html")]
pub struct PaginaAcabados {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub acabados: Vec<FilaAcabado>,
    pub valores: EntradaDeAcabado,
    pub errores: ErroresDeCampo,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

#[derive(Template)]
#[template(path = "catalogo_acabado.html")]
pub struct PaginaAcabado {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: i64,
    pub version: i64,
    pub archivado: bool,
    pub valores: EntradaDeAcabado,
    pub errores: ErroresDeCampo,
    pub aviso: Option<String>,
}

pub struct FilaDeArticulo {
    pub id: i64,
    pub proveedor: String,
    pub referencia: String,
    pub descripcion: String,
    pub tipo: String,
    pub unidad: &'static str,
    pub barra: String,
    pub precio: String,
    pub precio_en_campo: String,
}

impl From<&FilaArticulo> for FilaDeArticulo {
    fn from(a: &FilaArticulo) -> Self {
        FilaDeArticulo {
            id: a.id,
            proveedor: a.proveedor.clone(),
            referencia: a.referencia.clone(),
            descripcion: a.descripcion.clone(),
            tipo: a.tipo.clone(),
            unidad: nombre_de_unidad(&a.unidad),
            barra: a
                .longitud_barra_dmm
                .map(|l| format!("{} cm", Longitud::desde_decimas_de_mm(l).formatear_cm()))
                .unwrap_or_default(),
            precio: precio(a.precio_pu),
            precio_en_campo: a
                .precio_pu
                .map(|p| PrecioUnitario::desde_diezmilesimas(p).formatear_numero())
                .unwrap_or_default(),
        }
    }
}

#[derive(Template)]
#[template(path = "catalogo_articulos.html")]
pub struct PaginaArticulos {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub articulos: Vec<FilaDeArticulo>,
    pub proveedores: Vec<Opcion>,
    pub proveedor_elegido: String,
    pub texto: String,
    pub archivados: bool,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

#[derive(Template)]
#[template(path = "catalogo_articulo_formulario.html")]
pub struct PaginaFormularioArticulo {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: Option<i64>,
    pub version: i64,
    pub proveedores: Vec<Opcion>,
    pub proveedor_elegido: String,
    pub tipos: Vec<Opcion>,
    pub unidades: Vec<Opcion>,
    pub valores: EntradaDeArticulo,
    pub errores: ErroresDeCampo,
    pub aviso: Option<String>,
    pub archivado: bool,
    pub historial: Vec<PrecioHistorico>,
}

/// Una línea del histórico de precios de un artículo (una por versión de tarifa).
pub struct PrecioHistorico {
    pub tarifa: String,
    pub fecha: String,
    pub origen: &'static str,
    pub precio: String,
    pub vigente: bool,
}

pub fn entrada_de_articulo(a: &FilaArticulo) -> EntradaDeArticulo {
    EntradaDeArticulo {
        referencia: a.referencia.clone(),
        descripcion: a.descripcion.clone(),
        tipo: a.tipo.clone(),
        unidad: a.unidad.clone(),
        longitud_barra_cm: a
            .longitud_barra_dmm
            .map(|l| Longitud::desde_decimas_de_mm(l).formatear_cm())
            .unwrap_or_default(),
        unidades_por_caja: a
            .unidades_por_caja
            .map(|n| n.to_string())
            .unwrap_or_default(),
        con_acabado: a.con_acabado,
        stock_minimo: dominio::unidades::formatear_es(i128::from(a.stock_minimo_mil), 3, 0),
    }
}
