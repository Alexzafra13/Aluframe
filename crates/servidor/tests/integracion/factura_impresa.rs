//! La hoja A4 de la factura sale de lo congelado al emitir.

use crate::comun;

use comun::facturas::{con_cliente, emitida_libre};
use comun::{Contexto, get, texto};
use servidor::aplicacion::facturas::cargar_factura;

/// Fallo corregido: la fecha de la operación se podía escribir en el borrador, pero no se
/// congelaba ni se imprimía. La ley pide que conste cuando difiere de la de expedición.
#[tokio::test]
async fn la_fecha_de_la_operacion_se_congela_y_se_imprime_si_difiere() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = con_cliente(&contexto).await;
    let estado = &contexto.estado;
    let anterior = emitida_libre(estado, &actor, (Some("2026-10-15"), None)).await;
    let foto = cargar_factura(estado, &actor, anterior)
        .await
        .unwrap()
        .factura
        .snapshot_json
        .unwrap();
    assert!(
        foto.contains("\"fecha_operacion\":\"2026-10-15\""),
        "{foto}"
    );
    let hoja = texto(
        contexto
            .pedir(get(&format!("/facturas/{anterior}/imprimir"), Some(&token)))
            .await,
    )
    .await;
    assert!(hoja.contains("Fecha de la operación") && hoja.contains("15/10/2026"));

    let mismo_dia = emitida_libre(estado, &actor, (None, None)).await;
    let hoja = texto(
        contexto
            .pedir(get(
                &format!("/facturas/{mismo_dia}/imprimir"),
                Some(&token),
            ))
            .await,
    )
    .await;
    assert!(
        !hoja.contains("Fecha de la operación"),
        "si es la de emisión no se repite"
    );
}
