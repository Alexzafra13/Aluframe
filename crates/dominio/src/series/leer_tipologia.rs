use super::lector::{Lector, leer_lista, opcional, ruta_hija};
use super::leer_partes::{leer_dibujo, leer_limites, leer_opciones};
use super::leer_serie::{es_codigo_valido, leer_estado};
use super::{
    AccesorioDeSerie, AplicaA, Dato, EstadoSerie, PiezaDeSerie, PiezaDeVidrio, Tipologia,
    UnidadCantidad, Valor, Vidrio,
};

const CAMPOS_DE_TIPOLOGIA: [&str; 8] = [
    "nombre",
    "estado",
    "limites",
    "opciones",
    "dibujo",
    "piezas",
    "vidrio",
    "accesorios",
];

pub fn leer_tipologias(lector: &mut Lector, valor: &Valor) -> Option<Vec<Tipologia>> {
    let entradas = lector.mapa(valor, "tipologias")?;
    if entradas.is_empty() {
        lector
            .informe
            .error("tipologias", "hace falta al menos una tipología");
        return None;
    }
    let mut tipologias = Vec::new();
    let mut todas_validas = true;
    for (codigo, valor) in entradas {
        match leer_tipologia(lector, codigo, valor) {
            Some(tipologia) => tipologias.push(tipologia),
            None => todas_validas = false,
        }
    }
    todas_validas.then_some(tipologias)
}

fn leer_tipologia(lector: &mut Lector, codigo: &str, valor: &Valor) -> Option<Tipologia> {
    let ruta = ruta_hija("tipologias", codigo);
    if !es_codigo_valido(codigo) {
        lector
            .informe
            .error(&ruta, "código no válido: letras, cifras y guion bajo");
    }
    let campos = lector.mapa(valor, &ruta)?;
    lector.solo_campos(campos, &CAMPOS_DE_TIPOLOGIA, &ruta);
    let r = |campo: &str| ruta_hija(&ruta, campo);
    let nombre = lector
        .obligatorio(campos, "nombre", &ruta)
        .and_then(|v| lector.texto(v, &r("nombre")));
    let estado = leer_estado_opcional(lector, opcional(campos, "estado"), &r("estado"));
    let limites = leer_limites(lector, opcional(campos, "limites"), &r("limites"));
    let opciones = leer_opciones(lector, opcional(campos, "opciones"), &r("opciones"));
    let dibujo = lector
        .obligatorio(campos, "dibujo", &ruta)
        .and_then(|v| leer_dibujo(lector, v, &r("dibujo")));
    let piezas = lector
        .obligatorio(campos, "piezas", &ruta)
        .and_then(|v| leer_piezas(lector, v, &r("piezas")));
    let vidrio = lector
        .obligatorio(campos, "vidrio", &ruta)
        .and_then(|v| leer_vidrio(lector, v, &r("vidrio")));
    let accesorios = lector
        .obligatorio(campos, "accesorios", &ruta)
        .and_then(|v| leer_accesorios(lector, v, &r("accesorios")));
    Some(Tipologia {
        codigo: codigo.to_owned(),
        nombre: nombre?,
        estado: estado?,
        limites: limites?,
        opciones: opciones?,
        dibujo: dibujo?,
        piezas: piezas?,
        vidrio: vidrio?,
        accesorios: accesorios?,
    })
}

fn leer_estado_opcional(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Option<EstadoSerie>> {
    match valor {
        Some(valor) => leer_estado(lector, valor, ruta).map(Some),
        None => Some(None),
    }
}

fn leer_piezas(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<Vec<PiezaDeSerie>> {
    leer_lista(lector, valor, ruta, leer_pieza)
}

fn leer_pieza(lector: &mut Lector, elemento: &Valor, ruta: &str) -> Option<PiezaDeSerie> {
    let campos = lector.mapa(elemento, ruta)?;
    lector.solo_campos(campos, &["perfil", "cantidad", "longitud", "corte"], ruta);
    let r = |campo: &str| ruta_hija(ruta, campo);
    let perfil = lector
        .obligatorio(campos, "perfil", ruta)
        .and_then(|v| lector.texto(v, &r("perfil")));
    let cantidad = lector
        .obligatorio(campos, "cantidad", ruta)
        .and_then(|v| lector.formula(v, &r("cantidad")));
    let longitud = lector
        .obligatorio(campos, "longitud", ruta)
        .and_then(|v| lector.formula(v, &r("longitud")));
    let corte = lector
        .obligatorio(campos, "corte", ruta)
        .and_then(|v| leer_corte(lector, v, &r("corte")));
    Some(PiezaDeSerie {
        perfil: perfil?,
        cantidad: cantidad?,
        longitud: longitud?,
        corte: corte?,
    })
}

fn leer_corte(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<Dato<u16>> {
    if valor.es_confirmar() {
        lector.anotar_confirmar(ruta);
        return Some(Dato::Confirmar);
    }
    let grados = lector.entero(valor, ruta, 1..=180)?;
    u16::try_from(grados).ok().map(Dato::Valor)
}

fn leer_vidrio(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<Dato<Vidrio>> {
    let campos = lector.mapa(valor, ruta)?;
    if opcional(campos, "estado").is_some_and(Valor::es_confirmar) {
        lector.solo_campos(campos, &["estado"], ruta);
        lector.anotar_confirmar(ruta);
        return Some(Dato::Confirmar);
    }
    lector.solo_campos(campos, &["tipo", "piezas"], ruta);
    let tipo = lector
        .obligatorio(campos, "tipo", ruta)
        .and_then(|v| lector.texto(v, &ruta_hija(ruta, "tipo")));
    let lista = lector.obligatorio(campos, "piezas", ruta)?;
    let piezas = leer_lista(
        lector,
        lista,
        &ruta_hija(ruta, "piezas"),
        leer_pieza_de_vidrio,
    );
    Some(Dato::Valor(Vidrio {
        tipo: tipo?,
        piezas: piezas?,
    }))
}

fn leer_pieza_de_vidrio(
    lector: &mut Lector,
    elemento: &Valor,
    ruta: &str,
) -> Option<PiezaDeVidrio> {
    let campos = lector.mapa(elemento, ruta)?;
    lector.solo_campos(campos, &["cantidad", "ancho", "alto"], ruta);
    let mut leer = |campo: &str| {
        let valor = lector.obligatorio(campos, campo, ruta)?;
        let ruta_campo = ruta_hija(ruta, campo);
        match lector.formula(valor, &ruta_campo)? {
            Dato::Valor(formula) => Some(formula),
            Dato::Confirmar => {
                lector.informe.error(&ruta_campo, "en un vidrio definido no se admite CONFIRMAR; usa vidrio: { estado: CONFIRMAR }");
                None
            }
        }
    };
    let (cantidad, ancho, alto) = (leer("cantidad"), leer("ancho"), leer("alto"));
    Some(PiezaDeVidrio {
        cantidad: cantidad?,
        ancho: ancho?,
        alto: alto?,
    })
}

fn leer_accesorios(
    lector: &mut Lector,
    valor: &Valor,
    ruta: &str,
) -> Option<Vec<AccesorioDeSerie>> {
    leer_lista(lector, valor, ruta, leer_accesorio)
}

fn leer_accesorio(lector: &mut Lector, elemento: &Valor, ruta: &str) -> Option<AccesorioDeSerie> {
    let campos = lector.mapa(elemento, ruta)?;
    lector.solo_campos(
        campos,
        &["articulo", "cantidad", "aplica_a", "unidad_cantidad"],
        ruta,
    );
    let r = |campo: &str| ruta_hija(ruta, campo);
    let articulo = lector
        .obligatorio(campos, "articulo", ruta)
        .and_then(|v| lector.texto(v, &r("articulo")));
    let cantidad = lector
        .obligatorio(campos, "cantidad", ruta)
        .and_then(|v| lector.formula(v, &r("cantidad")));
    let aplica_a = leer_aplica_a(lector, opcional(campos, "aplica_a"), &r("aplica_a"));
    let unidad = leer_unidad_cantidad(
        lector,
        opcional(campos, "unidad_cantidad"),
        &r("unidad_cantidad"),
    );
    Some(AccesorioDeSerie {
        articulo: articulo?,
        cantidad: cantidad?,
        aplica_a: aplica_a?,
        unidad_cantidad: unidad?,
    })
}

fn leer_aplica_a(lector: &mut Lector, valor: Option<&Valor>, ruta: &str) -> Option<AplicaA> {
    let Some(valor) = valor else {
        return Some(AplicaA::Ventana);
    };
    match lector.texto(valor, ruta)?.as_str() {
        "marco" => Some(AplicaA::Marco),
        "hojas" => Some(AplicaA::Hojas),
        "ventana" => Some(AplicaA::Ventana),
        _ => {
            lector
                .informe
                .error(ruta, "debe ser marco, hojas o ventana");
            None
        }
    }
}

fn leer_unidad_cantidad(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<UnidadCantidad> {
    let Some(valor) = valor else {
        return Some(UnidadCantidad::Unidad);
    };
    match lector.texto(valor, ruta)?.as_str() {
        "unidad" => Some(UnidadCantidad::Unidad),
        "metro" => Some(UnidadCantidad::Metro),
        _ => {
            lector.informe.error(ruta, "debe ser unidad o metro");
            None
        }
    }
}
