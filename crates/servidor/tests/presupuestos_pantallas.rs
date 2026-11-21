#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use axum::http::StatusCode;
use comun::presupuestos::{SERIE_DEMO, elegir_reglas, preparar_demo};
use comun::{Contexto, PIN, destino, formulario, get, post, texto};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};

async fn preparado() -> (Contexto, String, String) {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_DEMO).await;
    elegir_reglas(&estado, &actor).await;
    (contexto, token, actor.token_csrf)
}

#[tokio::test]
async fn crea_el_presupuesto_desde_el_cliente_y_abre_el_editor() {
    let (contexto, token, csrf) = preparado().await;
    let cuerpo = formulario(&[("_csrf", &csrf), ("cliente", "1")]);
    let respuesta = contexto
        .pedir(post("/presupuestos", Some(&token), &cuerpo))
        .await;
    assert_eq!(destino(&respuesta), Some("/presupuestos/1/lineas/nueva"));
    let editor = texto(
        contexto
            .pedir(get("/presupuestos/1/lineas/nueva", Some(&token)))
            .await,
    )
    .await;
    assert!(editor.contains("DEMO_FICTICIA · DEMO corredera 2 hojas (ficticia)"));
    assert!(editor.contains("Escribe el ancho y el alto"));
}

#[tokio::test]
async fn sin_series_el_editor_explica_donde_importarlas() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cliente = validar_cliente(&EntradaDeCliente::default()).unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    let cuerpo = formulario(&[("_csrf", &actor.token_csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &cuerpo))
        .await;
    let editor = texto(
        contexto
            .pedir(get("/presupuestos/1/lineas/nueva", Some(&token)))
            .await,
    )
    .await;
    assert!(editor.contains("No hay ninguna serie cargada"));
    assert!(editor.contains("href=\"/catalogo/series/importar\""));
}

#[tokio::test]
async fn el_calculo_en_vivo_devuelve_dibujo_piezas_y_total() {
    let (contexto, token, csrf) = preparado().await;
    let cuerpo = formulario(&[("_csrf", &csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &cuerpo))
        .await;
    let ventana = [
        ("_csrf", csrf.as_str()),
        ("seleccion", "1|corredera_2_hojas_demo"),
        ("ancho", "150"),
        ("alto", "120"),
        ("iva", "2100"),
    ];
    let respuesta = contexto
        .pedir(post(
            "/presupuestos/1/lineas/calcular",
            Some(&token),
            &formulario(&ventana),
        ))
        .await;
    let html = texto(respuesta).await;
    assert!(html.contains("<svg class=\"dibujo-ventana\""));
    assert!(html.contains("id=\"piezas\"") && html.contains("hx-swap-oob"));
    assert!(html.contains("Línea sin IVA"));
    let malo = [
        ("_csrf", csrf.as_str()),
        ("seleccion", "1|corredera_2_hojas_demo"),
        ("ancho", "ancho"),
        ("alto", "120"),
    ];
    let html = texto(
        contexto
            .pedir(post(
                "/presupuestos/1/lineas/calcular",
                Some(&token),
                &formulario(&malo),
            ))
            .await,
    )
    .await;
    assert!(html.contains("Escribe el ancho en cm"));
}

/// Presupuesto 1 con una ventana demo de 150 × 120 y una línea libre de 80 €.
async fn con_dos_lineas() -> (Contexto, String) {
    let (contexto, token, csrf) = preparado().await;
    let nuevo = formulario(&[("_csrf", &csrf), ("cliente", "1")]);
    contexto
        .pedir(post("/presupuestos", Some(&token), &nuevo))
        .await;
    let ventana = [
        ("_csrf", csrf.as_str()),
        ("seleccion", "1|corredera_2_hojas_demo"),
        ("ancho", "150"),
        ("alto", "120"),
        ("iva", "2100"),
    ];
    let respuesta = contexto
        .pedir(post(
            "/presupuestos/1/lineas/nueva",
            Some(&token),
            &formulario(&ventana),
        ))
        .await;
    assert_eq!(destino(&respuesta), Some("/presupuestos/1?aviso=linea"));
    let libre = [
        ("_csrf", csrf.as_str()),
        ("descripcion", "Instalación"),
        ("cantidad", "1"),
        ("precio", "80"),
        ("iva", "2100"),
    ];
    contexto
        .pedir(post(
            "/presupuestos/1/lineas/libre",
            Some(&token),
            &formulario(&libre),
        ))
        .await;
    (contexto, token)
}

#[tokio::test]
async fn la_ficha_muestra_las_lineas_y_por_que_no_se_acepta() {
    let (contexto, token) = con_dos_lineas().await;
    let ficha = texto(
        contexto
            .pedir(get("/presupuestos/1?aviso=linea", Some(&token)))
            .await,
    )
    .await;
    assert!(ficha.contains("Línea guardada."));
    assert!(ficha.contains("Línea 1 · DEMO corredera 2 hojas (ficticia) · 150 × 120 cm"));
    assert!(ficha.contains("Línea 2 · Instalación"));
    assert!(ficha.contains("No se puede aceptar todavía"));
}

#[tokio::test]
async fn la_hoja_impresa_no_lleva_costes() {
    let (contexto, token) = con_dos_lineas().await;
    let impreso = texto(
        contexto
            .pedir(get("/presupuestos/1/imprimir", Some(&token)))
            .await,
    )
    .await;
    assert!(impreso.contains("P-2026-0001") && impreso.contains("Taller de prueba"));
    assert!(impreso.contains("Provisional"));
    assert!(impreso.contains("Importe sin IVA") && impreso.contains("Total con IVA"));
    assert!(
        !impreso.contains("Perfiles"),
        "la hoja del cliente no lleva costes"
    );
}

#[tokio::test]
async fn una_linea_libre_sin_descripcion_avisa_en_la_ficha() {
    let (contexto, token, csrf) = preparado().await;
    contexto
        .pedir(post(
            "/presupuestos",
            Some(&token),
            &formulario(&[("_csrf", &csrf), ("cliente", "1")]),
        ))
        .await;
    let libre = [
        ("_csrf", csrf.as_str()),
        ("descripcion", " "),
        ("cantidad", "1"),
        ("precio", "80"),
        ("iva", "2100"),
    ];
    let respuesta = contexto
        .pedir(post(
            "/presupuestos/1/lineas/libre",
            Some(&token),
            &formulario(&libre),
        ))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(texto(respuesta).await.contains("Escribe la descripción"));
}

#[tokio::test]
async fn el_operario_no_ve_presupuestos_ni_ajustes_de_precio() {
    let (contexto, _, _) = preparado().await;
    let (_, propietario) = contexto.con_propietario_existente().await;
    let operario = contexto.con_operario(&propietario).await;
    let solicitud = SolicitudDeEntrada {
        usuario_id: operario,
        clave: PIN.into(),
        ip: "10.0.0.5".into(),
        duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
    };
    let ResultadoDeEntrada::Correcta { token, .. } =
        entrar(&contexto.estado, solicitud).await.unwrap()
    else {
        panic!("el operario no pudo entrar");
    };
    for ruta in ["/presupuestos", "/ajustes/presupuestos"] {
        assert_eq!(
            contexto.pedir(get(ruta, Some(&token))).await.status(),
            StatusCode::NOT_FOUND,
            "{ruta}"
        );
    }
}
