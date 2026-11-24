#![allow(clippy::unwrap_used, clippy::panic)]

//! Mosquitera enrollable (ADR-0031) con la serie que viene con el programa.

mod comun;

use std::path::Path;

use comun::{Contexto, formulario, get, post, texto};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use dominio::series::{ResultadoVentana, calcular_ventana, importar_serie};
use dominio::unidades::{Decimal, Longitud};
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::series_incluidas::cargar_series_incluidas;
use servidor::series::leer_yaml;

fn calcular(ancho: i64, alto: i64) -> ResultadoVentana {
    let ruta =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/series/mosquitera-enrollable.yaml");
    let valor = leer_yaml(&std::fs::read_to_string(ruta).unwrap()).unwrap();
    let (serie, _) = importar_serie(&valor).unwrap();
    let medida = Decimal::desde_entero;
    calcular_ventana(&serie, "mosquitera_enrollable", medida(ancho), medida(alto)).unwrap()
}

fn perfiles(r: &ResultadoVentana) -> Vec<(u32, String)> {
    let calculadas = r.piezas.iter().map(|p| (p.numero, p.perfil.clone()));
    let pendientes = r
        .piezas_pendientes
        .iter()
        .map(|p| (p.numero, p.perfil.clone()));
    let mut todas: Vec<_> = calculadas.chain(pendientes).collect();
    todas.sort();
    todas
}

#[test]
fn hasta_140_de_alto_lleva_cajon_de_45_y_la_guia_mide_alto_menos_4() {
    let r = calcular(120, 140);
    let esperados = ["CAJON_45", "GUIA_LATERAL"];
    let numeros: Vec<(u32, String)> = (1..).zip(esperados.map(String::from)).collect();
    assert_eq!(perfiles(&r), numeros, "sin el cajón de 50 ni el terminal");
    let cajon = r.piezas.iter().find(|p| p.perfil == "CAJON_45").unwrap();
    assert_eq!(
        cajon.longitud,
        Longitud::desde_decimas_de_mm(11_800),
        "ancho - 2"
    );
    let guia = r
        .piezas
        .iter()
        .find(|p| p.perfil == "GUIA_LATERAL")
        .unwrap();
    assert_eq!(
        (guia.cantidad, guia.longitud),
        (2, Longitud::desde_decimas_de_mm(13_600))
    );
}

/// El terminal viene dentro del cabezal: no se corta (propietario, 2026-11-24).
#[test]
fn el_terminal_no_se_corta() {
    let r = calcular(120, 180);
    assert!(!perfiles(&r).iter().any(|(_, perfil)| perfil == "TERMINAL"));
}

#[test]
fn por_encima_de_140_lleva_cajon_de_50_y_la_guia_mide_alto_menos_5() {
    let r = calcular(120, 180);
    assert!(r.piezas.iter().any(|p| p.perfil == "CAJON_50"));
    assert!(!r.piezas.iter().any(|p| p.perfil == "CAJON_45"));
    let guia = r
        .piezas
        .iter()
        .find(|p| p.perfil == "GUIA_LATERAL")
        .unwrap();
    assert_eq!(
        (guia.cantidad, guia.longitud),
        (2, Longitud::desde_decimas_de_mm(17_500))
    );
    assert!(r.piezas_pendientes.is_empty());
}

#[test]
fn no_lleva_cristal_ni_lo_pide() {
    let r = calcular(100, 150);
    assert_eq!(r.vidrio.as_deref(), Some(&[][..]));
    assert!(!r.pendientes.iter().any(|p| p.contains("Vidrio")));
    assert!(
        r.es_incompleta(),
        "en borrador: sin precio final hasta confirmar los datos"
    );
}

#[tokio::test]
async fn se_presupuesta_como_una_ventana_y_sale_su_dibujo() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let cliente = validar_cliente(&EntradaDeCliente::default()).unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    let csrf = actor.token_csrf.as_str();
    let cuerpo = formulario(&[("_csrf", csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &cuerpo))
        .await;
    let editor = texto(
        contexto
            .pedir(get("/presupuestos/1/lineas/nueva", Some(&token)))
            .await,
    )
    .await;
    let marca = "|mosquitera_enrollable\"";
    let fin = editor
        .find(marca)
        .expect("la mosquitera se elige en Serie y tipología");
    let inicio = editor[..fin].rfind('"').unwrap() + 1;
    let seleccion = format!("{}|mosquitera_enrollable", &editor[inicio..fin]);
    assert!(editor.contains("Mosquitera enrollable (Sin marca) · Mosquitera enrollable"));
    let ventana = formulario(&[
        ("_csrf", csrf),
        ("seleccion", &seleccion),
        ("ancho", "120"),
        ("alto", "160"),
        ("iva", "2100"),
    ]);
    let ruta = "/presupuestos/1/lineas/calcular";
    let html = texto(contexto.pedir(post(ruta, Some(&token), &ventana)).await).await;
    assert!(html.contains(r#"class="malla""#));
    assert!(html.contains("Cajón 50"), "{html}");
    assert!(html.contains("Kit mosquitera"));
}
