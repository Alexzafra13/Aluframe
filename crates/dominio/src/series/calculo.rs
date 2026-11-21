use thiserror::Error;

use super::resultado::MAXIMO_POR_PIEZA;
use super::{
    AccesorioCalculado, Dato, EstadoSerie, PiezaCalculada, ResultadoVentana, Serie, Tipologia,
    UnidadCantidad, VidrioCalculado,
};
use crate::formula::{ErrorFormula, Formula, Valores};
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
}

pub fn calcular_ventana(
    serie: &Serie,
    codigo_tipologia: &str,
    ancho: Decimal,
    alto: Decimal,
) -> Result<ResultadoVentana, ErrorCalculo> {
    let tipologia = serie
        .tipologia(codigo_tipologia)
        .ok_or_else(|| ErrorCalculo::TipologiaDesconocida(codigo_tipologia.to_owned()))?;
    for (nombre, medida) in [("ancho", ancho), ("alto", alto)] {
        if !medida.es_positivo() {
            return Err(ErrorCalculo::MedidaNoPositiva(nombre));
        }
    }
    let calculo = Calculo::new(serie, ancho, alto);
    let mut pendientes = pendientes_de_estado(serie, tipologia);
    let piezas = calculo.piezas(tipologia, &mut pendientes)?;
    let vidrio = calculo.vidrio(tipologia, &mut pendientes)?;
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
        accesorios,
        vidrio,
        pendientes,
        fuera_de_limites: super::validacion::comprobar_limites(tipologia, ancho, alto),
    })
}

fn pendientes_de_estado(serie: &Serie, tipologia: &Tipologia) -> Vec<String> {
    let estado = tipologia.estado.unwrap_or(serie.estado);
    match (serie.estado, estado) {
        (EstadoSerie::Demo, _) => vec!["Serie de demostración con datos ficticios.".into()],
        (_, EstadoSerie::Validada) if serie.estado == EstadoSerie::Validada => Vec::new(),
        _ => vec!["La serie está en borrador: falta validarla con ventanas reales.".into()],
    }
}

struct Calculo<'a> {
    serie: &'a Serie,
    valores: Valores,
}

impl<'a> Calculo<'a> {
    fn new(serie: &'a Serie, ancho: Decimal, alto: Decimal) -> Calculo<'a> {
        let mut valores = serie.parametros.clone();
        valores.insert("ancho".into(), ancho);
        valores.insert("alto".into(), alto);
        Calculo { serie, valores }
    }

    fn evaluar(&self, formula: &Formula, donde: &str) -> Result<Decimal, ErrorCalculo> {
        formula
            .evaluar(&self.valores)
            .map_err(|error| ErrorCalculo::Formula {
                donde: donde.to_owned(),
                error,
            })
    }

    fn cantidad(&self, formula: &Formula, donde: &str) -> Result<u32, ErrorCalculo> {
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

    /// El redondeo de la serie se aplica una sola vez, al resultado de la fórmula.
    fn longitud(&self, formula: &Formula, donde: &str) -> Result<Longitud, ErrorCalculo> {
        let valor = self.evaluar(formula, donde)?;
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

    fn piezas(
        &self,
        tipologia: &Tipologia,
        pendientes: &mut Vec<String>,
    ) -> Result<Vec<PiezaCalculada>, ErrorCalculo> {
        let mut piezas = Vec::new();
        for pieza in &tipologia.piezas {
            let (Dato::Valor(cantidad), Dato::Valor(longitud)) = (&pieza.cantidad, &pieza.longitud)
            else {
                pendientes.push(format!("{}: falta la cantidad o la medida.", pieza.perfil));
                continue;
            };
            if pieza.corte.es_confirmar() {
                pendientes.push(format!("{}: falta el ángulo de corte.", pieza.perfil));
            }
            let numero = u32::try_from(piezas.len() + 1).unwrap_or(u32::MAX);
            piezas.push(PiezaCalculada {
                numero,
                perfil: pieza.perfil.clone(),
                cantidad: self.cantidad(cantidad, &pieza.perfil)?,
                longitud: self.longitud(longitud, &pieza.perfil)?,
                corte: pieza.corte.valor().copied(),
                formula: longitud.texto().to_owned(),
            });
        }
        Ok(piezas)
    }

    fn vidrio(
        &self,
        tipologia: &Tipologia,
        pendientes: &mut Vec<String>,
    ) -> Result<Option<Vec<VidrioCalculado>>, ErrorCalculo> {
        let Dato::Valor(vidrio) = &tipologia.vidrio else {
            pendientes.push("Vidrio: falta el descuento del cristal.".into());
            return Ok(None);
        };
        let mut piezas = Vec::new();
        for (indice, pieza) in vidrio.piezas.iter().enumerate() {
            let donde = format!("Vidrio {}", indice + 1);
            piezas.push(VidrioCalculado {
                tipo: vidrio.tipo.clone(),
                cantidad: self.cantidad(&pieza.cantidad, &donde)?,
                ancho: self.longitud(&pieza.ancho, &format!("{donde} (ancho)"))?,
                alto: self.longitud(&pieza.alto, &format!("{donde} (alto)"))?,
            });
        }
        Ok(Some(piezas))
    }

    fn accesorios(
        &self,
        tipologia: &Tipologia,
        pendientes: &mut Vec<String>,
    ) -> Result<Vec<AccesorioCalculado>, ErrorCalculo> {
        let mut accesorios = Vec::new();
        for accesorio in &tipologia.accesorios {
            let Dato::Valor(formula) = &accesorio.cantidad else {
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
