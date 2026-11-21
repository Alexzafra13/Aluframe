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
