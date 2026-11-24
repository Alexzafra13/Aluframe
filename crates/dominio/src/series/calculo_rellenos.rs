//! Medidas que se escriben en la línea, cristales, chapas y parteluces (ADR-0030).

use std::collections::BTreeMap;

use super::calculo::{Calculo, ErrorCalculo};
use super::{ChapaCalculada, Dato, ParteLucesCalculado, Tipologia, VidrioCalculado};
use crate::formula::Formula;
use crate::unidades::{Decimal, Longitud, leer_decimal_es};

/// El valor de cada opción de tipo medida: el escrito o, si no hay o no se pregunta, el que trae
/// la serie. Fuera de su rango es un error, no un aviso: la medida la escribe quien presupuesta.
pub fn medidas_de_linea(
    tipologia: &Tipologia,
    elegidas: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Decimal>, ErrorCalculo> {
    let mut medidas = BTreeMap::new();
    for opcion in &tipologia.opciones {
        let Some(rango) = &opcion.medida else {
            continue;
        };
        let escrita = elegidas
            .get(&opcion.codigo)
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            // Si no se pregunta, lo que quede escrito en el campo oculto no cuenta.
            .filter(|_| tipologia.se_pregunta(opcion, elegidas));
        let texto = escrita.unwrap_or(&opcion.por_defecto);
        let fuera = || ErrorCalculo::MedidaDeOpcion {
            nombre: opcion.nombre.clone(),
            minimo: rango.minimo.to_string(),
            maximo: rango.maximo.to_string(),
        };
        let valor = leer_decimal_es(texto).map_err(|_| fuera())?;
        if valor < rango.minimo || valor > rango.maximo {
            return Err(fuera());
        }
        medidas.insert(opcion.codigo.clone(), valor);
    }
    Ok(medidas)
}

impl Calculo<'_> {
    fn medida_opcional(
        &self,
        dato: &Dato<Formula>,
        donde: &str,
    ) -> Result<Option<Longitud>, ErrorCalculo> {
        match dato {
            Dato::Valor(formula) if !self.falta_dato(formula) => {
                self.longitud(formula, donde).map(Some)
            }
            _ => Ok(None),
        }
    }

    pub(super) fn vidrio(
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
            let donde = match &pieza.nombre {
                Some(nombre) => format!("Vidrio {}", nombre.to_lowercase()),
                None => format!("Vidrio {}", indice + 1),
            };
            let ancho = self.medida_opcional(&pieza.ancho, &format!("{donde} (ancho)"))?;
            let alto = self.medida_opcional(&pieza.alto, &format!("{donde} (alto)"))?;
            let (Some(ancho), Some(alto)) = (ancho, alto) else {
                pendientes.push(format!("{donde}: falta el descuento del cristal."));
                continue;
            };
            piezas.push(VidrioCalculado {
                tipo: vidrio.tipo.clone(),
                nombre: pieza.nombre.clone(),
                cantidad: self.cantidad(&pieza.cantidad, &donde)?,
                ancho,
                alto,
            });
        }
        Ok(Some(piezas))
    }

    pub(super) fn chapas(
        &self,
        tipologia: &Tipologia,
        pendientes: &mut Vec<String>,
    ) -> Result<Vec<ChapaCalculada>, ErrorCalculo> {
        let mut chapas = Vec::new();
        for chapa in &tipologia.chapas {
            let donde = chapa
                .nombre
                .clone()
                .unwrap_or_else(|| chapa.articulo.clone());
            let Some(cantidad) = chapa.cantidad.valor().filter(|f| !self.falta_dato(f)) else {
                pendientes.push(format!("{donde}: falta la cantidad."));
                continue;
            };
            let ancho = self.medida_opcional(&chapa.ancho, &format!("{donde} (ancho)"))?;
            let alto = self.medida_opcional(&chapa.alto, &format!("{donde} (alto)"))?;
            if ancho.is_none() || alto.is_none() {
                pendientes.push(format!("{donde}: falta la medida de la chapa."));
            }
            chapas.push(ChapaCalculada {
                articulo: chapa.articulo.clone(),
                nombre: chapa.nombre.clone(),
                cantidad: self.cantidad(cantidad, &donde)?,
                ancho,
                alto,
            });
        }
        Ok(chapas)
    }

    /// Sin la altura real el dibujo coloca el parteluces a la medida de abajo y lo dice.
    pub(super) fn parteluces(
        &self,
        tipologia: &Tipologia,
        pendientes: &mut Vec<String>,
    ) -> Result<Option<ParteLucesCalculado>, ErrorCalculo> {
        let Some(parteluces) = &tipologia.parteluces else {
            return Ok(None);
        };
        let abajo = self.longitud(&parteluces.abajo, "Parteluces (medida de abajo)")?;
        let altura = self.medida_opcional(&parteluces.altura, "Parteluces (altura)")?;
        if altura.is_none() {
            pendientes.push("Parteluces: falta su altura real en la ventana.".into());
        }
        Ok(Some(ParteLucesCalculado { abajo, altura }))
    }
}
