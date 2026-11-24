//! Persiana en la ventana (ADR-0032): la medida escrita es la del conjunto y la ventana se
//! calcula con la de su marco, sin las dos guías y sin el cajón.

use std::collections::BTreeMap;

use super::calculo::{Calculo, ErrorCalculo};
use super::{Dato, PersianaCalculada, Tipologia};
use crate::unidades::Decimal;

const LADO_DE_LA_CINTA_POR_DEFECTO: &str = "derecha";

impl Calculo<'_> {
    /// Cambia `ancho` y `alto` por los del marco. Sin el alto del cajón, `alto` se queda sin
    /// valor: lo que dependa de él sale pendiente y la ventana, incompleta.
    pub(super) fn aplicar_persiana(
        &mut self,
        tipologia: &Tipologia,
        opciones: &BTreeMap<String, String>,
        pendientes: &mut Vec<String>,
    ) -> Result<Option<PersianaCalculada>, ErrorCalculo> {
        let Some(persiana) = &tipologia.persiana else {
            return Ok(None);
        };
        let guia = self.evaluar(&persiana.guia, "Persiana (guía)")?;
        let cajon = match &persiana.cajon {
            Dato::Valor(formula) => Some(self.evaluar(formula, "Persiana (cajón)")?),
            Dato::Confirmar => None,
        };
        let dos_guias = guia
            .multiplicar(Decimal::desde_entero(2))
            .map_err(|_| no_valida("Persiana (guía)", guia))?;
        let ancho = self.restar_de("ancho", dos_guias, "ancho")?;
        let alto = cajon
            .map(|cajon| self.restar_de("alto", cajon, "alto"))
            .transpose()?;
        self.fijar_medidas_del_marco(ancho, alto, pendientes);
        let cinta = tipologia
            .valor_de_opcion(&persiana.cinta, opciones)
            .unwrap_or_else(|| LADO_DE_LA_CINTA_POR_DEFECTO.into());
        Ok(Some(PersianaCalculada {
            guia: self.a_longitud(guia, "Persiana (guía)")?,
            cajon: cajon
                .map(|c| self.a_longitud(c, "Persiana (cajón)"))
                .transpose()?,
            ancho_ventana: self.a_longitud(ancho, "Persiana (ancho de la ventana)")?,
            alto_ventana: alto
                .map(|a| self.a_longitud(a, "Persiana (alto de la ventana)"))
                .transpose()?,
            cinta,
        }))
    }

    fn fijar_medidas_del_marco(
        &mut self,
        ancho: Decimal,
        alto: Option<Decimal>,
        pendientes: &mut Vec<String>,
    ) {
        self.valores.insert("ancho".into(), ancho);
        let Some(alto) = alto else {
            self.valores.remove("alto");
            pendientes.push(
                "Persiana: falta el alto del cajón; lo que depende del alto de la ventana queda \
                 pendiente."
                    .into(),
            );
            return;
        };
        self.valores.insert("alto".into(), alto);
    }

    /// La medida del conjunto menos lo que ocupa la persiana; debe quedar algo de ventana.
    fn restar_de(
        &self,
        medida: &str,
        resta: Decimal,
        nombre: &str,
    ) -> Result<Decimal, ErrorCalculo> {
        let donde = format!("Persiana ({nombre} de la ventana)");
        let total = self.valores.get(medida).copied().unwrap_or(Decimal::CERO);
        total
            .restar(resta)
            .ok()
            .filter(|v| v.es_positivo())
            .ok_or_else(|| no_valida(&donde, total))
    }
}

fn no_valida(donde: &str, valor: Decimal) -> ErrorCalculo {
    ErrorCalculo::MedidaNoValida {
        donde: donde.to_owned(),
        valor: valor.to_string(),
    }
}
