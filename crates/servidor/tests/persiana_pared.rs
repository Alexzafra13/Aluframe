#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Persiana suelta para montar en pared con guías de L (ADR-0033), con la serie que viene con el
//! programa: se pide hecha, se escribe la medida del hueco y va al presupuesto con su dibujo.

mod comun;

use std::collections::BTreeMap;
use std::path::Path;

use comun::{Contexto, destino, formulario, get, post, texto};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use dominio::dibujo::{DatosDeDibujo, dibujar_ventana};
use dominio::series::{ResultadoVentana, Serie, calcular_ventana, importar_serie};
use dominio::unidades::{Decimal, Longitud};
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::series_incluidas::cargar_series_incluidas;
use servidor::series::leer_yaml;

const TIPOLOGIA: &str = "persiana_pared";

fn serie() -> Serie {
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/series/persiana-pared.yaml");
    let valor = leer_yaml(&std::fs::read_to_string(ruta).unwrap()).unwrap();
    importar_serie(&valor).unwrap().0
}

fn calcular() -> ResultadoVentana {
    let medida = Decimal::desde_entero(120);
    calcular_ventana(&serie(), TIPOLOGIA, medida, medida).unwrap()
}

#[test]
fn no_tiene_nada_que_cortar_y_la_persiana_se_pide() {
    let r = calcular();
    assert!(r.piezas.is_empty() && r.piezas_pendientes.is_empty());
    assert_eq!(r.vidrio.as_deref(), Some(&[][..]), "no lleva cristal");
    assert!(r.accesorios.iter().any(|a| a.articulo == "PERSIANA"));
    assert!(
        !r.pendientes.iter().any(|p| p.contains("longitud de barra")),
        "sin perfiles a cortar no hace falta barra: {:?}",
        r.pendientes
    );
}

/// El ala de la guía (80 o 100 mm) y el cajón los dará el propietario: sin ellos, sin precio.
#[test]
fn sin_la_guia_ni_el_cajon_sale_incompleta() {
    let r = calcular();
    assert!(r.es_incompleta());
    let pendientes = r.pendientes.join(" ");
    assert!(pendientes.contains("guía de L"), "{pendientes}");
    assert!(pendientes.contains("alto del cajón"), "{pendientes}");
    let persiana = r.persiana.unwrap();
    assert_eq!(
        persiana.ancho_ventana,
        Longitud::desde_decimas_de_mm(12_000),
        "en pared la medida escrita es el hueco y no se le quita nada"
    );
}

fn dibujo(opciones: &[(&str, &str)]) -> String {
    let serie = serie();
    let opciones: BTreeMap<String, String> = opciones
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let medida = Decimal::desde_entero(120);
    let r = dominio::series::calcular_ventana_con_opciones(
        &serie,
        TIPOLOGIA,
        (medida, medida),
        &opciones,
    )
    .unwrap();
    let lado = Longitud::desde_decimas_de_mm(12_000);
    let tipologia = serie.tipologia(TIPOLOGIA).unwrap();
    dibujar_ventana(&DatosDeDibujo::desde_calculo(
        tipologia,
        &r,
        &opciones,
        (lado, lado),
    ))
}

#[test]
fn se_dibuja_con_cajon_guias_de_l_lamas_y_cinta_y_acota_el_hueco() {
    let svg = dibujo(&[]);
    for parte in [
        "persiana-cajon",
        "persiana-guia",
        "persiana-lamas",
        "persiana-asa",
        "persiana-cinta",
        "persiana-recogedor",
    ] {
        assert!(svg.contains(parte), "falta {parte}");
    }
    assert!(svg.contains(">hueco 120<"), "la medida escrita es el hueco");
    assert!(!svg.contains("class=\"cristal\""), "no hay ventana");
    assert!(!svg.contains(">marco "), "ni marco que acotar");
}

/// Con la cinta a la izquierda, la cinta va por la guía izquierda, fuera del hueco.
#[test]
fn la_cinta_va_por_la_guia_del_lado_elegido() {
    let x = |svg: &str| -> i64 {
        let resto = &svg[svg.find("class=\"persiana-cinta\"").unwrap()..];
        let x = &resto[resto.find(" x=\"").unwrap() + 4..];
        x[..x.find('"').unwrap()].parse().unwrap()
    };
    let derecha = x(&dibujo(&[]));
    let izquierda = x(&dibujo(&[("cinta", "izquierda")]));
    assert!(derecha > izquierda + 12_000, "{izquierda} … {derecha}");
}

/// Con las series del programa, un cliente y un presupuesto vacío: (contexto, sesión, csrf,
/// valor de «Serie y tipología» de la persiana en pared).
async fn presupuesto_abierto() -> (Contexto, String, String, String) {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let cliente = validar_cliente(&EntradaDeCliente::default()).unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    let csrf = actor.token_csrf.clone();
    let cuerpo = formulario(&[("_csrf", &csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &cuerpo))
        .await;
    let editor = texto(
        contexto
            .pedir(get("/presupuestos/1/lineas/nueva", Some(&token)))
            .await,
    )
    .await;
    let marca = format!("|{TIPOLOGIA}\"");
    let fin = editor.find(&marca).expect("se elige en Serie y tipología");
    let inicio = editor[..fin].rfind('"').unwrap() + 1;
    let seleccion = format!("{}|{TIPOLOGIA}", &editor[inicio..fin]);
    (contexto, token, csrf, seleccion)
}

#[tokio::test]
async fn se_anade_al_presupuesto_como_una_ventana() {
    let (contexto, token, csrf, seleccion) = presupuesto_abierto().await;
    let csrf = csrf.as_str();
    let linea = formulario(&[
        ("_csrf", csrf),
        ("seleccion", &seleccion),
        ("ancho", "120"),
        ("alto", "140"),
        ("iva", "2100"),
        ("opcion.cinta", "derecha"),
    ]);
    let html = texto(
        contexto
            .pedir(post(
                "/presupuestos/1/lineas/calcular",
                Some(&token),
                &linea,
            ))
            .await,
    )
    .await;
    assert!(html.contains("persiana-lamas"), "{html}");
    assert!(html.contains("Persiana"));
    let guardada = contexto
        .pedir(post("/presupuestos/1/lineas/nueva", Some(&token), &linea))
        .await;
    assert_eq!(destino(&guardada), Some("/presupuestos/1?aviso=linea"));
    let ficha = texto(contexto.pedir(get("/presupuestos/1", Some(&token))).await).await;
    assert!(ficha.contains("Persiana en pared"), "{ficha}");
    assert!(ficha.contains("+ Ventana, mosquitera o persiana"));
}
