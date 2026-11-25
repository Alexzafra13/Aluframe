//! Lo que pidió el propietario (2026-11-24): la ventana dice qué tipo es y de qué serie y marca
//! (con su logotipo), y cada herraje lleva su dibujo y el modelo que se compra (ADR-0029).

use crate::comun;

use axum::http::{StatusCode, header};
use comun::{Contexto, formulario, get, post, texto};
use dominio::catalogo::*;
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::catalogo::{crear_articulo, crear_proveedor};
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::series_incluidas::cargar_series_incluidas;

/// Da de alta el cierre embutido en el catálogo como el modelo que hoy usa el taller.
async fn dar_de_alta_el_cierre(contexto: &Contexto, actor: &servidor::aplicacion::Actor) {
    let proveedor = validar_proveedor(&EntradaDeProveedor {
        nombre: "Ferretería".into(),
        ..EntradaDeProveedor::default()
    })
    .unwrap();
    let proveedor = crear_proveedor(&contexto.estado, actor, proveedor)
        .await
        .unwrap();
    let cierre = validar_articulo(&EntradaDeArticulo {
        referencia: "CIERRE-EMBUTIDO".into(),
        descripcion: "Cierre de embutir Alualpha 267 i".into(),
        tipo: "herraje".into(),
        unidad: "unidad".into(),
        ..EntradaDeArticulo::default()
    })
    .unwrap();
    crear_articulo(&contexto.estado, actor, proveedor, cierre)
        .await
        .unwrap();
}

/// Calcula en vivo una corredera Aluprom 21 de 4 hojas con el cierre en el catálogo.
async fn calcular_con_el_cierre_en_el_catalogo() -> String {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    dar_de_alta_el_cierre(&contexto, &actor).await;
    let cliente = validar_cliente(&EntradaDeCliente::default()).unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    let csrf = actor.token_csrf.as_str();
    let cuerpo = formulario(&[("_csrf", csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &cuerpo))
        .await;
    let ventana = formulario(&[
        ("_csrf", csrf),
        ("seleccion", "1|corredera_4_hojas"),
        ("ancho", "303"),
        ("alto", "120"),
        ("iva", "2100"),
    ]);
    let ruta = "/presupuestos/1/lineas/calcular";
    texto(contexto.pedir(post(ruta, Some(&token), &ventana)).await).await
}

#[tokio::test]
async fn la_ventana_dice_su_tipo_y_su_serie_con_el_logotipo_de_la_marca() {
    let html = calcular_con_el_cierre_en_el_catalogo().await;
    assert!(html.contains(r#"class="ficha-tipo""#));
    assert!(html.contains("Ventana corredera 4 hojas"));
    assert!(html.contains(r#"src="/marcas/aluval.png" alt="Aluval""#));
    assert!(html.contains("Aluprom 21"));
}

#[tokio::test]
async fn los_herrajes_llevan_su_dibujo_y_el_modelo_del_catalogo() {
    let html = calcular_con_el_cierre_en_el_catalogo().await;
    for dibujo in ["cierre-embutido", "kit-plasticos", "rueda", "tornillo"] {
        assert!(
            html.contains(&format!("src=\"/perfiles/herrajes/{dibujo}.svg\"")),
            "falta el dibujo {dibujo}"
        );
    }
    assert!(html.contains("Cierre embutido"));
    assert!(html.contains("Cierre de embutir Alualpha 267 i"));
    assert!(html.contains("src=\"/perfiles/aluval/21344.svg\""));
}

#[tokio::test]
async fn sirve_los_logotipos_y_nada_fuera_de_su_carpeta() {
    let contexto = Contexto::nuevo().await;
    contexto.con_propietario().await;
    let logo = contexto.pedir(get("/marcas/aluval.png", None)).await;
    assert_eq!(logo.status(), StatusCode::OK);
    assert_eq!(logo.headers()[header::CONTENT_TYPE], "image/png");
    for ruta in ["/marcas/no-existe.png", "/marcas/aluval.svg"] {
        let respuesta = contexto.pedir(get(ruta, None)).await;
        assert_eq!(respuesta.status(), StatusCode::NOT_FOUND, "{ruta}");
    }
}
