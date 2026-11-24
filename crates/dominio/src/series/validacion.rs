use std::collections::{BTreeMap, BTreeSet};

use super::lector::ruta_hija;
use super::{
    Dato, Informe, Serie, SoloSi, TipoDibujo, Tipologia, calcular_ventana,
    calcular_ventana_con_opciones,
};
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
        validar_cada_opcion(serie, tipologia, &ruta, informe);
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
    codigos.extend(tipologia.chapas.iter().map(|c| c.articulo.clone()));
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
        && !["izquierda", "derecha", "ambas"].contains(&lado.as_str())
    {
        informe.error(
            &ruta_hija(&ruta, "hoja_movil"),
            "debe ser izquierda, derecha o ambas",
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

/// Cada `perfil_segun` y `solo_si` nombra una opción de lista de la tipología y uno de sus valores.
fn validar_variantes(tipologia: &Tipologia, ruta: &str, informe: &mut Informe) {
    for (opcion, valor) in condiciones(tipologia) {
        match tipologia.opciones.iter().find(|o| o.codigo == opcion) {
            None => informe.error(
                ruta,
                format!("«{opcion}» no es una opción de esta tipología"),
            ),
            Some(o) if o.medida.is_some() => informe.error(
                ruta,
                format!("«{opcion}» es una medida: no sirve de condición"),
            ),
            Some(o) if !o.valores.contains(&valor) => {
                informe.error(ruta, format!("«{valor}» no es un valor de «{opcion}»"));
            }
            Some(_) => {}
        }
    }
}

fn condiciones(t: &Tipologia) -> Vec<(String, String)> {
    let variantes = t.piezas.iter().flat_map(|p| &p.perfil_segun);
    let mut todas: Vec<(String, String)> = variantes
        .map(|v| (v.opcion.clone(), v.valor.clone()))
        .collect();
    let mut anadir = |solo_si: &SoloSi| todas.extend(solo_si.iter().cloned());
    t.piezas.iter().for_each(|p| anadir(&p.solo_si));
    t.accesorios.iter().for_each(|a| anadir(&a.solo_si));
    t.chapas.iter().for_each(|c| anadir(&c.solo_si));
    t.opciones.iter().for_each(|o| anadir(&o.solo_si));
    t.parteluces.iter().for_each(|p| anadir(&p.solo_si));
    if let Dato::Valor(vidrio) = &t.vidrio {
        vidrio.piezas.iter().for_each(|p| anadir(&p.solo_si));
    }
    todas
}

/// Combinaciones de opciones que se prueban al importar, como mucho. Una serie real tiene pocas
/// (Aluprom 21: 2 × 2 × 2 = 8); por encima, solo se prueba cada valor por separado.
const MAXIMO_DE_COMBINACIONES: usize = 256;

/// Además de los valores por defecto, se calcula con cada combinación de opciones de lista, para
/// que una fórmula que solo se usa con parteluces y Climalit no falle al presupuestar.
fn validar_cada_opcion(serie: &Serie, tipologia: &Tipologia, ruta: &str, informe: &mut Informe) {
    let cien = Decimal::desde_entero(100);
    for elegidas in combinaciones(tipologia) {
        let calculo =
            calcular_ventana_con_opciones(serie, &tipologia.codigo, (cien, cien), &elegidas);
        if let Err(error) = calculo {
            let caso: Vec<String> = elegidas.iter().map(|(k, v)| format!("{k} = {v}")).collect();
            let caso = format!("con {}, ancho = 100 y alto = 100", caso.join(", "));
            informe.error(ruta, format!("{caso}: {error}"));
            return;
        }
    }
}

fn combinaciones(tipologia: &Tipologia) -> Vec<BTreeMap<String, String>> {
    let listas: Vec<_> = tipologia
        .opciones
        .iter()
        .filter(|o| o.medida.is_none())
        .collect();
    let total = listas
        .iter()
        .try_fold(1_usize, |n, o| n.checked_mul(o.valores.len().max(1)));
    if total.is_none_or(|n| n > MAXIMO_DE_COMBINACIONES) {
        let sueltas = listas.iter().flat_map(|o| {
            o.valores.iter().map(|v| {
                let mut elegidas: BTreeMap<String, String> = o.solo_si.iter().cloned().collect();
                elegidas.insert(o.codigo.clone(), v.clone());
                elegidas
            })
        });
        return sueltas.collect();
    }
    let mut todas = vec![BTreeMap::new()];
    for opcion in listas {
        todas = todas
            .into_iter()
            .flat_map(|base| {
                opcion.valores.iter().map(move |valor| {
                    let mut elegidas = base.clone();
                    elegidas.insert(opcion.codigo.clone(), valor.clone());
                    elegidas
                })
            })
            .collect();
    }
    todas
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
                "El {nombre} ({medida}) es menor que el mínimo que admite la serie ({minimo})."
            ));
        }
        if let Some(maximo) = rango.maximo.valor().filter(|m| medida > **m) {
            avisos.push(format!(
                "El {nombre} ({medida}) supera el máximo que admite la serie ({maximo})."
            ));
        }
    }
    avisos
}
