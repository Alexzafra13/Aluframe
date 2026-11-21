use std::collections::HashMap;

use super::{TipoArticulo, UnidadDeVenta};
use crate::csv::{FilaCsv, TablaCsv, leer_csv};
use crate::informe::Informe;
use crate::unidades::{Longitud, PrecioUnitario, Unidad, leer_decimal_es};

pub const COLUMNAS_DE_TARIFA: [&str; 6] = [
    "referencia",
    "descripcion",
    "unidad",
    "precio_tarifa",
    "longitud_barra",
    "tipo",
];
const OBLIGATORIAS: [&str; 4] = ["referencia", "descripcion", "unidad", "precio_tarifa"];
const LARGO_REFERENCIA: usize = 60;
const LARGO_DESCRIPCION: usize = 200;

/// Una fila válida: precio del acabado base por unidad de venta y longitud de barra en cm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaDeTarifa {
    pub linea: usize,
    pub referencia: String,
    pub descripcion: String,
    pub unidad: UnidadDeVenta,
    pub precio: PrecioUnitario,
    pub longitud_barra: Option<Longitud>,
    pub tipo: Option<TipoArticulo>,
}

/// Lee todas las filas y anota todos los problemas a la vez; con errores no se puede importar.
pub fn leer_tarifa(texto: &str) -> (Vec<FilaDeTarifa>, Informe) {
    let mut informe = Informe::default();
    let tabla = match leer_csv(texto) {
        Ok(tabla) => tabla,
        Err(error) => {
            informe.error("", error.to_string());
            return (Vec::new(), informe);
        }
    };
    if !comprobar_cabecera(&tabla, &mut informe) {
        return (Vec::new(), informe);
    }
    let columnas = Columnas::de(&tabla);
    let mut filas = Vec::with_capacity(tabla.filas.len());
    let mut vistas: HashMap<String, usize> = HashMap::new();
    for fila in &tabla.filas {
        let Some(leida) = leer_fila(fila, &columnas, &mut informe) else {
            continue;
        };
        if let Some(anterior) = vistas.insert(leida.referencia.clone(), leida.linea) {
            let mensaje = format!(
                "la referencia {} ya aparece en la línea {anterior}",
                leida.referencia
            );
            informe.error(&linea(fila), mensaje);
            continue;
        }
        filas.push(leida);
    }
    if tabla.filas.is_empty() {
        informe.error("", "el archivo solo tiene la cabecera: no hay precios");
    }
    (filas, informe)
}

fn comprobar_cabecera(tabla: &TablaCsv, informe: &mut Informe) -> bool {
    let faltan: Vec<&str> = OBLIGATORIAS
        .into_iter()
        .filter(|c| tabla.columna(c).is_none())
        .collect();
    for sobrante in tabla.columnas_desconocidas(&COLUMNAS_DE_TARIFA) {
        informe.aviso(
            "cabecera",
            format!("la columna «{sobrante}» no se usa y se ignora"),
        );
    }
    if faltan.is_empty() {
        return true;
    }
    let lista = faltan.join(", ");
    informe.error(
        "cabecera",
        format!(
            "faltan columnas: {lista}. La plantilla es: {}",
            COLUMNAS_DE_TARIFA.join(";")
        ),
    );
    false
}

struct Columnas {
    referencia: Option<usize>,
    descripcion: Option<usize>,
    unidad: Option<usize>,
    precio: Option<usize>,
    longitud: Option<usize>,
    tipo: Option<usize>,
}

impl Columnas {
    fn de(tabla: &TablaCsv) -> Columnas {
        Columnas {
            referencia: tabla.columna("referencia"),
            descripcion: tabla.columna("descripcion"),
            unidad: tabla.columna("unidad"),
            precio: tabla.columna("precio_tarifa"),
            longitud: tabla.columna("longitud_barra"),
            tipo: tabla.columna("tipo"),
        }
    }
}

fn linea(fila: &FilaCsv) -> String {
    format!("línea {}", fila.linea)
}

fn leer_fila(fila: &FilaCsv, columnas: &Columnas, informe: &mut Informe) -> Option<FilaDeTarifa> {
    let donde = linea(fila);
    let errores_antes = informe.errores.len();
    let referencia = leer_texto(
        fila.campo(columnas.referencia),
        ("la referencia", LARGO_REFERENCIA),
        &donde,
        informe,
    );
    let descripcion = leer_texto(
        fila.campo(columnas.descripcion),
        ("la descripción", LARGO_DESCRIPCION),
        &donde,
        informe,
    );
    let unidad = leer_unidad(fila.campo(columnas.unidad), &donde, informe);
    let precio = leer_precio(fila.campo(columnas.precio), &donde, informe);
    let tipo = leer_tipo(fila.campo(columnas.tipo), &donde, informe);
    let longitud =
        unidad.and_then(|u| leer_longitud(fila.campo(columnas.longitud), u, &donde, informe));
    if informe.errores.len() > errores_antes {
        return None;
    }
    Some(FilaDeTarifa {
        linea: fila.linea,
        referencia: referencia?,
        descripcion: descripcion?,
        unidad: unidad?,
        precio: precio?,
        longitud_barra: longitud.flatten(),
        tipo,
    })
}

fn leer_texto(
    valor: Option<String>,
    (nombre, largo): (&str, usize),
    donde: &str,
    informe: &mut Informe,
) -> Option<String> {
    match valor {
        None => informe.error(donde, format!("falta {nombre}")),
        Some(texto) if texto.chars().count() > largo => {
            informe.error(donde, format!("{nombre} tiene más de {largo} caracteres"));
        }
        Some(texto) => return Some(texto),
    }
    None
}

fn leer_unidad(valor: Option<String>, donde: &str, informe: &mut Informe) -> Option<UnidadDeVenta> {
    let Some(texto) = valor else {
        informe.error(donde, "falta la unidad (barra, metro, m2, unidad o caja)");
        return None;
    };
    let unidad = UnidadDeVenta::desde_texto(&texto);
    if unidad.is_none() {
        let mensaje = format!("unidad desconocida «{texto}»: usa barra, metro, m2, unidad o caja");
        informe.error(donde, mensaje);
    }
    unidad
}

fn leer_precio(
    valor: Option<String>,
    donde: &str,
    informe: &mut Informe,
) -> Option<PrecioUnitario> {
    let Some(texto) = valor else {
        informe.error(donde, "falta el precio");
        return None;
    };
    match PrecioUnitario::desde_texto_es(&texto) {
        Ok(precio) if precio.es_positivo() => Some(precio),
        Ok(_) => {
            informe.error(
                donde,
                format!("el precio «{texto}» debe ser mayor que cero"),
            );
            None
        }
        Err(error) => {
            informe.error(donde, format!("precio: {error}"));
            None
        }
    }
}

fn leer_tipo(valor: Option<String>, donde: &str, informe: &mut Informe) -> Option<TipoArticulo> {
    let texto = valor?;
    let tipo = TipoArticulo::desde_texto(&texto);
    if tipo.is_none() {
        let mensaje = format!(
            "tipo desconocido «{texto}»: usa perfil, vidrio, herraje, accesorio, kit u otro"
        );
        informe.error(donde, mensaje);
    }
    tipo
}

/// Devuelve `Some(None)` cuando no aplica (no se vende por barras) y `None` si hay error.
fn leer_longitud(
    valor: Option<String>,
    unidad: UnidadDeVenta,
    donde: &str,
    informe: &mut Informe,
) -> Option<Option<Longitud>> {
    match (valor, unidad.es_barra()) {
        (None, true) => {
            informe.error(donde, "falta la longitud de barra (en cm, por ejemplo 640)");
            None
        }
        (None, false) => Some(None),
        (Some(_), false) => {
            informe.aviso(
                donde,
                "la longitud de barra solo se usa si la unidad es «barra»: se ignora",
            );
            Some(None)
        }
        (Some(texto), true) => leer_cm(&texto, donde, informe).map(Some),
    }
}

fn leer_cm(texto: &str, donde: &str, informe: &mut Informe) -> Option<Longitud> {
    let longitud = leer_decimal_es(texto).and_then(|cm| Longitud::desde_decimal(cm, Unidad::Cm));
    match longitud {
        Ok(longitud) if longitud > Longitud::CERO => Some(longitud),
        Ok(_) => {
            informe.error(donde, "la longitud de barra debe ser mayor que cero");
            None
        }
        Err(error) => {
            informe.error(donde, format!("longitud de barra: {error}"));
            None
        }
    }
}
