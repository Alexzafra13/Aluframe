use super::{error, obligatorio};
use crate::clientes::ErrorDeCampo;
use crate::tarifa::{TipoArticulo, UnidadDeVenta};
use crate::unidades::{Decimal, Longitud, Unidad, leer_decimal_es};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntradaDeArticulo {
    pub referencia: String,
    pub descripcion: String,
    pub tipo: String,
    pub unidad: String,
    pub longitud_barra_cm: String,
    pub unidades_por_caja: String,
    pub con_acabado: bool,
    pub stock_minimo: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatosDeArticulo {
    pub referencia: String,
    pub descripcion: String,
    pub tipo: TipoArticulo,
    pub unidad: UnidadDeVenta,
    pub longitud_barra: Option<Longitud>,
    pub unidades_por_caja: Option<i64>,
    pub con_acabado: bool,
    /// Stock mínimo en milésimas de la unidad de venta (DATOS 7.1).
    pub stock_minimo_mil: i64,
}

pub fn validar_articulo(entrada: &EntradaDeArticulo) -> Result<DatosDeArticulo, Vec<ErrorDeCampo>> {
    let mut errores = Vec::new();
    let referencia = obligatorio(
        "referencia",
        &entrada.referencia,
        (60, "escribe la referencia del proveedor"),
        &mut errores,
    );
    let descripcion = obligatorio(
        "descripcion",
        &entrada.descripcion,
        (200, "escribe una descripción"),
        &mut errores,
    );
    let tipo = TipoArticulo::desde_texto(&entrada.tipo).unwrap_or_else(|| {
        errores.push(error("tipo", "elige el tipo de artículo"));
        TipoArticulo::Otro
    });
    let unidad = UnidadDeVenta::desde_texto(&entrada.unidad);
    if unidad.is_none() {
        errores.push(error("unidad", "elige la unidad en que se vende"));
    }
    let unidad = unidad.unwrap_or(UnidadDeVenta::Unidad);
    let longitud_barra = leer_longitud(&entrada.longitud_barra_cm, unidad, &mut errores);
    let unidades_por_caja = leer_caja(&entrada.unidades_por_caja, unidad, &mut errores);
    let stock_minimo_mil = leer_stock_minimo(&entrada.stock_minimo, &mut errores);
    if !errores.is_empty() {
        return Err(errores);
    }
    Ok(DatosDeArticulo {
        referencia,
        descripcion,
        tipo,
        unidad,
        longitud_barra,
        unidades_por_caja,
        con_acabado: entrada.con_acabado,
        stock_minimo_mil,
    })
}

fn leer_longitud(
    texto: &str,
    unidad: UnidadDeVenta,
    errores: &mut Vec<ErrorDeCampo>,
) -> Option<Longitud> {
    if !unidad.es_barra() {
        return None;
    }
    let longitud = leer_decimal_es(texto).and_then(|cm| Longitud::desde_decimal(cm, Unidad::Cm));
    match longitud {
        Ok(l) if l > Longitud::CERO => Some(l),
        _ if texto.trim().is_empty() => {
            errores.push(error(
                "longitud_barra_cm",
                "un artículo por barras necesita la longitud de barra (en cm)",
            ));
            None
        }
        _ => {
            errores.push(error(
                "longitud_barra_cm",
                "escribe la longitud en cm, por ejemplo 640",
            ));
            None
        }
    }
}

fn leer_caja(texto: &str, unidad: UnidadDeVenta, errores: &mut Vec<ErrorDeCampo>) -> Option<i64> {
    if texto.trim().is_empty() || unidad != UnidadDeVenta::Caja {
        return None;
    }
    match texto.trim().parse::<i64>() {
        Ok(n) if n > 0 => Some(n),
        _ => {
            errores.push(error(
                "unidades_por_caja",
                "escribe cuántas unidades trae la caja (un número entero)",
            ));
            None
        }
    }
}

fn leer_stock_minimo(texto: &str, errores: &mut Vec<ErrorDeCampo>) -> i64 {
    if texto.trim().is_empty() {
        return 0;
    }
    let mil = leer_decimal_es(texto)
        .ok()
        .and_then(|d| d.multiplicar(Decimal::desde_entero(1_000)).ok())
        .and_then(Decimal::a_entero)
        .filter(|m| *m >= 0);
    mil.unwrap_or_else(|| {
        errores.push(error(
            "stock_minimo",
            "escribe una cantidad positiva con hasta 3 decimales",
        ));
        0
    })
}
