//! Persiana en la ventana (ADR-0032): la medida escrita es la del conjunto y la ventana se
//! calcula con la de su marco, sin las dos guías y sin el cajón. Suelta en pared (ADR-0033): la
//! medida es la del hueco y no se le quita nada.

use std::collections::BTreeMap;

use super::calculo::{Calculo, ErrorCalculo};
use super::{Dato, MontajeDePersiana, PersianaCalculada, Tipologia};
use crate::formula::Formula;
use crate::unidades::Decimal;

const LADO_DE_LA_CINTA_POR_DEFECTO: &str = "derecha";

impl Calculo<'_> {
    /// En la ventana cambia `ancho` y `alto` por los del marco; sin el alto del cajón, `alto` se
    /// queda sin valor y lo que dependa de él sale pendiente. En pared no cambia nada: la medida
    /// es la del hueco y la persiana se pide hecha.
    pub(super) fn aplicar_persiana(
        &mut self,
        tipologia: &Tipologia,
        opciones: &BTreeMap<String, String>,
        pendientes: &mut Vec<String>,
    ) -> Result<Option<PersianaCalculada>, ErrorCalculo> {
        let Some(persiana) = &tipologia.persiana else {
            return Ok(None);
        };
        let guia = self.dato_opcional(&persiana.guia, "Persiana (guía)")?;
        let cajon = self.dato_opcional(&persiana.cajon, "Persiana (cajón)")?;
        let (ancho, alto) = match persiana.montaje {
            MontajeDePersiana::Ventana => self.medidas_del_marco(guia, cajon, pendientes)?,
            MontajeDePersiana::Pared => self.medidas_del_hueco(guia, cajon, pendientes),
        };
        let cinta = tipologia
            .valor_de_opcion(&persiana.cinta, opciones)
            .unwrap_or_else(|| LADO_DE_LA_CINTA_POR_DEFECTO.into());
        let longitud = |valor: Option<Decimal>, donde: &str| {
            valor.map(|v| self.a_longitud(v, donde)).transpose()
        };
        Ok(Some(PersianaCalculada {
            montaje: persiana.montaje,
            guia: longitud(guia, "Persiana (guía)")?,
            cajon: longitud(cajon, "Persiana (cajón)")?,
            ancho_ventana: self.a_longitud(ancho, "Persiana (ancho)")?,
            alto_ventana: longitud(alto, "Persiana (alto)")?,
            cinta,
        }))
    }

    fn dato_opcional(
        &self,
        dato: &Dato<Formula>,
        donde: &str,
    ) -> Result<Option<Decimal>, ErrorCalculo> {
        match dato {
            Dato::Valor(formula) => self.evaluar(formula, donde).map(Some),
            Dato::Confirmar => Ok(None),
        }
    }

    /// Persiana en la ventana: el marco es el conjunto sin las dos guías y sin el cajón.
    fn medidas_del_marco(
        &mut self,
        guia: Option<Decimal>,
        cajon: Option<Decimal>,
        pendientes: &mut Vec<String>,
    ) -> Result<(Decimal, Option<Decimal>), ErrorCalculo> {
        // Al leer la serie ya se exige la guía en la ventana.
        let guia = guia.ok_or_else(|| no_valida("Persiana (guía)", Decimal::CERO))?;
        let dos_guias = guia
            .multiplicar(Decimal::desde_entero(2))
            .map_err(|_| no_valida("Persiana (guía)", guia))?;
        let ancho = self.restar_de("ancho", dos_guias, "ancho")?;
        let alto = cajon
            .map(|cajon| self.restar_de("alto", cajon, "alto"))
            .transpose()?;
        self.fijar_medidas_del_marco(ancho, alto, pendientes);
        Ok((ancho, alto))
    }

    /// Persiana en pared: la medida es el hueco; lo que falte solo impide el precio final.
    fn medidas_del_hueco(
        &self,
        guia: Option<Decimal>,
        cajon: Option<Decimal>,
        pendientes: &mut Vec<String>,
    ) -> (Decimal, Option<Decimal>) {
        if guia.is_none() {
            pendientes.push("Persiana: falta el ancho de la guía de L (80 o 100 mm).".into());
        }
        if cajon.is_none() {
            pendientes.push("Persiana: falta el alto del cajón.".into());
        }
        let medida = |nombre: &str| self.valores.get(nombre).copied().unwrap_or(Decimal::CERO);
        (medida("ancho"), Some(medida("alto")))
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
