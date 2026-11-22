use std::collections::BTreeMap;

use super::lector::{Campos, Lector, opcional, ruta_hija};
use super::leer_tipologia::leer_tipologias;
use super::{Dato, EstadoSerie, Redondeo, Referencia, Serie, Valor};
use crate::formula::Funcion;
use crate::unidades::{Decimal, Longitud, SentidoRedondeo, Unidad};

const CAMPOS_DE_SERIE: [&str; 10] = [
    "serie",
    "fabricante",
    "proveedor",
    "estado",
    "unidades",
    "redondeo",
    "longitud_barra",
    "parametros",
    "referencias",
    "tipologias",
];

pub fn leer_cabecera_y_tipologias(lector: &mut Lector, raiz: &Valor) -> Option<Serie> {
    let campos = lector.mapa(raiz, "")?;
    lector.solo_campos(campos, &CAMPOS_DE_SERIE, "");
    let parametros = leer_parametros(lector, campos);
    let codigo = leer_codigo(lector, campos);
    let fabricante = lector
        .obligatorio(campos, "fabricante", "")
        .and_then(|v| lector.texto(v, "fabricante"));
    let proveedor = lector
        .obligatorio(campos, "proveedor", "")
        .and_then(|v| lector.dato_texto(v, "proveedor"));
    let estado = lector
        .obligatorio(campos, "estado", "")
        .and_then(|v| leer_estado(lector, v, "estado"));
    let unidad = leer_unidad(lector, campos);
    let redondeo = leer_redondeo(lector, campos);
    let longitud_barra = unidad.and_then(|u| leer_longitud_barra(lector, campos, u));
    let referencias = leer_referencias(lector, campos);
    let tipologias = lector
        .obligatorio(campos, "tipologias", "")
        .and_then(|v| leer_tipologias(lector, v));
    Some(Serie {
        codigo: codigo?,
        fabricante: fabricante?,
        proveedor: proveedor?,
        estado: estado?,
        unidad: unidad?,
        redondeo: redondeo?,
        longitud_barra: longitud_barra?,
        parametros: parametros?,
        referencias: referencias?,
        tipologias: tipologias?,
    })
}

pub fn es_codigo_valido(codigo: &str) -> bool {
    !codigo.is_empty()
        && codigo.len() <= 40
        && codigo
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn leer_codigo(lector: &mut Lector, campos: &Campos) -> Option<String> {
    let codigo = lector
        .obligatorio(campos, "serie", "")
        .and_then(|v| lector.texto(v, "serie"))?;
    let valido = es_codigo_valido(&codigo) && !codigo.bytes().any(|b| b.is_ascii_lowercase());
    if !valido {
        lector.informe.error(
            "serie",
            "usa solo MAYÚSCULAS, cifras y guion bajo (hasta 40)",
        );
        return None;
    }
    Some(codigo)
}

pub fn leer_estado(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<EstadoSerie> {
    match lector.texto(valor, ruta)?.as_str() {
        "borrador" => Some(EstadoSerie::Borrador),
        "validada" => Some(EstadoSerie::Validada),
        "demo" => Some(EstadoSerie::Demo),
        _ => {
            lector
                .informe
                .error(ruta, "debe ser borrador, validada o demo");
            None
        }
    }
}

fn leer_unidad(lector: &mut Lector, campos: &Campos) -> Option<Unidad> {
    let texto = lector
        .obligatorio(campos, "unidades", "")
        .and_then(|v| lector.texto(v, "unidades"))?;
    let unidad = Unidad::desde_texto(&texto);
    if unidad.is_none() {
        lector.informe.error("unidades", "debe ser cm o mm");
    }
    unidad
}

fn leer_redondeo(lector: &mut Lector, campos: &Campos) -> Option<Redondeo> {
    let bloque = lector.obligatorio(campos, "redondeo", "")?;
    let campos = lector.mapa(bloque, "redondeo")?;
    lector.solo_campos(campos, &["precision", "sentido"], "redondeo");
    let precision = lector
        .obligatorio(campos, "precision", "redondeo")
        .and_then(|v| lector.decimal(v, "redondeo.precision"))
        .filter(|p| p.es_positivo());
    let texto = lector
        .obligatorio(campos, "sentido", "redondeo")
        .and_then(|v| lector.texto(v, "redondeo.sentido"))?;
    let Some(sentido) = SentidoRedondeo::desde_texto(&texto) else {
        lector
            .informe
            .error("redondeo.sentido", "debe ser abajo, arriba o cercano");
        return None;
    };
    let Some(precision) = precision else {
        lector
            .informe
            .error("redondeo.precision", "debe ser un número positivo");
        return None;
    };
    Some(Redondeo { precision, sentido })
}

fn leer_longitud_barra(
    lector: &mut Lector,
    campos: &Campos,
    unidad: Unidad,
) -> Option<Dato<Longitud>> {
    let valor = lector.obligatorio(campos, "longitud_barra", "")?;
    match lector.dato_decimal(valor, "longitud_barra")? {
        Dato::Confirmar => Some(Dato::Confirmar),
        Dato::Valor(numero) => match Longitud::desde_decimal(numero, unidad) {
            Ok(longitud) if longitud > Longitud::CERO => Some(Dato::Valor(longitud)),
            _ => {
                lector
                    .informe
                    .error("longitud_barra", "debe ser una longitud positiva");
                None
            }
        },
    }
}

fn leer_parametros(lector: &mut Lector, campos: &Campos) -> Option<BTreeMap<String, Decimal>> {
    let mut parametros = BTreeMap::new();
    let Some(valor) = opcional(campos, "parametros") else {
        return Some(parametros);
    };
    let entradas = lector.mapa(valor, "parametros")?;
    for (nombre, valor) in entradas {
        let ruta = ruta_hija("parametros", nombre);
        if !es_nombre_de_parametro(nombre) {
            lector.informe.error(&ruta, "nombre no válido: minúsculas, cifras y guion bajo, sin usar ancho, alto ni funciones");
            continue;
        }
        if let Some(numero) = lector.decimal(valor, &ruta) {
            parametros.insert(nombre.clone(), numero);
            lector.variables.insert(nombre.clone());
        }
    }
    Some(parametros)
}

fn es_nombre_de_parametro(nombre: &str) -> bool {
    let reservado =
        ["ancho", "alto", "si"].contains(&nombre) || Funcion::desde_nombre(nombre).is_some();
    let primera_valida = nombre
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_lowercase());
    let resto_valido = nombre
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    !reservado && primera_valida && resto_valido
}

fn leer_referencias(lector: &mut Lector, campos: &Campos) -> Option<BTreeMap<String, Referencia>> {
    let valor = lector.obligatorio(campos, "referencias", "")?;
    let entradas = lector.mapa(valor, "referencias")?;
    let mut referencias = BTreeMap::new();
    for (codigo, valor) in entradas {
        let ruta = ruta_hija("referencias", codigo);
        if !es_codigo_valido(codigo) {
            lector
                .informe
                .error(&ruta, "código no válido: letras, cifras y guion bajo");
            continue;
        }
        let Some(campos) = lector.mapa(valor, &ruta) else {
            continue;
        };
        lector.solo_campos(campos, &["referencia", "proveedor", "silueta"], &ruta);
        let referencia = lector
            .obligatorio(campos, "referencia", &ruta)
            .and_then(|v| lector.dato_texto(v, &ruta_hija(&ruta, "referencia")));
        let proveedor = opcional(campos, "proveedor").and_then(|v| lector.texto(v, &ruta));
        let silueta = opcional(campos, "silueta").and_then(|v| leer_silueta(lector, v, &ruta));
        if let Some(referencia) = referencia {
            referencias.insert(
                codigo.clone(),
                Referencia {
                    referencia,
                    proveedor,
                    silueta,
                },
            );
        }
    }
    Some(referencias)
}

/// `fabricante/archivo` en minúsculas: nombra un archivo de `data/perfiles/` sin salir de ella.
fn leer_silueta(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<String> {
    let ruta = ruta_hija(ruta, "silueta");
    let texto = lector.texto(valor, &ruta)?;
    let parte_valida = |parte: &str| {
        !parte.is_empty()
            && parte
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    };
    match texto.split_once('/') {
        Some((fabricante, archivo)) if parte_valida(fabricante) && parte_valida(archivo) => {
            Some(texto)
        }
        _ => {
            lector.informe.error(
                &ruta,
                "escribe fabricante/archivo en minúsculas, por ejemplo aluval/21210",
            );
            None
        }
    }
}
