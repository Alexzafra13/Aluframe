use super::lector::{Campos, Lector, leer_lista, opcional, ruta_hija};
use super::leer_partes::{
    leer_dibujo, leer_limites, leer_opciones, leer_perfil_segun, leer_solo_si,
};
use super::leer_posicion::leer_posicion;
use super::leer_rellenos::{leer_chapas, leer_parteluces, leer_vidrio};
use super::leer_serie::{es_codigo_valido, leer_estado};
use super::{
    AccesorioDeSerie, AplicaA, Dato, Dibujo, EstadoSerie, ParteLucesDeSerie, PiezaDeChapa,
    PiezaDeSerie, Tipologia, UnidadCantidad, Valor, VarianteDePerfil, Vidrio,
};

const CAMPOS_DE_TIPOLOGIA: [&str; 10] = [
    "nombre",
    "estado",
    "limites",
    "opciones",
    "dibujo",
    "piezas",
    "vidrio",
    "accesorios",
    "chapas",
    "parteluces",
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
    // Las medidas de la línea (ADR-0030) son variables de las fórmulas de esta tipología.
    let variables_de_la_serie = lector.variables.clone();
    let medidas = opciones.iter().flatten().filter(|o| o.medida.is_some());
    lector.variables.extend(medidas.map(|o| o.codigo.clone()));
    let (dibujo, piezas, vidrio, accesorios, chapas, parteluces) =
        leer_cuerpo(lector, campos, &ruta);
    lector.variables = variables_de_la_serie;
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
        chapas: chapas?,
        parteluces: parteluces?,
    })
}

type Cuerpo = (
    Option<Dibujo>,
    Option<Vec<PiezaDeSerie>>,
    Option<Dato<Vidrio>>,
    Option<Vec<AccesorioDeSerie>>,
    Option<Vec<PiezaDeChapa>>,
    Option<Option<ParteLucesDeSerie>>,
);

/// Todo lo que se calcula: se lee entero aunque algo falle, para dar todos los errores juntos.
fn leer_cuerpo(lector: &mut Lector, campos: &Campos, ruta: &str) -> Cuerpo {
    let r = |campo: &str| ruta_hija(ruta, campo);
    let dibujo = lector
        .obligatorio(campos, "dibujo", ruta)
        .and_then(|v| leer_dibujo(lector, v, &r("dibujo")));
    let piezas = lector
        .obligatorio(campos, "piezas", ruta)
        .and_then(|v| leer_piezas(lector, v, &r("piezas")));
    // Sin `vidrio`, la tipología no lleva cristal (una mosquitera, ADR-0031).
    let vidrio = match opcional(campos, "vidrio") {
        Some(v) => leer_vidrio(lector, v, &r("vidrio")),
        None => Some(Dato::Valor(Vidrio {
            tipo: String::new(),
            piezas: Vec::new(),
        })),
    };
    let accesorios = lector
        .obligatorio(campos, "accesorios", ruta)
        .and_then(|v| leer_accesorios(lector, v, &r("accesorios")));
    let chapas = leer_chapas(lector, opcional(campos, "chapas"), &r("chapas"));
    let parteluces = leer_parteluces(lector, opcional(campos, "parteluces"), &r("parteluces"));
    (dibujo, piezas, vidrio, accesorios, chapas, parteluces)
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
    lector.solo_campos(
        campos,
        &[
            "perfil",
            "perfil_segun",
            "cantidad",
            "longitud",
            "corte",
            "posicion",
            "solo_si",
        ],
        ruta,
    );
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
    let (posicion, perfil_segun) = leer_extras_de_pieza(lector, campos, ruta)?;
    let solo_si = leer_solo_si(lector, opcional(campos, "solo_si"), &r("solo_si"))?;
    Some(PiezaDeSerie {
        solo_si,
        perfil: perfil?,
        perfil_segun,
        cantidad: cantidad?,
        longitud: longitud?,
        corte: corte?,
        posicion,
    })
}

type ExtrasDePieza = (Option<String>, Vec<VarianteDePerfil>);

/// Los campos opcionales de una pieza: `posicion` (ADR-0017) y `perfil_segun` (ADR-0018).
fn leer_extras_de_pieza(lector: &mut Lector, campos: &Campos, ruta: &str) -> Option<ExtrasDePieza> {
    let r = |campo: &str| ruta_hija(ruta, campo);
    let posicion = match opcional(campos, "posicion") {
        Some(v) => Some(leer_posicion(lector, v, &r("posicion"))?),
        None => None,
    };
    let perfil_segun = match opcional(campos, "perfil_segun") {
        Some(v) => leer_perfil_segun(lector, v, &r("perfil_segun"))?,
        None => Vec::new(),
    };
    Some((posicion, perfil_segun))
}

fn leer_corte(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<Dato<u16>> {
    if valor.es_confirmar() {
        lector.anotar_confirmar(ruta);
        return Some(Dato::Confirmar);
    }
    let grados = lector.entero(valor, ruta, 1..=180)?;
    u16::try_from(grados).ok().map(Dato::Valor)
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
        &[
            "articulo",
            "cantidad",
            "aplica_a",
            "unidad_cantidad",
            "solo_si",
        ],
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
    let solo_si = leer_solo_si(lector, opcional(campos, "solo_si"), &r("solo_si"))?;
    Some(AccesorioDeSerie {
        solo_si,
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
