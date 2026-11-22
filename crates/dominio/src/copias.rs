//! Copias de seguridad automáticas (decisión del propietario, H7): cuándo toca hacer una y qué
//! copias se conservan. Todo en hora de Madrid, que es la que entiende el taller.

use std::collections::BTreeSet;

/// Hora de la copia diaria: a mediodía el taller suele estar parado y el equipo encendido.
pub const MINUTO_DE_LA_COPIA_DIARIA: u32 = 14 * 60;
/// Tras un intento fallido (sin disco, disco lleno…) se vuelve a probar pasado este tiempo.
pub const MINUTOS_ENTRE_REINTENTOS: i64 = 30;
/// Al cerrar no se repite una copia recién hecha (cerrar y abrir seguido).
pub const MINUTOS_SIN_REPETIR_AL_CERRAR: i64 = 10;
pub const DIARIAS_QUE_SE_CONSERVAN: usize = 7;
pub const SEMANALES_QUE_SE_CONSERVAN: usize = 4;

const PREFIJO: &str = "alumframe-";
const EXTENSION: &str = ".alumframe";

/// Un instante en hora local: días desde el 1-1-1970 y minuto del día.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MomentoLocal {
    pub dias: i64,
    pub minuto: u32,
}

impl MomentoLocal {
    pub fn nuevo((anio, mes, dia): (i32, u8, u8), (hora, minuto): (u8, u8)) -> MomentoLocal {
        MomentoLocal {
            dias: dias_desde_1970(anio, mes, dia),
            minuto: u32::from(hora) * 60 + u32::from(minuto),
        }
    }

    fn en_minutos(self) -> i64 {
        self.dias * 1440 + i64::from(self.minuto)
    }

    /// Semanas de lunes a domingo (el 1-1-1970 fue jueves).
    fn semana(self) -> i64 {
        (self.dias + 3).div_euclid(7)
    }
}

/// Algoritmo «days from civil» de Howard Hinnant, para el calendario gregoriano.
pub fn dias_desde_1970(anio: i32, mes: u8, dia: u8) -> i64 {
    let mes = i64::from(mes);
    let anio = i64::from(anio) - i64::from(mes <= 2);
    let era = anio.div_euclid(400);
    let del_era = anio - era * 400;
    let mes_desde_marzo = (mes + 9) % 12;
    let dia_del_anio = (153 * mes_desde_marzo + 2) / 5 + i64::from(dia) - 1;
    let dia_del_era = del_era * 365 + del_era / 4 - del_era / 100 + dia_del_anio;
    era * 146_097 + dia_del_era - 719_468
}

/// `alumframe-2026-11-22-1400.alumframe`: se ordena por nombre y se lee la fecha sin abrirla.
pub fn nombre_de_copia((anio, mes, dia): (i32, u8, u8), (hora, minuto): (u8, u8)) -> String {
    format!("{PREFIJO}{anio:04}-{mes:02}-{dia:02}-{hora:02}{minuto:02}{EXTENSION}")
}

/// El momento de una copia automática por su nombre; `None` si no es una de ellas.
pub fn leer_nombre_de_copia(nombre: &str) -> Option<MomentoLocal> {
    let centro = nombre.strip_prefix(PREFIJO)?.strip_suffix(EXTENSION)?;
    let partes: Vec<&str> = centro.split('-').collect();
    let [anio, mes, dia, horas] = partes.as_slice() else {
        return None;
    };
    let anio: i32 = cifras(anio, 4)?.parse().ok()?;
    let mes: u8 = cifras(mes, 2)?.parse().ok()?;
    let dia: u8 = cifras(dia, 2)?.parse().ok()?;
    let horas = cifras(horas, 4)?;
    let hora: u8 = horas.get(..2)?.parse().ok()?;
    let minuto: u8 = horas.get(2..)?.parse().ok()?;
    let valida = (1..=12).contains(&mes) && (1..=31).contains(&dia) && hora < 24 && minuto < 60;
    valida.then(|| MomentoLocal::nuevo((anio, mes, dia), (hora, minuto)))
}

fn cifras(texto: &str, largo: usize) -> Option<&str> {
    (texto.len() == largo && texto.bytes().all(|b| b.is_ascii_digit())).then_some(texto)
}

/// La copia de las 14:00 más reciente que ya debería existir.
fn ultima_programada(ahora: MomentoLocal) -> MomentoLocal {
    let dias = if ahora.minuto >= MINUTO_DE_LA_COPIA_DIARIA {
        ahora.dias
    } else {
        ahora.dias - 1
    };
    MomentoLocal {
        dias,
        minuto: MINUTO_DE_LA_COPIA_DIARIA,
    }
}

/// Toca si falta la copia de las 14:00 (también al arrancar tras un apagón), sin insistir
/// cada minuto cuando el último intento falló.
pub fn toca_copia(
    ultima_completa: Option<MomentoLocal>,
    ultimo_intento: Option<MomentoLocal>,
    ahora: MomentoLocal,
) -> bool {
    let falta = ultima_completa.is_none_or(|c| c < ultima_programada(ahora));
    let reciente = ultimo_intento
        .is_some_and(|i| ahora.en_minutos() - i.en_minutos() < MINUTOS_ENTRE_REINTENTOS);
    falta && !reciente
}

/// Al cerrar el programa se copia siempre, salvo si se acaba de hacer una.
pub fn toca_copia_al_cerrar(ultima_completa: Option<MomentoLocal>, ahora: MomentoLocal) -> bool {
    ultima_completa
        .is_none_or(|c| ahora.en_minutos() - c.en_minutos() >= MINUTOS_SIN_REPETIR_AL_CERRAR)
}

/// Las copias automáticas que sobran: se guarda la más reciente de cada uno de los 7 últimos
/// días con copia y, de lo anterior, la más reciente de cada una de 4 semanas. Lo que no es una
/// copia automática (por su nombre) no se toca nunca.
pub fn copias_que_sobran(nombres: &[String]) -> Vec<String> {
    let mut copias: Vec<(MomentoLocal, &String)> = nombres
        .iter()
        .filter_map(|n| leer_nombre_de_copia(n).map(|m| (m, n)))
        .collect();
    copias.sort_by(|a, b| b.cmp(a));
    let mut dias = BTreeSet::new();
    let mut semanas = BTreeSet::new();
    let mut sobran = Vec::new();
    for (momento, nombre) in copias {
        let diaria = dias.len() < DIARIAS_QUE_SE_CONSERVAN && dias.insert(momento.dias);
        let semanal = !dias.contains(&momento.dias)
            && semanas.len() < SEMANALES_QUE_SE_CONSERVAN
            && semanas.insert(momento.semana());
        if !diaria && !semanal {
            sobran.push(nombre.clone());
        }
    }
    sobran
}

#[cfg(test)]
#[path = "copias_pruebas.rs"]
mod pruebas;
