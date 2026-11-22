#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::{StatusCode, header};
use comun::{Contexto, formulario, get, post, texto};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::series_incluidas::cargar_series_incluidas;

#[tokio::test]
async fn sirve_las_siluetas_y_nada_fuera_de_su_carpeta() {
    let contexto = Contexto::nuevo().await;
    contexto.con_propietario().await;
    let silueta = contexto
        .pedir(get("/perfiles/aluval/21210.svg", None))
        .await;
    assert_eq!(silueta.status(), StatusCode::OK);
    assert_eq!(silueta.headers()[header::CONTENT_TYPE], "image/svg+xml");
    for ruta in [
        "/perfiles/aluval/indice.txt",
        "/perfiles/aluval/99999.svg",
        "/perfiles/../series/aluprom-21.yaml",
    ] {
        let respuesta = contexto.pedir(get(ruta, None)).await;
        assert_eq!(respuesta.status(), StatusCode::NOT_FOUND, "{ruta}");
    }
}

/// Calcula en vivo una corredera Aluprom 21 de 150 × 120 con las opciones dadas.
async fn calcular_aluprom(opciones: &[(&str, &str)]) -> String {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let cliente = validar_cliente(&EntradaDeCliente::default()).unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    let csrf = actor.token_csrf.as_str();
    contexto
        .pedir(post(
            "/presupuestos",
            Some(&token),
            &formulario(&[("_csrf", csrf), ("cliente", "1")]),
        ))
        .await;
    let mut ventana = vec![
        ("_csrf", csrf),
        ("seleccion", "1|corredera_2_hojas"),
        ("ancho", "150"),
        ("alto", "120"),
        ("iva", "2100"),
    ];
    ventana.extend_from_slice(opciones);
    let respuesta = contexto
        .pedir(post(
            "/presupuestos/1/lineas/calcular",
            Some(&token),
            &formulario(&ventana),
        ))
        .await;
    texto(respuesta).await
}

fn tiene_silueta(html: &str, referencia: &str) -> bool {
    html.contains(&format!("src=\"/perfiles/aluval/{referencia}.svg\""))
}

/// Lo que pidió el propietario: junto al número de cada pieza, el dibujo de su perfil.
#[tokio::test]
async fn las_piezas_de_aluprom_21_llevan_el_dibujo_de_su_perfil() {
    let html = calcular_aluprom(&[]).await;
    for referencia in ["21210", "21220", "21310", "21340", "21350"] {
        assert!(
            tiene_silueta(&html, referencia),
            "falta la silueta {referencia}"
        );
    }
    assert!(html.contains("Hoja lateral") && html.contains("Hoja cierre"));
}

#[tokio::test]
async fn con_climalit_salen_las_hojas_climalit_y_su_dibujo() {
    let html = calcular_aluprom(&[("opcion.acristalamiento", "climalit")]).await;
    for referencia in ["21312", "21341", "21351"] {
        assert!(
            tiene_silueta(&html, referencia),
            "falta la silueta {referencia}"
        );
    }
    assert!(html.contains("Hoja lateral climalit"));
    assert!(!tiene_silueta(&html, "21310"));
}
