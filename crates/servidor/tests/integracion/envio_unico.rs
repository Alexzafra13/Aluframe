//! Un formulario enviado dos veces (doble toque en la tablet) solo se atiende una vez (ADR-0036).

use crate::comun;

use axum::http::StatusCode;
use comun::{Contexto, destino, formulario, get, post, texto};

fn alta(csrf: &str, envio: &str, nombre: &str) -> String {
    formulario(&[
        ("_csrf", csrf),
        ("_envio", envio),
        ("nombre", nombre),
        ("telefono", "+34612345678"),
        ("fiscal_pais", "España"),
    ])
}

async fn veces_en_la_lista(contexto: &Contexto, token: &str, nombre: &str) -> usize {
    let lista = texto(contexto.pedir(get("/clientes", Some(token))).await).await;
    lista.matches(nombre).count()
}

#[tokio::test]
async fn el_mismo_envio_dos_veces_da_de_alta_un_solo_cliente_y_lleva_al_mismo_sitio() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cuerpo = alta(&actor.token_csrf, "envio-1", "Carmen Repetida");
    let primera = contexto
        .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
        .await;
    let segunda = contexto
        .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
        .await;
    assert_eq!(primera.status(), StatusCode::SEE_OTHER);
    assert_eq!(segunda.status(), StatusCode::SEE_OTHER);
    assert_eq!(destino(&primera), destino(&segunda));
    assert_eq!(
        veces_en_la_lista(&contexto, &token, "Carmen Repetida").await,
        1
    );
}

#[tokio::test]
async fn dos_envios_a_la_vez_solo_se_atienden_una_vez() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let cuerpo = alta(&actor.token_csrf, "envio-2", "Luis Simultaneo");
    let (a, b) = tokio::join!(
        contexto.pedir(post("/clientes/nuevo", Some(&token), &cuerpo)),
        contexto.pedir(post("/clientes/nuevo", Some(&token), &cuerpo)),
    );
    assert_eq!(destino(&a), destino(&b));
    assert_eq!(
        veces_en_la_lista(&contexto, &token, "Luis Simultaneo").await,
        1
    );
}

#[tokio::test]
async fn tras_un_error_se_puede_corregir_y_enviar_el_mismo_formulario() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    let sin_nombre = alta(&actor.token_csrf, "envio-3", "");
    let error = contexto
        .pedir(post("/clientes/nuevo", Some(&token), &sin_nombre))
        .await;
    assert_eq!(error.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let corregido = alta(&actor.token_csrf, "envio-3", "Ana Corregida");
    let bien = contexto
        .pedir(post("/clientes/nuevo", Some(&token), &corregido))
        .await;
    assert_eq!(bien.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        veces_en_la_lista(&contexto, &token, "Ana Corregida").await,
        1
    );
}

#[tokio::test]
async fn envios_distintos_o_sin_marca_se_atienden_cada_uno() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = contexto.con_propietario().await;
    for envio in ["envio-4", "envio-5", ""] {
        let cuerpo = alta(&actor.token_csrf, envio, "Pedro Varias");
        let respuesta = contexto
            .pedir(post("/clientes/nuevo", Some(&token), &cuerpo))
            .await;
        assert_eq!(respuesta.status(), StatusCode::SEE_OTHER);
    }
    assert_eq!(
        veces_en_la_lista(&contexto, &token, "Pedro Varias").await,
        3
    );
}
