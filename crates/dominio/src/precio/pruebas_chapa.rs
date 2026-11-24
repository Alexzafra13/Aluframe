use super::pruebas::{bp, catalogo, perfiles, resultado};
use super::*;
use crate::unidades::Longitud;

/// Mientras el propietario no decida cómo se cobra la chapa, no hay precio final con chapa.
#[test]
fn una_ventana_con_chapa_no_tiene_precio_final() {
    let mut con_chapa = resultado();
    con_chapa.chapas.push(crate::series::ChapaCalculada {
        articulo: "CHAPA".into(),
        nombre: None,
        cantidad: 2,
        ancho: None,
        alto: Some(Longitud::desde_decimas_de_mm(5_000)),
    });
    let coste = calcular_coste_de_materiales(
        &con_chapa,
        &perfiles(),
        &catalogo(),
        (Some(ModoPerfiles::BarrasEnteras), bp(1_500)),
    );
    assert!(coste.faltan.iter().any(|f| f.contains("cómo se cobra")));
}
