use std::collections::BTreeMap;

use thiserror::Error;

use super::resultado::MAXIMO_POR_PIEZA;
use super::{
    AccesorioCalculado, Dato, EstadoSerie, PiezaCalculada, PiezaPendiente, ResultadoVentana, Serie,
    Tipologia, UnidadCantidad,
};
use crate::formula::{ErrorFormula, Formula, Valores};
use crate::texto::nombre_legible;
use crate::unidades::{Decimal, Longitud};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErrorCalculo {
    #[error("La tipología «{0}» no existe en esta serie.")]
    TipologiaDesconocida(String),
    #[error("El {0} debe ser mayor que cero.")]
    MedidaNoPositiva(&'static str),
    #[error("{donde}: {error}")]
    Formula { donde: String, error: ErrorFormula },
    #[error("{donde}: la cantidad sale {valor} y debe ser un número entero entre 1 y 999.")]
    CantidadNoValida { donde: String, valor: String },
    #[error("{donde}: la medida sale {valor} y debe ser mayor que cero.")]
    MedidaNoValida { donde: String, valor: String },
    #[error("{nombre}: escribe una medida entre {minimo} y {maximo}.")]
    MedidaDeOpcion {
        nombre: String,
        minimo: String,
        maximo: String,
    },
}

/// Con los valores por defecto de las opciones de la línea.
pub fn calcular_ventana(
    serie: &Serie,
    codigo_tipologia: &str,
    ancho: Decimal,
    alto: Decimal,
) -> Result<ResultadoVentana, ErrorCalculo> {
    calcular_ventana_con_opciones(serie, codigo_tipologia, (ancho, alto), &BTreeMap::new())
}

/// Las opciones elegidas pueden cambiar el perfil de alguna pieza (`perfil_segun`).
pub fn calcular_ventana_con_opciones(
    serie: &Serie,
    codigo_tipologia: &str,
    (ancho, alto): (Decimal, Decimal),
    opciones: &BTreeMap<String, String>,
) -> Result<ResultadoVentana, ErrorCalculo> {
    let tipologia = serie
        .tipologia(codigo_tipologia)
        .ok_or_else(|| ErrorCalculo::TipologiaDesconocida(codigo_tipologia.to_owned()))?
        .con_opciones(opciones);
    let tipologia = &tipologia;
    comprobar_medidas(ancho, alto)?;
    let mut calculo = Calculo::new(serie, ancho, alto);
    let medidas = super::calculo_rellenos::medidas_de_linea(tipologia, opciones)?;
    calculo.valores.extend(medidas);
    let mut pendientes = pendientes_de_estado(serie, tipologia);
    let persiana = calculo.aplicar_persiana(tipologia, opciones, &mut pendientes)?;
    let (piezas, piezas_pendientes) = calculo.piezas(tipologia, &mut pendientes)?;
    let vidrio = calculo.vidrio(tipologia, &mut pendientes)?;
    let chapas = calculo.chapas(tipologia, &mut pendientes)?;
    let parteluces = calculo.parteluces(tipologia, &mut pendientes)?;
    let accesorios = calculo.accesorios(tipologia, &mut pendientes)?;
    pendientes.extend(super::validacion::referencias_sin_confirmar(
        serie, tipologia,
    ));
    if serie.longitud_barra.es_confirmar() {
        pendientes.push("Falta la longitud de barra de la serie.".into());
    }
    Ok(ResultadoVentana {
        serie: serie.codigo.clone(),
        tipologia: tipologia.codigo.clone(),
        ancho,
        alto,
        piezas,
        piezas_pendientes,
        accesorios,
        vidrio,
        chapas,
        parteluces,
        persiana,
        pendientes,
        fuera_de_limites: super::validacion::comprobar_limites(tipologia, ancho, alto),
    })
}

/// Lo primero que se lee en la tarjeta de la línea es lo que más falta: la medida antes que
/// el ángulo.
fn lo_que_falta(pieza: &super::PiezaDeSerie) -> Vec<String> {
    let nombre = nombre_legible(&pieza.perfil);
    let medida = match (pieza.cantidad.es_confirmar(), pieza.longitud.es_confirmar()) {
        (true, true) => Some("la cantidad y la medida de corte"),
        (true, false) => Some("la cantidad"),
        (false, true) => Some("la medida de corte"),
        (false, false) => None,
    };
    let angulo = pieza.corte.es_confirmar().then_some("el ángulo de corte");
    medida
        .into_iter()
        .chain(angulo)
        .map(|falta| format!("{nombre}: falta {falta}."))
        .collect()
}

fn comprobar_medidas(ancho: Decimal, alto: Decimal) -> Result<(), ErrorCalculo> {
    for (nombre, medida) in [("ancho", ancho), ("alto", alto)] {
        if !medida.es_positivo() {
            return Err(ErrorCalculo::MedidaNoPositiva(nombre));
        }
    }
    Ok(())
}

fn pendientes_de_estado(serie: &Serie, tipologia: &Tipologia) -> Vec<String> {
    let estado = tipologia.estado.unwrap_or(serie.estado);
    match (serie.estado, estado) {
        (EstadoSerie::Demo, _) => vec!["Serie de demostración con datos ficticios.".into()],
        (_, EstadoSerie::Validada) if serie.estado == EstadoSerie::Validada => Vec::new(),
        _ => vec!["La serie está en borrador: falta validarla con ventanas reales.".into()],
    }
}

pub(super) struct Calculo<'a> {
    serie: &'a Serie,
    pub(super) valores: Valores,
}

impl<'a> Calculo<'a> {
    fn new(serie: &'a Serie, ancho: Decimal, alto: Decimal) -> Calculo<'a> {
        let mut valores = serie.parametros.clone();
        valores.insert("ancho".into(), ancho);
        valores.insert("alto".into(), alto);
        Calculo { serie, valores }
    }

    pub(super) fn evaluar(&self, formula: &Formula, donde: &str) -> Result<Decimal, ErrorCalculo> {
        formula
            .evaluar(&self.valores)
            .map_err(|error| ErrorCalculo::Formula {
                donde: donde.to_owned(),
                error,
            })
    }

    /// Una fórmula con un nombre sin valor: el `alto` de una ventana con persiana mientras falte
    /// el alto del cajón (ADR-0032). Lo que la use queda pendiente, sin inventar la medida.
    pub(super) fn falta_dato(&self, formula: &Formula) -> bool {
        formula
            .variables()
            .iter()
            .any(|nombre| !self.valores.contains_key(nombre))
    }

    pub(super) fn cantidad(&self, formula: &Formula, donde: &str) -> Result<u32, ErrorCalculo> {
        let valor = self.evaluar(formula, donde)?;
        valor
            .a_entero()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| (1..=MAXIMO_POR_PIEZA).contains(n))
            .ok_or_else(|| ErrorCalculo::CantidadNoValida {
                donde: donde.to_owned(),
                valor: valor.to_string(),
            })
    }

    /// Una pieza con cantidad 0 no la lleva esta medida (ADR-0031): con `si()` la serie elige
    /// entre dos perfiles, como el cajón de 45 o de 50 de una mosquitera según el alto.
    fn cantidad_o_ninguna(
        &self,
        formula: &Formula,
        donde: &str,
    ) -> Result<Option<u32>, ErrorCalculo> {
        if self.evaluar(formula, donde)? == Decimal::CERO {
            return Ok(None);
        }
        self.cantidad(formula, donde).map(Some)
    }

    /// El redondeo de la serie se aplica una sola vez, al resultado de la fórmula.
    pub(super) fn longitud(
        &self,
        formula: &Formula,
        donde: &str,
    ) -> Result<Longitud, ErrorCalculo> {
        let valor = self.evaluar(formula, donde)?;
        self.a_longitud(valor, donde)
    }

    pub(super) fn a_longitud(&self, valor: Decimal, donde: &str) -> Result<Longitud, ErrorCalculo> {
        let no_valida = || ErrorCalculo::MedidaNoValida {
            donde: donde.to_owned(),
            valor: valor.to_string(),
        };
        let redondeo = &self.serie.redondeo;
        let redondeado = valor
            .redondear(redondeo.precision, redondeo.sentido)
            .map_err(|_| no_valida())?;
        Longitud::desde_decimal(redondeado, self.serie.unidad)
            .ok()
            .filter(|l| *l > Longitud::CERO)
            .ok_or_else(no_valida)
    }

    /// Una pieza sin medida no se puede cortar, pero se lista como pendiente con su número. Se
    /// numeran las piezas que lleva esta medida, también las pendientes: así el número no cambia
    /// cuando llegue la medida, y no salta cuando una pieza no va (cantidad 0).
    fn piezas(
        &self,
        tipologia: &Tipologia,
        pendientes: &mut Vec<String>,
    ) -> Result<(Vec<PiezaCalculada>, Vec<PiezaPendiente>), ErrorCalculo> {
        let (mut piezas, mut sin_medida) = (Vec::new(), Vec::new());
        let mut numero = 0;
        for pieza in &tipologia.piezas {
            let cantidad = match &pieza.cantidad {
                Dato::Valor(formula) if self.falta_dato(formula) => None,
                Dato::Valor(formula) => match self.cantidad_o_ninguna(formula, &pieza.perfil)? {
                    Some(n) => Some(n),
                    None => continue,
                },
                Dato::Confirmar => None,
            };
            numero += 1;
            pendientes.extend(lo_que_falta(pieza));
            let longitud = pieza.longitud.valor().filter(|f| !self.falta_dato(f));
            let (Some(cantidad), Some(longitud)) = (cantidad, longitud) else {
                sin_medida.push(PiezaPendiente {
                    numero,
                    perfil: pieza.perfil.clone(),
                    cantidad,
                    posicion: pieza.posicion.clone(),
                });
                continue;
            };
            piezas.push(PiezaCalculada {
                numero,
                perfil: pieza.perfil.clone(),
                cantidad,
                longitud: self.longitud(longitud, &pieza.perfil)?,
                corte: pieza.corte.valor().copied(),
                formula: longitud.texto().to_owned(),
                posicion: pieza.posicion.clone(),
            });
        }
        Ok((piezas, sin_medida))
    }

    fn accesorios(
        &self,
        tipologia: &Tipologia,
        pendientes: &mut Vec<String>,
    ) -> Result<Vec<AccesorioCalculado>, ErrorCalculo> {
        let mut accesorios = Vec::new();
        for accesorio in &tipologia.accesorios {
            let Some(formula) = accesorio.cantidad.valor().filter(|f| !self.falta_dato(f)) else {
                pendientes.push(format!("{}: falta la cantidad.", accesorio.articulo));
                continue;
            };
            let cantidad = match accesorio.unidad_cantidad {
                UnidadCantidad::Unidad => {
                    Decimal::desde_entero(i64::from(self.cantidad(formula, &accesorio.articulo)?))
                }
                UnidadCantidad::Metro => self.evaluar(formula, &accesorio.articulo)?,
            };
            accesorios.push(AccesorioCalculado {
                articulo: accesorio.articulo.clone(),
                cantidad,
                aplica_a: accesorio.aplica_a,
                unidad_cantidad: accesorio.unidad_cantidad,
            });
        }
        Ok(accesorios)
    }
}
