use crate::comun;

use axum::http::StatusCode;
use comun::{Contexto, PIN, destino, formulario, get, get_htmx, post, texto};
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};

#[tokio::test]
async fn no_da_de_alta_un_cliente_sin_nombre_ni_telefono() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cuerpo = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("tipo", "particular"),
        ("version", "0"),
    ]);
    let respuesta = contexto
        .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = texto(respuesta).await;
    assert!(html.contains("escribe el nombre o la razón social"));
    assert!(html.contains("escribe un teléfono de contacto"));
    let lista = texto(contexto.pedir(get("/clientes", Some(&token))).await).await;
    assert!(lista.contains("Aún no hay clientes"));
}

#[tokio::test]
async fn guarda_el_movil_sin_prefijo_y_agrupado() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cuerpo = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("nombre", "Jose"),
        ("telefono", "+34612345678"),
        ("fiscal_pais", "España"),
    ]);
    let respuesta = contexto
        .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
        .await;
    assert_eq!(destino(&respuesta), Some("/clientes/1?aviso=creado"));
    let ficha = texto(contexto.pedir(get("/clientes/1", Some(&token))).await).await;
    assert!(ficha.contains("612 345 678"));
}

#[tokio::test]
async fn un_nif_mal_escrito_no_guarda_y_conserva_lo_escrito() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cuerpo = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("nombre", "Construcciones Peña"),
        ("telefono", "924555555"),
        ("nif", "12345678A"),
        ("d0_calle", "Calle Mayor 3"),
    ]);
    let respuesta = contexto
        .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = texto(respuesta).await;
    assert!(html.contains("la letra o el dígito de control no coincide"));
    assert!(html.contains("value=\"Construcciones Peña\""));
    assert!(html.contains("value=\"Calle Mayor 3\""));
    let lista = texto(contexto.pedir(get("/clientes", Some(&token))).await).await;
    assert!(lista.contains("Aún no hay clientes"));
}

#[tokio::test]
async fn copia_la_direccion_fiscal_como_montaje_y_anade_otras() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cuerpo = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("nombre", "Ana"),
        ("telefono", "612345678"),
        ("fiscal_calle", "Calle Sevilla 1"),
        ("fiscal_localidad", "Zafra"),
        ("misma_direccion", "1"),
        ("d0_etiqueta", "Obra"),
        ("d0_calle", "Calle Mayor 3"),
        ("d1_calle", ""),
    ]);
    contexto
        .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
        .await;
    let ficha = texto(contexto.pedir(get("/clientes/1", Some(&token))).await).await;
    assert!(ficha.contains("Dirección fiscal · Calle Sevilla 1, Zafra"));
    assert!(ficha.contains("Obra · Calle Mayor 3"));
}

#[tokio::test]
async fn la_busqueda_con_htmx_devuelve_solo_la_lista() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    for nombre in ["José Peña", "Ana Ruiz"] {
        let cuerpo = formulario(&[
            ("_csrf", &actor.token_csrf),
            ("nombre", nombre),
            ("telefono", "612345678"),
        ]);
        contexto
            .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
            .await;
    }
    let html = texto(contexto.pedir(get_htmx("/clientes?q=pena", &token)).await).await;
    assert!(!html.contains("<html"));
    assert!(html.contains("José Peña"));
    assert!(!html.contains("Ana Ruiz"));
    let nif = texto(
        contexto
            .pedir(get_htmx("/clientes/comprobar-nif?nif=12345678z", &token))
            .await,
    )
    .await;
    assert!(nif.contains("Correcto: 12345678Z"));
}

#[tokio::test]
async fn avisa_si_otra_persona_guardo_antes_sin_perder_lo_escrito() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let alta = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("nombre", "Ana"),
        ("telefono", "612345678"),
    ]);
    contexto
        .pedir(post("/clientes/nuevo", Some(&token), &alta))
        .await;
    let primera = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("version", "1"),
        ("nombre", "Ana María"),
        ("telefono", "612345678"),
    ]);
    let respuesta = contexto
        .pedir(post("/clientes/1/editar", Some(&token), &primera))
        .await;
    assert_eq!(destino(&respuesta), Some("/clientes/1?aviso=guardado"));
    let tarde = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("version", "1"),
        ("nombre", "Ana Luisa"),
        ("telefono", "612345678"),
    ]);
    let respuesta = contexto
        .pedir(post("/clientes/1/editar", Some(&token), &tarde))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let html = texto(respuesta).await;
    assert!(html.contains("Otra persona ha cambiado esto"));
    assert!(html.contains("value=\"Ana Luisa\""));
}

#[tokio::test]
async fn borrar_exige_confirmacion() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let alta = formulario(&[
        ("_csrf", &actor.token_csrf),
        ("nombre", "Ana"),
        ("telefono", "612345678"),
    ]);
    contexto
        .pedir(post("/clientes/nuevo", Some(&token), &alta))
        .await;
    let sin = formulario(&[("_csrf", &actor.token_csrf)]);
    let respuesta = contexto
        .pedir(post("/clientes/1/borrar", Some(&token), &sin))
        .await;
    assert_eq!(respuesta.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let con = formulario(&[("_csrf", &actor.token_csrf), ("confirmar", "si")]);
    let respuesta = contexto
        .pedir(post("/clientes/1/borrar", Some(&token), &con))
        .await;
    assert_eq!(destino(&respuesta), Some("/clientes?aviso=borrado"));
    let ficha = contexto.pedir(get("/clientes/1", Some(&token))).await;
    assert_eq!(ficha.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn el_operario_recibe_404_en_las_pantallas_de_clientes() {
    let contexto = Contexto::nuevo().await;
    let (_, propietario) = contexto.con_propietario().await;
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
    for ruta in [
        "/clientes/nuevo",
        "/clientes/comprobar-nif?nif=1",
        "/clientes/direccion-en-blanco",
    ] {
        let respuesta = contexto.pedir(get(ruta, Some(&token))).await;
        assert_eq!(respuesta.status(), StatusCode::NOT_FOUND, "{ruta}");
    }
}

/// Fallo corregido: «Nuevo presupuesto» y «Nueva factura» solo ofrecían los 50 primeros clientes;
/// el resto parecía no existir y la pantalla invitaba a darlo de alta otra vez.
#[tokio::test]
async fn nuevo_presupuesto_y_factura_ofrecen_todos_los_clientes() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    for numero in 1..=60 {
        let nombre = if numero == 60 {
            "Zoe Zapata".to_owned()
        } else {
            format!("Cliente {numero:02}")
        };
        let cliente = dominio::clientes::validar_cliente(&dominio::clientes::EntradaDeCliente {
            nombre,
            ..Default::default()
        })
        .unwrap();
        servidor::aplicacion::clientes::crear_cliente(&contexto.estado, &actor, cliente, vec![])
            .await
            .unwrap();
    }
    for ruta in ["/presupuestos", "/facturas"] {
        let pagina = texto(contexto.pedir(get(ruta, Some(&token))).await).await;
        assert!(pagina.contains("Zoe Zapata"), "{ruta}");
        assert!(pagina.contains("Cliente 01"), "{ruta}");
    }
}

/// La búsqueda mientras se escribe pide solo la lista; la navegación sin recarga (ADR-0036),
/// también por HTMX, pide la página entera.
#[tokio::test]
async fn la_navegacion_sin_recarga_recibe_la_pagina_entera_de_clientes() {
    use axum::body::Body;
    use axum::http::{Request, header};
    let contexto = Contexto::nuevo().await;
    let (token, _) = contexto.con_propietario().await;
    let trozo = texto(contexto.pedir(get_htmx("/clientes", &token)).await).await;
    assert!(!trozo.contains("<html"));
    let navegacion = Request::get("/clientes")
        .header(header::COOKIE, format!("alumframe_sesion={token}"))
        .header("hx-request", "true")
        .header("hx-boosted", "true")
        .body(Body::empty())
        .unwrap();
    let pagina = texto(contexto.pedir(navegacion).await).await;
    assert!(pagina.contains("<html"));
}
