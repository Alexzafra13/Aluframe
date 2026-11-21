//! Lectura del formulario del editor de línea y vuelta a texto de una línea guardada.

use std::collections::BTreeMap;

use dominio::unidades::{Dinero, Longitud, Porcentaje, Unidad};

use super::presupuesto_acciones::leer_porcentaje;
use crate::aplicacion::ajustes_presupuesto::leer_dinero;
use crate::aplicacion::calculo_de_ventana::EntradaDeVentana;
use crate::aplicacion::datos_del_editor::{leer_seleccion, valor_de_seleccion};
use crate::aplicacion::series::leer_medida;
use crate::bd::consultas::lineas_presupuesto::FilaLinea;
use crate::errores::ErrorApp;
use crate::http::formulario::Campos;
use crate::vistas::{ValoresDeVentana, porcentaje_en_campo};

pub fn valores_del_formulario(c: &Campos) -> ValoresDeVentana {
    ValoresDeVentana {
        seleccion: c.texto("seleccion"),
        ancho: c.texto("ancho"),
        alto: c.texto("alto"),
        cantidad: c.texto("cantidad"),
        acabado: c.texto("acabado"),
        vidrio: c.texto("vidrio"),
        mano_de_obra: c.texto("mano_de_obra"),
        descuento: c.texto("descuento"),
        iva: c.texto("iva"),
        limites_revisados: c.marcado("limites_revisados"),
    }
}

pub fn opciones_elegidas(pares: &[(String, String)]) -> BTreeMap<String, String> {
    pares
        .iter()
        .filter_map(|(c, v)| Some((c.strip_prefix("opcion.")?.to_owned(), v.clone())))
        .collect()
}

/// Convierte lo escrito en una ventana calculable; el primer error se muestra en el editor.
pub fn leer_ventana(
    v: &ValoresDeVentana,
    opciones: BTreeMap<String, String>,
) -> Result<(EntradaDeVentana, Porcentaje, bool), ErrorApp> {
    let (serie_version_id, tipologia) = leer_seleccion(&v.seleccion)
        .ok_or_else(|| ErrorApp::Validacion("Elige la serie y la tipología.".into()))?;
    let cantidad = match v.cantidad.trim() {
        "" => 1,
        texto => texto
            .parse::<u32>()
            .ok()
            .filter(|n| (1..=999).contains(n))
            .ok_or_else(|| {
                ErrorApp::Validacion("La cantidad debe ser un número entero entre 1 y 999.".into())
            })?,
    };
    let mano_de_obra = match v.mano_de_obra.trim() {
        "" => Dinero::CERO,
        texto => leer_dinero(texto).ok_or_else(|| {
            ErrorApp::Validacion("Mano de obra: escribe un importe en euros.".into())
        })?,
    };
    let entrada = EntradaDeVentana {
        serie_version_id,
        tipologia,
        ancho: leer_medida(&v.ancho, "ancho")?,
        alto: leer_medida(&v.alto, "alto")?,
        opciones,
        acabado_id: v.acabado.parse().ok(),
        vidrio_articulo_id: v.vidrio.parse().ok(),
        cantidad,
        mano_de_obra,
        descuento: leer_porcentaje(&v.descuento, "Descuento")?,
    };
    let iva =
        Porcentaje::desde_puntos_basicos(v.iva.parse::<i64>().unwrap_or(2_100).clamp(0, 10_000));
    Ok((entrada, iva, v.limites_revisados))
}

pub fn texto_cm(dmm: Option<i64>) -> String {
    dmm.and_then(|d| Longitud::desde_decimas_de_mm(d).a_decimal(Unidad::Cm).ok())
        .map(|d| d.to_string().replace('.', ","))
        .unwrap_or_default()
}

pub fn importe_en_campo(centimos: i64) -> String {
    if centimos == 0 {
        return String::new();
    }
    Dinero::desde_centimos(centimos)
        .formatear()
        .trim_end_matches(" €")
        .to_owned()
}

pub fn valores_de_linea(l: &FilaLinea) -> ValoresDeVentana {
    ValoresDeVentana {
        seleccion: valor_de_seleccion(
            l.serie_version_id.unwrap_or_default(),
            l.tipologia.as_deref().unwrap_or_default(),
        ),
        ancho: texto_cm(l.ancho_dmm),
        alto: texto_cm(l.alto_dmm),
        cantidad: (l.cantidad_mil / 1_000).to_string(),
        acabado: l.acabado_id.map(|a| a.to_string()).unwrap_or_default(),
        vidrio: l
            .vidrio_articulo_id
            .map(|a| a.to_string())
            .unwrap_or_default(),
        mano_de_obra: importe_en_campo(l.mano_de_obra_cent),
        descuento: porcentaje_en_campo(l.descuento_bp),
        iva: l.tipo_iva_bp.to_string(),
        limites_revisados: !l.revisar_limites,
    }
}
