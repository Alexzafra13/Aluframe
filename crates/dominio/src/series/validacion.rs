use std::collections::BTreeSet;

use super::lector::ruta_hija;
use super::{Dato, Informe, Serie, TipoDibujo, Tipologia, calcular_ventana};
use crate::unidades::Decimal;

/// Comprobaciones entre partes del archivo (FORMATO_SERIES.md, apartado 11).
pub fn validar_serie(serie: &Serie, informe: &mut Informe) {
    let mut usadas = BTreeSet::new();
    for tipologia in &serie.tipologias {
        let ruta = ruta_hija("tipologias", &tipologia.codigo);
        for codigo in codigos_usados(tipologia) {
            if !serie.referencias.contains_key(&codigo) {
                informe.error(&ruta, format!("«{codigo}» no está en referencias"));
            }
            usadas.insert(codigo);
        }
        validar_dibujo(tipologia, &ruta, informe);
        validar_opciones(tipologia, &ruta, informe);
        validar_variantes(tipologia, &ruta, informe);
        if tipologia.limites.is_none() {
            informe.aviso(
                &ruta,
                "no tiene límites de medida; no se avisará de medidas fuera de rango",
            );
        }
        let cien = Decimal::desde_entero(100);
        if let Err(error) = calcular_ventana(serie, &tipologia.codigo, cien, cien) {
            informe.error(&ruta, format!("con ancho = 100 y alto = 100: {error}"));
        }
    }
    for codigo in serie.referencias.keys().filter(|c| !usadas.contains(*c)) {
        informe.aviso(
            &ruta_hija("referencias", codigo),
            "no se usa en ninguna tipología",
        );
    }
}

fn codigos_usados(tipologia: &Tipologia) -> Vec<String> {
    let mut codigos: Vec<String> = tipologia.piezas.iter().map(|p| p.perfil.clone()).collect();
    let variantes = tipologia.piezas.iter().flat_map(|p| &p.perfil_segun);
    codigos.extend(variantes.map(|v| v.perfil.clone()));
    codigos.extend(tipologia.accesorios.iter().map(|a| a.articulo.clone()));
    if let Dato::Valor(vidrio) = &tipologia.vidrio {
        codigos.push(vidrio.tipo.clone());
    }
    codigos.sort();
    codigos.dedup();
    codigos
}

fn validar_dibujo(tipologia: &Tipologia, ruta: &str, informe: &mut Informe) {
    let ruta = ruta_hija(ruta, "dibujo");
    let dibujo = &tipologia.dibujo;
    if dibujo.tipo == TipoDibujo::Corredera && ![2, 4].contains(&dibujo.hojas) {
        informe.error(&ruta, "una corredera se dibuja con 2 o 4 hojas");
    }
    if let Some(lado) = &dibujo.hoja_movil
        && !["izquierda", "derecha"].contains(&lado.as_str())
    {
        informe.error(
            &ruta_hija(&ruta, "hoja_movil"),
            "debe ser izquierda o derecha",
        );
    }
    if dibujo.tipo != TipoDibujo::Abatible {
        return;
    }
    match &dibujo.bisagras {
        Some(codigo) if tipologia.accesorios.iter().any(|a| &a.articulo == codigo) => {}
        Some(codigo) => informe.error(
            &ruta,
            format!("las bisagras «{codigo}» no están entre los accesorios"),
        ),
        None => informe.error(
            &ruta,
            "una abatible necesita «bisagras» (código del accesorio)",
        ),
    }
    match &dibujo.cremona {
        Some(codigo) if tipologia.opciones.iter().any(|o| &o.codigo == codigo) => {}
        Some(codigo) => informe.error(
            &ruta,
            format!("la cremona «{codigo}» no es una opción de línea"),
        ),
        None => informe.error(
            &ruta,
            "una abatible necesita «cremona» (código de la opción)",
        ),
    }
}

fn validar_variantes(tipologia: &Tipologia, ruta: &str, informe: &mut Informe) {
    let ruta = ruta_hija(ruta, "piezas");
    for variante in tipologia.piezas.iter().flat_map(|p| &p.perfil_segun) {
        match tipologia
            .opciones
            .iter()
            .find(|o| o.codigo == variante.opcion)
        {
            None => informe.error(
                &ruta,
                format!("«{}» no es una opción de esta tipología", variante.opcion),
            ),
            Some(opcion) if !opcion.valores.contains(&variante.valor) => informe.error(
                &ruta,
                format!("«{}» no es un valor de «{}»", variante.valor, opcion.codigo),
            ),
            Some(_) => {}
        }
    }
}

fn validar_opciones(tipologia: &Tipologia, ruta: &str, informe: &mut Informe) {
    let mut vistas = BTreeSet::new();
    for opcion in &tipologia.opciones {
        if !vistas.insert(&opcion.codigo) {
            informe.error(
                &ruta_hija(ruta, "opciones"),
                format!("«{}» está repetida", opcion.codigo),
            );
        }
    }
}

pub fn referencias_sin_confirmar(serie: &Serie, tipologia: &Tipologia) -> Option<String> {
    let faltan: Vec<String> = codigos_usados(tipologia)
        .into_iter()
        .filter(|codigo| {
            serie
                .referencias
                .get(codigo)
                .is_none_or(|r| r.referencia.es_confirmar())
        })
        .collect();
    (!faltan.is_empty()).then(|| {
        format!(
            "Falta la referencia del proveedor (y el precio) de: {}.",
            faltan.join(", ")
        )
    })
}

pub fn comprobar_limites(tipologia: &Tipologia, ancho: Decimal, alto: Decimal) -> Vec<String> {
    let Some(limites) = &tipologia.limites else {
        return Vec::new();
    };
    let mut avisos = Vec::new();
    for (nombre, medida, rango) in [
        ("ancho", ancho, &limites.ancho),
        ("alto", alto, &limites.alto),
    ] {
        if let Some(minimo) = rango.minimo.valor().filter(|m| medida < **m) {
            avisos.push(format!(
                "El {nombre} es menor que el mínimo de la serie ({minimo})."
            ));
        }
        if let Some(maximo) = rango.maximo.valor().filter(|m| medida > **m) {
            avisos.push(format!(
                "El {nombre} supera el máximo de la serie ({maximo})."
            ));
        }
    }
    avisos
}
