mod calculo;
mod importacion;
mod parteluces;
mod siluetas;
mod variantes;

use super::*;
use crate::unidades::Decimal;

pub(super) fn t(texto: &str) -> Valor {
    Valor::Texto(texto.into())
}

pub(super) fn n(texto: &str) -> Valor {
    Valor::Numero(texto.into())
}

pub(super) fn m(campos: Vec<(&str, Valor)>) -> Valor {
    Valor::Mapa(campos.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}

fn pieza(perfil: &str, cantidad: Valor, longitud: &str, corte: Valor) -> Valor {
    m(vec![
        ("perfil", t(perfil)),
        ("cantidad", cantidad),
        ("longitud", t(longitud)),
        ("corte", corte),
    ])
}

fn corredera() -> Valor {
    m(vec![
        ("nombre", t("Corredera")),
        ("limites", limites_corredera()),
        (
            "dibujo",
            m(vec![
                ("tipo", t("corredera")),
                ("hojas", n("2")),
                ("hoja_movil", t("derecha")),
            ]),
        ),
        (
            "piezas",
            Valor::Lista(vec![
                pieza("MARCO", n("2"), "alto", n("90")),
                pieza("HOJA", n("4"), "ancho / 2 - descuento", n("90")),
            ]),
        ),
        ("vidrio", vidrio_corredera()),
        ("accesorios", accesorios_corredera()),
    ])
}

fn limites_corredera() -> Valor {
    m(vec![
        ("ancho", m(vec![("min", n("40")), ("max", n("300"))])),
        ("alto", m(vec![("min", n("40")), ("max", t(CONFIRMAR))])),
    ])
}

fn vidrio_corredera() -> Valor {
    let pieza = m(vec![
        ("cantidad", n("2")),
        ("ancho", t("ancho / 2 - 7")),
        ("alto", t("alto - 9")),
    ]);
    m(vec![
        ("tipo", t("CRISTAL")),
        ("piezas", Valor::Lista(vec![pieza])),
    ])
}

fn accesorios_corredera() -> Valor {
    Valor::Lista(vec![
        m(vec![
            ("articulo", t("TORNILLO")),
            ("cantidad", n("9")),
            ("aplica_a", t("marco")),
        ]),
        m(vec![
            ("articulo", t("TORNILLO")),
            ("cantidad", n("8")),
            ("aplica_a", t("hojas")),
        ]),
        m(vec![
            ("articulo", t("JUNTA")),
            ("cantidad", t("(ancho + alto) * 2 / 100")),
            ("unidad_cantidad", t("metro")),
        ]),
    ])
}

fn abatible() -> Valor {
    m(vec![
        ("nombre", t("Abatible")),
        (
            "opciones",
            Valor::Lista(vec![m(vec![
                ("codigo", t("lado_cremona")),
                ("nombre", t("Lado")),
                ("valores", Valor::Lista(vec![t("izquierda"), t("derecha")])),
                ("por_defecto", t("derecha")),
            ])]),
        ),
        (
            "dibujo",
            m(vec![
                ("tipo", t("abatible")),
                ("hojas", n("2")),
                ("bisagras", t("BISAGRA")),
                ("cremona", t("lado_cremona")),
            ]),
        ),
        (
            "piezas",
            Valor::Lista(vec![
                pieza("MARCO", n("2"), "alto", n("45")),
                pieza("INVERSOR", t(CONFIRMAR), "alto - 10.3", t(CONFIRMAR)),
                pieza("HOJA", n("4"), "alto - 4.1", t(CONFIRMAR)),
            ]),
        ),
        ("vidrio", m(vec![("estado", t(CONFIRMAR))])),
        (
            "accesorios",
            Valor::Lista(vec![
                m(vec![
                    ("articulo", t("BISAGRA")),
                    ("cantidad", t("2 * si(alto > 140, 3, 2)")),
                ]),
                m(vec![("articulo", t("PLETINA")), ("cantidad", t(CONFIRMAR))]),
            ]),
        ),
    ])
}

fn referencia(texto: &str) -> Valor {
    m(vec![("referencia", t(texto))])
}

pub(super) fn serie_base() -> Valor {
    m(vec![
        ("serie", t("PRUEBA_1")),
        ("fabricante", t("FABRICA")),
        ("proveedor", t("PROVEEDOR")),
        ("estado", t("validada")),
        ("unidades", t("cm")),
        (
            "redondeo",
            m(vec![("precision", n("0.1")), ("sentido", t("abajo"))]),
        ),
        ("longitud_barra", n("640")),
        ("parametros", m(vec![("descuento", n("2"))])),
        (
            "referencias",
            m(vec![
                ("MARCO", referencia("R-1")),
                ("HOJA", referencia("R-2")),
                ("CRISTAL", referencia("V-4")),
                ("TORNILLO", referencia("T-1")),
                ("JUNTA", referencia("J-1")),
                ("INVERSOR", referencia(CONFIRMAR)),
                ("BISAGRA", referencia("B-1")),
                ("PLETINA", referencia("P-1")),
            ]),
        ),
        (
            "tipologias",
            m(vec![("corredera", corredera()), ("abatible", abatible())]),
        ),
    ])
}

pub(super) fn cambiar(valor: &mut Valor, ruta: &[&str], nuevo: Option<Valor>) {
    let Valor::Mapa(campos) = valor else {
        panic!("no es un mapa")
    };
    let (clave, resto) = ruta.split_first().unwrap();
    if resto.is_empty() {
        campos.retain(|(k, _)| k != clave);
        if let Some(nuevo) = nuevo {
            campos.push(((*clave).to_owned(), nuevo));
        }
        return;
    }
    let hijo = campos
        .iter_mut()
        .find(|(k, _)| k == clave)
        .map(|(_, v)| v)
        .unwrap();
    cambiar(hijo, resto, nuevo);
}

pub(super) fn errores_con(ruta: &[&str], nuevo: Option<Valor>) -> Vec<String> {
    let mut valor = serie_base();
    cambiar(&mut valor, ruta, nuevo);
    importar_serie(&valor)
        .unwrap_err()
        .errores
        .iter()
        .map(ToString::to_string)
        .collect()
}

pub(super) fn d(texto: &str) -> Decimal {
    Decimal::desde_texto(texto).unwrap()
}
