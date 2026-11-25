use std::collections::BTreeMap;

use askama::Template;
use dominio::series::{Dato, OpcionDeLinea, Serie};

use super::{
    APedirEnPantalla, ChapaEnPantalla, FichaDeSerie, GrupoMenu, HerrajeEnPantalla, Opcion,
    UsuarioEnPantalla, a_pedir_en_pantalla, chapas_en_pantalla, herrajes_en_pantalla,
};
use crate::aplicacion::calculo_de_ventana::CalculoDeVentana;
use crate::series::siluetas::url_de_silueta;

pub use dominio::texto::nombre_legible;

pub struct PestanaDeLinea {
    pub orden: i64,
    pub titulo: String,
    pub completa: bool,
}

pub struct OpcionEnPantalla {
    pub codigo: String,
    pub nombre: String,
    /// (valor, texto en pantalla): «climalit» se lee «Climalit». Vacío si es una medida.
    pub valores: Vec<(String, String)>,
    pub elegido: String,
    pub es_medida: bool,
    /// `parteluces=si;relleno_abajo=chapa`: el navegador la muestra u oculta al cambiar las
    /// demás sin pedir nada al servidor (ADR-0030).
    pub solo_si: String,
    pub visible: bool,
}

/// Las opciones de la tipología tal como se ven, con lo elegido o el valor de la serie.
pub fn opciones_en_pantalla(
    opciones: &[OpcionDeLinea],
    elegidas: &BTreeMap<String, String>,
) -> Vec<OpcionEnPantalla> {
    let valor_de = |codigo: &str| {
        elegidas.get(codigo).cloned().or_else(|| {
            let opcion = opciones.iter().find(|o| o.codigo == codigo)?;
            Some(opcion.por_defecto.clone())
        })
    };
    opciones
        .iter()
        .map(|o| OpcionEnPantalla {
            codigo: o.codigo.clone(),
            nombre: o.nombre.clone(),
            valores: o
                .valores
                .iter()
                .map(|v| (v.clone(), nombre_legible(v)))
                .collect(),
            elegido: valor_de(&o.codigo).unwrap_or_default().replace('.', ","),
            es_medida: o.medida.is_some(),
            solo_si: o
                .solo_si
                .iter()
                .map(|(opcion, valor)| format!("{opcion}={valor}"))
                .collect::<Vec<_>>()
                .join(";"),
            visible: o
                .solo_si
                .iter()
                .all(|(opcion, valor)| valor_de(opcion).as_deref() == Some(valor.as_str())),
        })
        .collect()
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
    pub silueta: Option<String>,
}

#[derive(Default)]
pub struct VistaCalculo {
    pub dibujo: String,
    pub piezas: Vec<PiezaEnPantalla>,
    pub ficha: Option<FichaDeSerie>,
    /// La tipología no lleva cristal (una mosquitera): se ocultan el vidrio y sus capas.
    pub sin_vidrio: bool,
    /// Lo que no se corta: cierres, kits, ruedas, tornillos…
    pub herrajes: Vec<HerrajeEnPantalla>,
    /// Cristal y lo que se encarga a medida (la persiana).
    pub para_pedir: Vec<APedirEnPantalla>,
    pub chapas: Vec<ChapaEnPantalla>,
    pub faltan: Vec<String>,
    pub fuera_de_limites: Vec<String>,
    pub perfiles: String,
    pub alternativa: String,
    pub cristal: String,
    pub accesorios: String,
    pub mano_de_obra: String,
    pub precio: String,
    pub completa: bool,
    /// Lo marcó quien presupuesta: la medida se sale de la serie pero es la que es.
    pub limites_revisados: bool,
}

fn pieza_en_pantalla(serie: &Serie, p: &dominio::series::PiezaCalculada) -> PiezaEnPantalla {
    PiezaEnPantalla {
        silueta: url_de_silueta(serie, &p.perfil),
        numero: p.numero,
        nombre: nombre_legible(&p.perfil),
        cantidad: p.cantidad,
        medida: p.longitud.formatear_cm(),
        corte: p.corte.map_or_else(|| "—".into(), |g| format!("{g}°")),
    }
}

/// Las que se cortan y las que aún no tienen medida («pendiente»), por su número.
fn piezas_en_pantalla(c: &CalculoDeVentana) -> Vec<PiezaEnPantalla> {
    let r = &c.resultado;
    let mut piezas: Vec<PiezaEnPantalla> = r
        .piezas
        .iter()
        .map(|p| pieza_en_pantalla(&c.serie, p))
        .collect();
    piezas.extend(r.piezas_pendientes.iter().map(|p| PiezaEnPantalla {
        silueta: url_de_silueta(&c.serie, &p.perfil),
        numero: p.numero,
        nombre: nombre_legible(&p.perfil),
        cantidad: p.cantidad.unwrap_or_default(),
        medida: "pendiente".into(),
        corte: String::new(),
    }));
    piezas.sort_by_key(|p| p.numero);
    piezas
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
            piezas: piezas_en_pantalla(c),
            ficha: Some(FichaDeSerie::desde(c)),
            sin_vidrio: c
                .serie
                .tipologia(&c.resultado.tipologia)
                .is_some_and(|t| matches!(&t.vidrio, Dato::Valor(v) if v.tipo.is_empty())),
            herrajes: herrajes_en_pantalla(c),
            para_pedir: a_pedir_en_pantalla(c),
            chapas: chapas_en_pantalla(c),
            faltan: c.precio.faltan.clone(),
            fuera_de_limites: c.resultado.fuera_de_limites.clone(),
            perfiles: m.perfiles.formatear(),
            alternativa,
            cristal: m.cristal.formatear(),
            accesorios: m.accesorios.formatear(),
            mano_de_obra: String::new(),
            precio: c.precio.precio.formatear(),
            completa: c.precio.es_final(),
            limites_revisados: false,
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
