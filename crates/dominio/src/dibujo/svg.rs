use std::fmt::Write;

use super::acabado::{definiciones, grupo_de_herrajes, perfil};
use super::geometria::{Lado, Plano, Rect};
use super::marcadores::{colocar, radio};
use super::persiana;
use super::{ColorDeDibujo, DatosDeDibujo};
use crate::series::TipoDibujo;

/// Los colores fijos van en la hoja de estilos (clases); los del acabado, en atributos del
/// propio dibujo (`acabado.rs`). Así sirven en pantalla y al imprimir.
pub fn escribir(plano: &Plano, datos: &DatosDeDibujo) -> String {
    let m = plano.margen;
    let (ancho, alto) = (plano.contorno.ancho, plano.contorno.alto);
    let mut s = String::with_capacity(4_096);
    let _ = write!(
        s,
        r#"<svg class="dibujo-ventana" viewBox="{} {} {} {}" role="img" aria-labelledby="titulo-dibujo" xmlns="http://www.w3.org/2000/svg"><title id="titulo-dibujo">{}</title>{}"#,
        -m,
        -m / 2,
        ancho + 2 * m,
        alto + m + m / 2,
        escapar(&datos.titulo),
        definiciones(&datos.color)
    );
    if plano.tipo == TipoDibujo::Mosquitera {
        super::mosquitera::dibujar(&mut s, plano, datos);
    } else {
        cuerpo_de_ventana(&mut s, plano, datos);
    }
    super::cotas::escribir(&mut s, plano, datos);
    super::parteluces::cota(&mut s, plano, datos);
    numeros_de_pieza(&mut s, plano, datos);
    s.push_str("</svg>");
    s
}

/// Persiana, marco, hojas, parteluces y herrajes de una ventana.
fn cuerpo_de_ventana(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    persiana::dibujar_cajon_y_guias(s, plano, datos);
    perfil(s, &datos.color, ("marco", plano.marco), plano.grosor_marco);
    for (indice, rectangulo) in plano.hojas.iter().enumerate() {
        dibujar_hoja(s, plano, (*rectangulo, indice), &datos.color);
    }
    persiana::dibujar_lamas(s, plano);
    persiana::dibujar_cinta(s, plano, datos);
    super::parteluces::dibujar(s, plano, datos);
    s.push_str(&grupo_de_herrajes(&datos.color));
    match plano.tipo {
        TipoDibujo::Corredera => flechas_de_corredera(s, plano, datos),
        TipoDibujo::Abatible => herrajes_de_abatible(s, plano, datos),
        TipoDibujo::Fija | TipoDibujo::Mosquitera => {}
    }
    s.push_str("</g>");
}

fn rect(s: &mut String, clase: &str, r: Rect) {
    let _ = write!(
        s,
        r#"<rect class="{clase}" x="{}" y="{}" width="{}" height="{}"/>"#,
        r.x, r.y, r.ancho, r.alto
    );
}

fn dibujar_hoja(
    s: &mut String,
    plano: &Plano,
    (hoja, indice): (Rect, usize),
    color: &ColorDeDibujo,
) {
    if plano.tipo != TipoDibujo::Fija {
        perfil(s, color, ("hoja", hoja), plano.grosor_hoja);
    }
    let cristal = if plano.tipo == TipoDibujo::Fija {
        hoja
    } else {
        hoja.dentro(plano.grosor_hoja)
    };
    let _ = write!(s, r#"<g class="capa-cristal-{indice}">"#);
    rect(s, "cristal", cristal);
    let lado = cristal.ancho.min(cristal.alto) * 45 / 100;
    let _ = write!(
        s,
        r#"<path class="reflejo" d="M{} {} h{lado} L{} {} Z"/></g>"#,
        cristal.x,
        cristal.y,
        cristal.x,
        cristal.y + lado
    );
}

fn flecha(s: &mut String, hoja: Rect, hacia_la_derecha: bool) {
    let y = hoja.y + hoja.alto / 2;
    let largo = hoja.ancho * 35 / 100;
    let punta = (largo / 6).max(1);
    let (desde, hasta, signo) = if hacia_la_derecha {
        (
            hoja.x + hoja.ancho * 33 / 100,
            hoja.x + hoja.ancho * 33 / 100 + largo,
            -1,
        )
    } else {
        (
            hoja.x + hoja.ancho * 67 / 100,
            hoja.x + hoja.ancho * 67 / 100 - largo,
            1,
        )
    };
    let _ = write!(
        s,
        r#"<path class="flecha" d="M{desde} {y} H{hasta} M{} {} L{hasta} {y} L{} {}"/>"#,
        hasta + signo * punta,
        y - punta,
        hasta + signo * punta,
        y + punta
    );
}

/// En 2 hojas se abren la indicada o `ambas` (por defecto la derecha): flecha hacia el centro y
/// cierre en su lateral, contra el marco, como en el catálogo de Aluval. En 4 se abren las cuatro
/// (propietario, 2026-11-23): las de los lados hacia el centro y las centrales hacia fuera, con un
/// cierre en cada lateral y dos en el centro, uno en cada hoja central.
fn flechas_de_corredera(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let hojas = &plano.hojas;
    if let [primera, segunda, tercera, cuarta] = hojas.as_slice() {
        for (hoja, hacia_la_derecha) in [
            (primera, true),
            (segunda, false),
            (tercera, true),
            (cuarta, false),
        ] {
            flecha(s, *hoja, hacia_la_derecha);
        }
        cierre_en_el_lado(s, plano, *primera, false);
        cierres_del_centro(s, plano, segunda.centro_y());
        cierre_en_el_lado(s, plano, *cuarta, true);
        return;
    }
    let movil = datos.dibujo.hoja_movil.as_deref().unwrap_or("derecha");
    if let Some(izquierda) = hojas.first().filter(|_| movil != "derecha") {
        flecha(s, *izquierda, true);
        cierre_en_el_lado(s, plano, *izquierda, false);
    }
    if let Some(derecha) = hojas.last().filter(|_| movil != "izquierda") {
        flecha(s, *derecha, false);
        cierre_en_el_lado(s, plano, *derecha, true);
    }
}

/// El cierre va en el canto de la hoja contrario al sentido en que se abre.
fn cierre_en_el_lado(s: &mut String, plano: &Plano, hoja: Rect, derecho: bool) {
    let cuarto = plano.grosor_hoja / 4;
    let x = if derecho {
        hoja.derecha() - cuarto - (plano.grosor_hoja / 2).max(1)
    } else {
        hoja.x + cuarto
    };
    manilla(s, x, hoja.centro_y(), plano.grosor_hoja);
}

/// Las centrales se dibujan solapadas, pero en el taller se juntan en el centro con el cierre
/// 4 hojas: un cierre a cada lado de la junta.
fn cierres_del_centro(s: &mut String, plano: &Plano, centro_y: i64) {
    let centro = plano.marco.x + plano.marco.ancho / 2;
    let hueco = (plano.grosor_hoja / 4).max(1);
    let ancho_manilla = (plano.grosor_hoja / 2).max(1);
    manilla(
        s,
        centro - hueco - ancho_manilla,
        centro_y,
        plano.grosor_hoja,
    );
    manilla(s, centro + hueco, centro_y, plano.grosor_hoja);
}

fn manilla(s: &mut String, x: i64, centro_y: i64, grosor: i64) {
    let alto = grosor * 3;
    let _ = write!(
        s,
        r#"<rect class="manilla" x="{x}" y="{}" width="{}" height="{alto}" rx="{}"/>"#,
        centro_y - alto / 2,
        (grosor / 2).max(1),
        (grosor / 4).max(1)
    );
}

/// Cada hoja gira hacia su bisagra: la mitad izquierda de las hojas a la izquierda y el resto a
/// la derecha. Sin opción elegida, la cremona va en la última hoja.
fn herrajes_de_abatible(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let total = plano.hojas.len();
    for (indice, hoja) in plano.hojas.iter().enumerate() {
        let lado = if total == 1 || indice < total / 2 {
            Lado::Izquierda
        } else {
            Lado::Derecha
        };
        let (bisagra_x, cierre_x) = match lado {
            Lado::Izquierda => (hoja.x, hoja.derecha()),
            Lado::Derecha => (hoja.derecha(), hoja.x),
        };
        let _ = write!(
            s,
            r#"<path class="apertura" d="M{cierre_x} {} L{bisagra_x} {} L{cierre_x} {}"/>"#,
            hoja.y + plano.grosor_hoja,
            hoja.centro_y(),
            hoja.abajo() - plano.grosor_hoja
        );
        bisagras(
            s,
            *hoja,
            bisagra_x,
            (datos.bisagras_por_hoja, plano.grosor_hoja),
        );
        let lleva_cremona = match datos.hoja_cremona {
            Some(elegida) => elegida == lado,
            None => indice + 1 == total,
        };
        if lleva_cremona {
            let x = match lado {
                Lado::Izquierda => cierre_x - plano.grosor_hoja,
                Lado::Derecha => cierre_x + plano.grosor_hoja / 2,
            };
            manilla(s, x, hoja.centro_y(), plano.grosor_hoja);
        }
    }
}

fn bisagras(s: &mut String, hoja: Rect, x: i64, (cuantas, grosor): (u32, i64)) {
    let n = i64::from(cuantas);
    for i in 0..n {
        let fraccion_mil = if n == 1 { 500 } else { 150 + 700 * i / (n - 1) };
        let alto = grosor * 2;
        let _ = write!(
            s,
            r#"<rect class="bisagra" x="{}" y="{}" width="{}" height="{alto}"/>"#,
            x - grosor / 4,
            hoja.y + hoja.alto * fraccion_mil / 1000 - alto / 2,
            (grosor / 2).max(1)
        );
    }
}

fn numeros_de_pieza(s: &mut String, plano: &Plano, datos: &DatosDeDibujo) {
    let r = radio(plano);
    let letra = r * 13 / 10;
    s.push_str(r#"<g class="capa-piezas">"#);
    for (numero, x, y) in colocar(plano, &datos.piezas) {
        let _ = write!(
            s,
            r#"<g class="pieza" data-pieza="{numero}"><circle cx="{x}" cy="{y}" r="{r}"/><text x="{x}" y="{}" font-size="{letra}" text-anchor="middle">{numero}</text></g>"#,
            y + letra * 35 / 100
        );
    }
    s.push_str("</g>");
}

pub fn escapar(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
