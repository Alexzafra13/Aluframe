use std::collections::BTreeMap;

use super::Dato;
use crate::formula::Formula;
use crate::unidades::{Decimal, Longitud, SentidoRedondeo, Unidad};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoSerie {
    Borrador,
    Validada,
    Demo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redondeo {
    pub precision: Decimal,
    pub sentido: SentidoRedondeo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Referencia {
    pub referencia: Dato<String>,
    pub proveedor: Option<String>,
    /// Dibujo de la sección del perfil (`fabricante/archivo`, de `data/perfiles/`): solo para
    /// reconocer la pieza; no interviene en el precio (ADR-0017).
    pub silueta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Serie {
    pub codigo: String,
    pub fabricante: String,
    pub proveedor: Dato<String>,
    pub estado: EstadoSerie,
    pub unidad: Unidad,
    pub redondeo: Redondeo,
    pub longitud_barra: Dato<Longitud>,
    pub parametros: BTreeMap<String, Decimal>,
    pub referencias: BTreeMap<String, Referencia>,
    pub tipologias: Vec<Tipologia>,
}

impl Serie {
    pub fn tipologia(&self, codigo: &str) -> Option<&Tipologia> {
        self.tipologias.iter().find(|t| t.codigo == codigo)
    }
}

impl Tipologia {
    /// La tipología con cada pieza ya en el perfil que corresponde a las opciones elegidas; lo
    /// que no se elige toma el valor por defecto de la opción.
    pub fn con_opciones(&self, elegidas: &BTreeMap<String, String>) -> Tipologia {
        let valor_de = |codigo: &str| {
            elegidas.get(codigo).cloned().or_else(|| {
                let opcion = self.opciones.iter().find(|o| o.codigo == codigo)?;
                Some(opcion.por_defecto.clone())
            })
        };
        let mut resuelta = self.clone();
        for pieza in &mut resuelta.piezas {
            let elegida = pieza
                .perfil_segun
                .iter()
                .find(|v| valor_de(&v.opcion).as_deref() == Some(v.valor.as_str()));
            if let Some(variante) = elegida {
                pieza.perfil = variante.perfil.clone();
            }
            pieza.perfil_segun.clear();
        }
        resuelta
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tipologia {
    pub codigo: String,
    pub nombre: String,
    pub estado: Option<EstadoSerie>,
    pub limites: Option<Limites>,
    pub opciones: Vec<OpcionDeLinea>,
    pub dibujo: Dibujo,
    pub piezas: Vec<PiezaDeSerie>,
    pub vidrio: Dato<Vidrio>,
    pub accesorios: Vec<AccesorioDeSerie>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiezaDeSerie {
    pub perfil: String,
    pub cantidad: Dato<Formula>,
    pub longitud: Dato<Formula>,
    pub corte: Dato<u16>,
    /// Dónde va su número en el dibujo cuando el nombre no lo deja claro (`HOJA_CENTRAL`…).
    pub posicion: Option<String>,
    /// Otro perfil cuando una opción de la línea toma un valor (hojas Climalit, ADR-0018).
    pub perfil_segun: Vec<VarianteDePerfil>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarianteDePerfil {
    pub opcion: String,
    pub valor: String,
    pub perfil: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vidrio {
    pub tipo: String,
    pub piezas: Vec<PiezaDeVidrio>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiezaDeVidrio {
    pub cantidad: Formula,
    pub ancho: Formula,
    pub alto: Formula,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AplicaA {
    Marco,
    Hojas,
    Ventana,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnidadCantidad {
    Unidad,
    Metro,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccesorioDeSerie {
    pub articulo: String,
    pub cantidad: Dato<Formula>,
    pub aplica_a: AplicaA,
    pub unidad_cantidad: UnidadCantidad,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpcionDeLinea {
    pub codigo: String,
    pub nombre: String,
    pub valores: Vec<String>,
    pub por_defecto: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDibujo {
    Corredera,
    Abatible,
    Fija,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dibujo {
    pub tipo: TipoDibujo,
    pub hojas: u8,
    pub hoja_movil: Option<String>,
    pub bisagras: Option<String>,
    pub cremona: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rango {
    pub minimo: Dato<Decimal>,
    pub maximo: Dato<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limites {
    pub ancho: Rango,
    pub alto: Rango,
}
