use super::lector::Lector;
use super::valor::Valor;

/// Sitios del dibujo donde puede ir el número de una pieza (`dibujo::marcadores`).
const POSICIONES: [&str; 8] = [
    "marco_lateral",
    "marco_superior",
    "marco_inferior",
    "hoja_lateral",
    "hoja_central",
    "hoja_superior",
    "hoja_inferior",
    "centro",
];

pub(super) fn leer_posicion(lector: &mut Lector, valor: &Valor, ruta: &str) -> Option<String> {
    let texto = lector.texto(valor, ruta)?;
    if POSICIONES.contains(&texto.as_str()) {
        return Some(texto.to_uppercase());
    }
    lector.informe.error(
        ruta,
        format!("posición no válida; usa una de: {}", POSICIONES.join(", ")),
    );
    None
}
