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
    /// El valor elegido de una opción o, si no se eligió, el que trae la serie.
    pub fn valor_de_opcion(
        &self,
        codigo: &str,
        elegidas: &BTreeMap<String, String>,
    ) -> Option<String> {
        elegidas.get(codigo).cloned().or_else(|| {
            let opcion = self.opciones.iter().find(|o| o.codigo == codigo)?;
            Some(opcion.por_defecto.clone())
        })
    }

    pub fn cumple(&self, condicion: &SoloSi, elegidas: &BTreeMap<String, String>) -> bool {
        condicion.iter().all(|(opcion, valor)| {
            self.valor_de_opcion(opcion, elegidas).as_deref() == Some(valor.as_str())
        })
    }

    /// Una opción con `solo_si` que no se cumple ni se pregunta ni cuenta (ADR-0030).
    pub fn se_pregunta(&self, opcion: &OpcionDeLinea, elegidas: &BTreeMap<String, String>) -> bool {
        self.cumple(&opcion.solo_si, elegidas)
    }

    /// La tipología con cada pieza ya en el perfil que corresponde a las opciones elegidas; lo
    /// que no se elige toma el valor por defecto de la opción.
    pub fn con_opciones(&self, elegidas: &BTreeMap<String, String>) -> Tipologia {
        let valor_de = |codigo: &str| self.valor_de_opcion(codigo, elegidas);
        let cumple = |condicion: &SoloSi| self.cumple(condicion, elegidas);
        let mut resuelta = self.clone();
        resuelta.piezas.retain(|p| cumple(&p.solo_si));
        resuelta.accesorios.retain(|a| cumple(&a.solo_si));
        resuelta.chapas.retain(|c| cumple(&c.solo_si));
        if let Dato::Valor(vidrio) = &mut resuelta.vidrio {
            vidrio.piezas.retain(|p| cumple(&p.solo_si));
        }
        resuelta.parteluces = resuelta.parteluces.filter(|p| cumple(&p.solo_si));
        resuelta.persiana = resuelta.persiana.filter(|p| cumple(&p.solo_si));
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
    /// Paneles opacos (la chapa de abajo de un parteluces), ADR-0030.
    pub chapas: Vec<PiezaDeChapa>,
    pub parteluces: Option<ParteLucesDeSerie>,
    /// Persiana en la ventana (ADR-0032): la medida escrita es la del conjunto.
    pub persiana: Option<PersianaDeSerie>,
}

/// Persiana montada en la ventana (ADR-0032). La medida que se escribe es la de todo el conjunto;
/// las fórmulas de la tipología ven en `ancho` y `alto` la del marco, que sale de quitar una guía
/// a cada lado y el cajón arriba. La persiana se pide: va en `accesorios`, no se corta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersianaDeSerie {
    pub solo_si: SoloSi,
    pub montaje: MontajeDePersiana,
    /// Ancho de cada guía lateral; se evalúa con la medida escrita. En la ventana es obligatorio;
    /// en pared (el ala de la guía de L) puede quedar en `CONFIRMAR`.
    pub guia: Dato<Formula>,
    /// Alto del cajón según la medida del conjunto; `CONFIRMAR` hasta tener el dato.
    pub cajon: Dato<Formula>,
    /// Opción de la línea que dice en qué lado va la cinta (`derecha` o `izquierda`).
    pub cinta: String,
}

/// Dónde se monta la persiana (ADR-0032 y ADR-0033).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MontajeDePersiana {
    /// En la ventana: la medida escrita es la del conjunto y el marco sale de quitarle guías y
    /// cajón.
    Ventana,
    /// Suelta, en la pared con guías de L: la medida escrita es la del hueco y no se le quita
    /// nada; guías y cajón quedan por fuera. Se pide hecha.
    Pared,
}

/// Condición `solo_si: { opcion: valor }`: la parte solo existe si se cumplen todas.
pub type SoloSi = Vec<(String, String)>;

/// Un travesaño horizontal en cada hoja (ADR-0030). Solo da el dibujo: sus piezas, cristales y
/// chapas se declaran en `piezas`, `vidrio` y `chapas` con el mismo `solo_si`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParteLucesDeSerie {
    pub solo_si: SoloSi,
    /// Medida de la chapa o del cristal de abajo: lo que se escribe al presupuestar.
    pub abajo: Formula,
    /// Altura del eje del parteluces desde abajo de la ventana; `CONFIRMAR` hasta tener el dato.
    pub altura: Dato<Formula>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiezaDeChapa {
    /// Código de `referencias` (CHAPA_LACADA_2_CARAS…).
    pub articulo: String,
    pub nombre: Option<String>,
    pub cantidad: Dato<Formula>,
    pub ancho: Dato<Formula>,
    pub alto: Dato<Formula>,
    pub solo_si: SoloSi,
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
    pub solo_si: SoloSi,
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
    /// «Arriba», «Abajo»: para distinguirlos cuando hay parteluces.
    pub nombre: Option<String>,
    pub cantidad: Formula,
    pub ancho: Dato<Formula>,
    pub alto: Dato<Formula>,
    pub solo_si: SoloSi,
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
    pub solo_si: SoloSi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpcionDeLinea {
    pub codigo: String,
    pub nombre: String,
    /// Vacío si la opción es una medida.
    pub valores: Vec<String>,
    pub por_defecto: String,
    /// Con rango, la opción es una medida en la unidad de la serie que se escribe al
    /// presupuestar y se usa en las fórmulas con su código (ADR-0030).
    pub medida: Option<RangoDeMedida>,
    /// Solo se pregunta (y solo cuenta) si se cumple.
    pub solo_si: SoloSi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangoDeMedida {
    pub minimo: Decimal,
    pub maximo: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDibujo {
    Corredera,
    Abatible,
    Fija,
    /// Mosquitera enrollable: cajón, guías, malla y terminal (ADR-0031).
    Mosquitera,
    /// Persiana suelta en pared: cajón, guías de L y lamas (ADR-0033).
    Persiana,
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
