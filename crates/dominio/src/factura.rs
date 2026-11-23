//! Facturas y cobros (H6): importe de cada línea, estado de cobro a partir de los cobros y lo
//! que conviene avisar antes de emitir. Los totales e IVA salen de `precio::calcular_totales`.

use crate::precio::aplicar_porcentaje;
use crate::unidades::{Dinero, Porcentaje, PrecioUnitario};

/// Cantidad × precio unitario, menos el descuento de la línea (igual que en el presupuesto).
pub fn importe_de_linea(
    precio: PrecioUnitario,
    cantidad_mil: i64,
    descuento: Porcentaje,
) -> Option<Dinero> {
    let bruto = precio.importe(cantidad_mil)?;
    aplicar_porcentaje(
        bruto,
        Porcentaje::desde_puntos_basicos(-descuento.puntos_basicos()),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDeCobro {
    Pendiente,
    Parcial,
    Pagada,
    Vencida,
    /// Sus rectificativas emitidas la dejan a cero: ya no hay nada que cobrar.
    Anulada,
}

impl EstadoDeCobro {
    pub fn nombre(self) -> &'static str {
        match self {
            EstadoDeCobro::Pendiente => "Pendiente",
            EstadoDeCobro::Parcial => "Parcial",
            EstadoDeCobro::Pagada => "Pagada",
            EstadoDeCobro::Vencida => "Vencida",
            EstadoDeCobro::Anulada => "Anulada",
        }
    }
}

/// Lo que cuenta para cobrar una factura ordinaria emitida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportesDeFactura {
    pub total: Dinero,
    /// La suma de sus rectificativas emitidas: negativa si le quitan importe.
    pub rectificado: Dinero,
    pub cobrado: Dinero,
}

impl ImportesDeFactura {
    /// El total que se debe de verdad, una vez aplicadas sus rectificativas.
    pub fn a_cobrar(self) -> Dinero {
        Dinero::desde_centimos(
            self.total
                .centimos()
                .saturating_add(self.rectificado.centimos()),
        )
    }

    /// Lo que falta por cobrar; nunca negativo (lo cobrado de más se ve en la ficha).
    pub fn pendiente(self) -> Dinero {
        let falta = self
            .a_cobrar()
            .centimos()
            .saturating_sub(self.cobrado.centimos());
        Dinero::desde_centimos(falta.max(0))
    }
}

/// Se calcula siempre con los cobros y las rectificativas, nunca se guarda (DATOS 3.8). Vence si
/// pasó la fecha de vencimiento (AAAA-MM-DD, en Madrid) sin cobrarse entera.
pub fn estado_de_cobro(
    importes: ImportesDeFactura,
    (vencimiento, hoy): (Option<&str>, &str),
) -> EstadoDeCobro {
    let (a_cobrar, cobrado) = (importes.a_cobrar(), importes.cobrado);
    if importes.rectificado.centimos() != 0 && a_cobrar.centimos() <= 0 {
        return EstadoDeCobro::Anulada;
    }
    if a_cobrar.centimos() <= 0 || cobrado >= a_cobrar {
        return EstadoDeCobro::Pagada;
    }
    if vencimiento.is_some_and(|v| v < hoy) {
        return EstadoDeCobro::Vencida;
    }
    if cobrado.centimos() > 0 {
        EstadoDeCobro::Parcial
    } else {
        EstadoDeCobro::Pendiente
    }
}

/// Lo que una factura completa suele llevar del cliente (SPEC, H6): se avisa, no se bloquea.
pub fn avisos_para_emitir(nombre: Option<&str>, nif: Option<&str>) -> Vec<&'static str> {
    let vacio = |t: Option<&str>| t.is_none_or(|t| t.trim().is_empty());
    let mut avisos = Vec::new();
    if vacio(nombre) {
        avisos.push("el cliente no tiene nombre o razón social");
    }
    if vacio(nif) {
        avisos.push("el cliente no tiene NIF (puede valer una factura simplificada: confírmalo con tu gestor)");
    }
    avisos
}

/// «F-2026-0001»; la rectificativa usa la serie «R».
pub fn codigo_de_factura(serie: &str, anio: i64, numero: Option<i64>) -> String {
    match numero {
        Some(n) => format!("{serie}-{anio}-{n:04}"),
        None => "Borrador".to_owned(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn euros(e: i64) -> Dinero {
        Dinero::desde_centimos(e * 100)
    }

    #[test]
    fn el_importe_de_la_linea_aplica_cantidad_y_descuento() {
        let precio = PrecioUnitario::desde_texto_es("80").unwrap();
        assert_eq!(
            importe_de_linea(precio, 2_000, Porcentaje::CERO),
            Some(euros(160))
        );
        let diez = Porcentaje::desde_puntos_basicos(1_000);
        assert_eq!(importe_de_linea(precio, 2_000, diez), Some(euros(144)));
        assert_eq!(
            importe_de_linea(precio, -1_000, Porcentaje::CERO),
            Some(euros(-80))
        );
    }

    fn importes(total: i64, rectificado: i64, cobrado: i64) -> ImportesDeFactura {
        ImportesDeFactura {
            total: euros(total),
            rectificado: euros(rectificado),
            cobrado: euros(cobrado),
        }
    }

    #[test]
    fn el_estado_sale_de_lo_cobrado_y_del_vencimiento() {
        let hoy = "2026-11-22";
        assert_eq!(
            estado_de_cobro(importes(100, 0, 0), (None, hoy)),
            EstadoDeCobro::Pendiente
        );
        assert_eq!(
            estado_de_cobro(importes(100, 0, 40), (None, hoy)),
            EstadoDeCobro::Parcial
        );
        assert_eq!(
            estado_de_cobro(importes(100, 0, 100), (None, hoy)),
            EstadoDeCobro::Pagada
        );
        assert_eq!(
            estado_de_cobro(importes(100, 0, 40), (Some("2026-11-21"), hoy)),
            EstadoDeCobro::Vencida
        );
        assert_eq!(
            estado_de_cobro(importes(100, 0, 40), (Some(hoy), hoy)),
            EstadoDeCobro::Parcial
        );
        assert_eq!(
            estado_de_cobro(importes(-50, 0, 0), (None, hoy)),
            EstadoDeCobro::Pagada,
            "una rectificativa no se cobra"
        );
        assert_eq!(EstadoDeCobro::Vencida.nombre(), "Vencida");
    }

    /// Fallo corregido: una factura anulada con una rectificativa seguía «Vencida» en el Inicio.
    #[test]
    fn las_rectificativas_emitidas_cuentan_al_cobrar() {
        let vencida = (Some("2026-01-31"), "2026-11-22");
        let anulada = importes(100, -100, 0);
        assert_eq!(estado_de_cobro(anulada, vencida), EstadoDeCobro::Anulada);
        assert_eq!(anulada.pendiente(), Dinero::CERO);
        assert_eq!(
            estado_de_cobro(importes(100, -100, 100), vencida),
            EstadoDeCobro::Anulada,
            "cobrada y después anulada: lo cobrado se ve en la ficha"
        );
        let rebajada = importes(100, -20, 0);
        assert_eq!(rebajada.a_cobrar(), euros(80));
        assert_eq!(rebajada.pendiente(), euros(80));
        assert_eq!(estado_de_cobro(rebajada, vencida), EstadoDeCobro::Vencida);
        assert_eq!(
            estado_de_cobro(importes(100, -20, 80), vencida),
            EstadoDeCobro::Pagada,
            "lo que queda tras la rectificativa está cobrado"
        );
        assert_eq!(
            importes(100, 30, 100).pendiente(),
            euros(30),
            "una rectificativa al alza aumenta lo que se debe"
        );
        assert_eq!(importes(100, 0, 150).pendiente(), Dinero::CERO);
        assert_eq!(EstadoDeCobro::Anulada.nombre(), "Anulada");
    }

    #[test]
    fn avisa_sin_bloquear_de_lo_que_falta_y_numera() {
        assert_eq!(avisos_para_emitir(Some("Ana"), Some("12345678Z")).len(), 0);
        assert_eq!(avisos_para_emitir(Some(" "), None).len(), 2);
        assert_eq!(codigo_de_factura("F", 2026, Some(7)), "F-2026-0007");
        assert_eq!(codigo_de_factura("R", 2026, None), "Borrador");
    }
}
