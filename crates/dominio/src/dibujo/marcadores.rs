use super::geometria::{Plano, Rect};

/// Dónde va el número de cada pieza. Se deduce del nombre del perfil (MARCO_LATERAL,
/// HOJA_RODAMIENTO…) porque las series no declaran posiciones; lo que no se reconoce se
/// coloca en una columna a la derecha, así que ningún número se pierde.
pub fn colocar(plano: &Plano, piezas: &[(u32, String)]) -> Vec<(u32, i64, i64)> {
    let mut colocados: Vec<(u32, i64, i64)> = Vec::with_capacity(piezas.len());
    let radio = radio(plano);
    let mut sueltos = 0;
    for (numero, perfil) in piezas {
        let (x, mut y) = posicion(plano, &perfil.to_uppercase()).unwrap_or_else(|| {
            sueltos += 1;
            (
                plano.marco.derecha() + plano.margen / 2,
                radio * 3 * sueltos,
            )
        });
        while colocados
            .iter()
            .any(|(_, cx, cy)| (cx - x).abs() < 2 * radio && (cy - y).abs() < 2 * radio)
        {
            y += 2 * radio + radio / 2;
        }
        colocados.push((*numero, x, y));
    }
    colocados
}

pub fn radio(plano: &Plano) -> i64 {
    (plano.marco.ancho.max(plano.marco.alto) / 40).max(1)
}

fn posicion(plano: &Plano, perfil: &str) -> Option<(i64, i64)> {
    let marco = plano.marco;
    let medio_marco = plano.grosor_marco / 2;
    if perfil.contains("MARCO") {
        return Some(match () {
            _ if perfil.contains("SUPERIOR") => {
                (marco.x + marco.ancho * 35 / 100, marco.y + medio_marco)
            }
            _ if perfil.contains("INFERIOR") => (
                marco.x + marco.ancho * 70 / 100,
                marco.abajo() - medio_marco,
            ),
            _ => (marco.x + medio_marco, marco.y + marco.alto * 72 / 100),
        });
    }
    if perfil == "CENTRO" {
        // Donde se juntan las hojas centrales (el cierre 4 hojas de una corredera).
        let hoja = *plano.hojas.first()?;
        return Some((marco.x + marco.ancho / 2, hoja.y + hoja.alto * 78 / 100));
    }
    let hoja = *plano.hojas.first()?;
    posicion_en_hoja(hoja, plano.grosor_hoja / 2, perfil)
}

fn posicion_en_hoja(hoja: Rect, medio: i64, perfil: &str) -> Option<(i64, i64)> {
    let es = |palabras: &[&str]| palabras.iter().any(|p| perfil.contains(p));
    if es(&["CENTRAL", "INVERSOR", "ENCUENTRO"]) {
        Some((hoja.derecha() - medio, hoja.y + hoja.alto * 72 / 100))
    } else if es(&["RODAMIENTO", "HORIZONTAL", "INFERIOR"]) {
        Some((hoja.x + hoja.ancho / 2, hoja.abajo() - medio))
    } else if es(&["SUPERIOR"]) {
        Some((hoja.x + hoja.ancho / 2, hoja.y + medio))
    } else if es(&["HOJA", "CIERRE", "VERTICAL", "LATERAL"]) {
        Some((hoja.x + medio, hoja.y + hoja.alto * 40 / 100))
    } else {
        None
    }
}
