use super::lector::{Lector, leer_lista, opcional, ruta_hija};
use super::leer_serie::es_codigo_valido;
use super::{Dibujo, Limites, OpcionDeLinea, Rango, TipoDibujo, Valor, VarianteDePerfil};

pub fn leer_limites(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Option<Limites>> {
    let Some(valor) = valor else {
        return Some(None);
    };
    let campos = lector.mapa(valor, ruta)?;
    lector.solo_campos(campos, &["ancho", "alto"], ruta);
    let ancho = lector
        .obligatorio(campos, "ancho", ruta)
        .and_then(|v| leer_rango(lector, v, &ruta_hija(ruta, "ancho")));
    let alto = lector
        .obligatorio(campos, "alto", ruta)
        .and_then(|v| leer_rango(lector, v, &ruta_hija(ruta, "alto")));
    Some(Some(Limites {
        ancho: ancho?,
        alto: alto?,
    }))
}

fn leer_rango(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<Rango> {
    let campos = lector.mapa(valor, ruta)?;
    lector.solo_campos(campos, &["min", "max"], ruta);
    let minimo = lector
        .obligatorio(campos, "min", ruta)
        .and_then(|v| lector.dato_decimal(v, &ruta_hija(ruta, "min")));
    let maximo = lector
        .obligatorio(campos, "max", ruta)
        .and_then(|v| lector.dato_decimal(v, &ruta_hija(ruta, "max")));
    Some(Rango {
        minimo: minimo?,
        maximo: maximo?,
    })
}

pub fn leer_opciones(
    lector: &mut Lector,
    valor: Option<&Valor>,
    ruta: &str,
) -> Option<Vec<OpcionDeLinea>> {
    match valor {
        None => Some(Vec::new()),
        Some(valor) => leer_lista(lector, valor, ruta, leer_opcion),
    }
}

fn leer_opcion(lector: &mut Lector, elemento: &Valor, ruta: &str) -> Option<OpcionDeLinea> {
    let campos = lector.mapa(elemento, ruta)?;
    lector.solo_campos(
        campos,
        &["codigo", "nombre", "valores", "por_defecto"],
        ruta,
    );
    let r = |campo: &str| ruta_hija(ruta, campo);
    let codigo = lector
        .obligatorio(campos, "codigo", ruta)
        .and_then(|v| lector.texto(v, &r("codigo")));
    let nombre = lector
        .obligatorio(campos, "nombre", ruta)
        .and_then(|v| lector.texto(v, &r("nombre")));
    let valores = lector
        .obligatorio(campos, "valores", ruta)
        .and_then(|v| leer_lista(lector, v, &r("valores"), |l, v, r| l.texto(v, r)));
    let por_defecto = lector
        .obligatorio(campos, "por_defecto", ruta)
        .and_then(|v| lector.texto(v, &r("por_defecto")));
    let (valores, por_defecto) = (valores?, por_defecto?);
    if !valores.contains(&por_defecto) {
        lector.informe.error(
            &r("por_defecto"),
            format!("«{por_defecto}» no está entre los valores"),
        );
        return None;
    }
    Some(OpcionDeLinea {
        codigo: codigo?,
        nombre: nombre?,
        valores,
        por_defecto,
    })
}

pub fn leer_dibujo(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<Dibujo> {
    let campos = lector.mapa(valor, ruta)?;
    lector.solo_campos(
        campos,
        &["tipo", "hojas", "hoja_movil", "bisagras", "cremona"],
        ruta,
    );
    let r = |campo: &str| ruta_hija(ruta, campo);
    let tipo = lector
        .obligatorio(campos, "tipo", ruta)
        .and_then(|v| leer_tipo_de_dibujo(lector, v, &r("tipo")));
    let hojas = lector
        .obligatorio(campos, "hojas", ruta)
        .and_then(|v| lector.entero(v, &r("hojas"), 1..=8));
    let mut texto_opcional = |campo: &str| match opcional(campos, campo) {
        None => Some(None),
        Some(v) => lector.texto(v, &r(campo)).map(Some),
    };
    let (hoja_movil, bisagras, cremona) = (
        texto_opcional("hoja_movil"),
        texto_opcional("bisagras"),
        texto_opcional("cremona"),
    );
    Some(Dibujo {
        tipo: tipo?,
        hojas: u8::try_from(hojas?).ok()?,
        hoja_movil: hoja_movil?,
        bisagras: bisagras?,
        cremona: cremona?,
    })
}

fn leer_tipo_de_dibujo(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<TipoDibujo> {
    match lector.texto(valor, ruta)?.as_str() {
        "corredera" => Some(TipoDibujo::Corredera),
        "abatible" => Some(TipoDibujo::Abatible),
        "fija" => Some(TipoDibujo::Fija),
        _ => {
            lector
                .informe
                .error(ruta, "debe ser corredera, abatible o fija");
            None
        }
    }
}

/// `perfil_segun: { opcion: { valor: CODIGO } }`. Que la opción y el valor existan se comprueba
/// después, con la tipología entera (`validacion`).
pub fn leer_perfil_segun(
    lector: &mut Lector,
    valor: &Valor,
    ruta: &str,
) -> Option<Vec<VarianteDePerfil>> {
    let opciones = lector.mapa(valor, ruta)?;
    let mut variantes = Vec::new();
    for (opcion, por_valor) in opciones {
        let ruta_opcion = ruta_hija(ruta, opcion);
        for (valor, perfil) in lector.mapa(por_valor, &ruta_opcion)? {
            let ruta_valor = ruta_hija(&ruta_opcion, valor);
            let perfil = lector.texto(perfil, &ruta_valor)?;
            if !es_codigo_valido(&perfil) {
                lector
                    .informe
                    .error(&ruta_valor, "código no válido: letras, cifras y guion bajo");
                return None;
            }
            variantes.push(VarianteDePerfil {
                opcion: opcion.clone(),
                valor: valor.clone(),
                perfil,
            });
        }
    }
    Some(variantes)
}
