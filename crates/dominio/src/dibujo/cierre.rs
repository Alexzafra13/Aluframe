//! Cierre embutido de la corredera, como en la foto del propietario: una placa alargada con el
//! pie redondeado, la ranura negra del tirador arriba y un punto debajo. La placa va en el color
//! de los herrajes (blanca o negra, ADR-0019); la ranura, siempre negra.

use std::fmt::Write;

/// `x` es el canto izquierdo de la placa; `grosor`, el de la hoja.
pub fn dibujar(s: &mut String, x: i64, centro_y: i64, grosor: i64) {
    let ancho = (grosor / 2).max(1);
    let alto = grosor * 9 / 2;
    let y = centro_y - alto / 2;
    let _ = write!(
        s,
        r#"<rect class="manilla" x="{x}" y="{y}" width="{ancho}" height="{alto}" rx="{}"/>"#,
        ancho / 2
    );
    let ranura_ancho = (ancho / 2).max(1);
    let ranura_alto = alto * 2 / 5;
    let ranura_y = y + ancho * 3 / 4;
    let ranura_x = x + (ancho - ranura_ancho) / 2;
    let _ = write!(
        s,
        r##"<rect class="ranura" x="{ranura_x}" y="{ranura_y}" width="{ranura_ancho}" height="{ranura_alto}" rx="{}" fill="#15191D" stroke="#5C636B"/><circle class="ranura" cx="{}" cy="{}" r="{}" fill="#15191D"/>"##,
        ranura_ancho / 2,
        x + ancho / 2,
        ranura_y + ranura_alto + ancho * 2 / 5,
        (ancho / 10).max(1)
    );
}
