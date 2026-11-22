use askama::Template;

use super::{GrupoMenu, UsuarioEnPantalla};
use crate::aplicacion::ajustes_presupuesto::TextosDeAjustes;

#[derive(Template)]
#[template(path = "ajustes_presupuestos.html")]
pub struct PaginaAjustesPresupuestos {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub textos: TextosDeAjustes,
    pub logo: Option<String>,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

impl PaginaAjustesPresupuestos {
    pub fn v(&self, clave: &str) -> &str {
        self.textos.valor(clave)
    }
}

use dominio::unidades::{Dinero, Porcentaje};

use super::Opcion;
use crate::aplicacion::datos_de_empresa::CabeceraDeEmpresa;
use crate::aplicacion::presupuestos::FichaDePresupuesto;
use crate::aplicacion::resumen_de_linea::ResumenDeVentana;
use crate::bd::consultas::lineas_presupuesto::FilaLinea;
use crate::bd::consultas::presupuestos::FilaPresupuesto;
use crate::fecha::{formatear_para_pantalla, leer_utc};

pub fn euros(centimos: i64) -> String {
    Dinero::desde_centimos(centimos).formatear()
}

pub fn nombre_de_estado(estado: &str) -> &'static str {
    match estado {
        "borrador" => "Borrador",
        "enviado" => "Enviado",
        "aceptado" => "Aceptado",
        "rechazado" => "Rechazado",
        _ => "Caducado",
    }
}

pub struct FilaDePresupuesto {
    pub id: i64,
    pub codigo: String,
    pub cliente: String,
    pub estado: String,
    pub nombre_estado: &'static str,
    pub total: String,
    pub completo: bool,
    pub fecha: String,
}

impl From<&FilaPresupuesto> for FilaDePresupuesto {
    fn from(p: &FilaPresupuesto) -> Self {
        FilaDePresupuesto {
            id: p.id,
            codigo: p.codigo(),
            cliente: dominio::clientes::nombre_visible(p.cliente_id, p.cliente_nombre.as_deref()),
            estado: p.estado.clone(),
            nombre_estado: nombre_de_estado(&p.estado),
            total: euros(p.total_cent),
            completo: p.completo,
            fecha: leer_utc(&p.creado_en)
                .map(formatear_para_pantalla)
                .unwrap_or_default(),
        }
    }
}

#[derive(Template)]
#[template(path = "presupuestos.html")]
pub struct PaginaPresupuestos {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub filas: Vec<FilaDePresupuesto>,
    pub filtro: String,
    pub clientes: Vec<Opcion>,
}

pub struct TarjetaDeLinea {
    pub orden: i64,
    pub es_ventana: bool,
    pub descripcion: String,
    pub cantidad: String,
    pub precio: String,
    pub completa: bool,
    pub motivo: String,
    pub dibujo: String,
}

impl From<&FilaLinea> for TarjetaDeLinea {
    fn from(l: &FilaLinea) -> Self {
        let faltan: Vec<String> = serde_json::from_str(&l.faltan_json).unwrap_or_default();
        let mut motivo = faltan.first().cloned().unwrap_or_default();
        if l.revisar_limites {
            motivo = "medida fuera de los límites de la serie: revísala".into();
        }
        TarjetaDeLinea {
            orden: l.orden,
            es_ventana: l.tipo == "ventana",
            descripcion: l.descripcion.clone(),
            cantidad: dominio::unidades::formatear_es(i128::from(l.cantidad_mil), 3, 0),
            precio: euros(l.precio_cent),
            completa: !l.incompleta && !l.revisar_limites,
            motivo,
            dibujo: ResumenDeVentana::leer(l.resultado_json.as_deref()).dibujo,
        }
    }
}

/// Perfiles, cristal, accesorios y mano de obra del pie fijo (costes, solo para el propietario).
#[derive(Default)]
pub struct Desglose {
    pub perfiles: String,
    pub cristal: String,
    pub accesorios: String,
    pub mano_de_obra: String,
}

pub fn desglosar(lineas: &[FilaLinea]) -> Desglose {
    let mut suma = [0i64; 4];
    for l in lineas.iter().filter(|l| l.tipo == "ventana") {
        let r = ResumenDeVentana::leer(l.resultado_json.as_deref());
        let n = l.cantidad_mil / 1_000;
        for (i, valor) in [
            r.perfiles_cent,
            r.cristal_cent,
            r.accesorios_cent,
            l.mano_de_obra_cent,
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(total) = suma.get_mut(i) {
                *total += valor * n;
            }
        }
    }
    let [perfiles, cristal, accesorios, mano] = suma.map(euros);
    Desglose {
        perfiles,
        cristal,
        accesorios,
        mano_de_obra: mano,
    }
}

#[derive(Template)]
#[template(path = "presupuesto.html")]
pub struct PaginaPresupuesto {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub ficha: FichaDePresupuesto,
    pub codigo: String,
    pub cliente: String,
    pub nombre_estado: &'static str,
    pub lineas: Vec<TarjetaDeLinea>,
    pub desglose: Desglose,
    pub ivas: Vec<(String, String, String)>,
    pub motivos: Vec<String>,
    pub direcciones: Vec<Opcion>,
    pub opciones_iva: Vec<Opcion>,
    pub descuento: String,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

impl PaginaPresupuesto {
    pub fn total(&self) -> String {
        euros(self.ficha.presupuesto.total_cent)
    }

    pub fn base(&self) -> String {
        euros(self.ficha.presupuesto.base_cent)
    }

    pub fn editable(&self) -> bool {
        self.ficha.es_editable()
    }

    pub fn estado(&self) -> &str {
        &self.ficha.presupuesto.estado
    }

    pub fn direccion_elegida(&self) -> String {
        self.ficha
            .presupuesto
            .direccion_montaje_id
            .map(|d| d.to_string())
            .unwrap_or_default()
    }
}

pub fn porcentaje_texto(bp: i64) -> String {
    Porcentaje::desde_puntos_basicos(bp).formatear(false)
}

pub struct LineaImpresa {
    pub orden: i64,
    pub es_ventana: bool,
    /// El primer tramo de la descripción; el resto va en renglones debajo.
    pub titulo: String,
    pub detalles: Vec<String>,
    pub cantidad: String,
    pub precio: String,
    pub dibujo: String,
}

#[derive(Template)]
#[template(path = "presupuesto_imprimir.html")]
pub struct PaginaPresupuestoImpreso {
    pub id: i64,
    pub codigo: String,
    pub fecha: String,
    pub validez: String,
    pub empresa: CabeceraDeEmpresa,
    pub cliente: Vec<String>,
    pub direccion_montaje: String,
    pub lineas: Vec<LineaImpresa>,
    pub bases: Vec<(String, String, String)>,
    pub descuento: String,
    pub base: String,
    pub total: String,
    pub condiciones: String,
    pub provisional: bool,
    pub estado: &'static str,
}
