//! Pantalla de recuperación de la aplicación de escritorio (ver `escritorio::recuperacion`):
//! se sirve en lugar del programa cuando la base de datos de este equipo no se puede usar.
//! Los botones llevan un token aleatorio: otra página web no puede pulsarlos por su cuenta.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

use super::recursos::servir_recurso;
use super::rutas::escritorio::latido;
use crate::autenticacion::tokens_iguales;
use crate::escritorio::recuperacion::{Recuperacion, abrir_carpeta, apartar_base};
use crate::escritorio::ventana::escapar_html;

struct Estado {
    recuperacion: Recuperacion,
    token: String,
    /// Marca de tiempo para el nombre de la base apartada.
    marca: String,
}

#[derive(Deserialize)]
struct Pulsado {
    token: String,
}

pub fn crear_router_de_recuperacion(
    recuperacion: Recuperacion,
    (token, marca): (String, String),
) -> Router {
    let estado = Arc::new(Estado {
        recuperacion,
        token,
        marca,
    });
    Router::new()
        .route("/", get(mostrar))
        .route("/abrir-carpeta", post(abrir))
        .route("/empezar-de-cero", post(empezar_de_cero))
        .route("/escritorio/latido", get(latido))
        .route("/recursos/{*ruta}", get(servir_recurso))
        .with_state(estado)
}

fn pagina(cuerpo: &str) -> Html<String> {
    Html(format!(
        "<!doctype html><html lang=es><meta charset=utf-8>\
         <meta name=viewport content=\"width=device-width,initial-scale=1\"><title>Aluframe</title>\
         <body style=\"font-family:sans-serif;margin:2rem;max-width:40rem;background:#ECE9E0;\
         color:#15191D;line-height:1.5\">{cuerpo}\
         <script src=\"/recursos/js/latido.js\" defer></script></body></html>"
    ))
}

fn boton(accion: &str, texto: &str, token: &str) -> String {
    format!(
        "<form method=post action=\"{accion}\" style=\"margin:1rem 0\">\
         <input type=hidden name=token value=\"{token}\">\
         <button type=submit style=\"min-height:44px;padding:0 1.2rem;font-size:1rem;\
         border-radius:8px;border:1.5px solid #1F4E79;background:#FBFAF6;color:#1F4E79\">\
         {texto}</button></form>"
    )
}

async fn mostrar(State(e): State<Arc<Estado>>) -> Html<String> {
    let r = &e.recuperacion;
    let mut cuerpo = format!(
        "<h1>Aluframe no ha podido arrancar</h1><p>{}</p>\
         <p>Carpeta de datos:<br><code>{}</code></p>{}",
        escapar_html(&r.explicacion),
        escapar_html(&r.carpeta.display().to_string()),
        boton("/abrir-carpeta", "Abrir la carpeta de datos", &e.token)
    );
    if r.puede_empezar_de_cero {
        cuerpo.push_str(
            "<h2>Empezar de cero</h2><p>Si esa base no tiene datos de verdad (por ejemplo, de \
             pruebas antiguas), puedes empezar con una nueva. La antigua <strong>no se \
             borra</strong>: se guarda en la subcarpeta «apartadas», por si acaso.</p>",
        );
        cuerpo.push_str(&boton("/empezar-de-cero", "Empezar de cero", &e.token));
    }
    pagina(&cuerpo)
}

async fn abrir(State(e): State<Arc<Estado>>, Form(p): Form<Pulsado>) -> Response {
    if !tokens_iguales(&p.token, &e.token) {
        return StatusCode::FORBIDDEN.into_response();
    }
    abrir_carpeta(&e.recuperacion.carpeta);
    Redirect::to("/").into_response()
}

async fn empezar_de_cero(State(e): State<Arc<Estado>>, Form(p): Form<Pulsado>) -> Response {
    let r = &e.recuperacion;
    if !tokens_iguales(&p.token, &e.token) || !r.puede_empezar_de_cero {
        return StatusCode::FORBIDDEN.into_response();
    }
    match apartar_base(&r.carpeta, &e.marca) {
        Ok(destino) => pagina(&format!(
            "<h1>Listo</h1><p>La base antigua está guardada en<br><code>{}</code></p>\
             <p>Cierra esta ventana y vuelve a abrir Aluframe: empezará con una base nueva.</p>",
            escapar_html(&destino.display().to_string())
        ))
        .into_response(),
        Err(error) => pagina(&format!(
            "<h1>No se pudo apartar la base</h1><p>{}</p><p>Cierra Aluframe y vuelve a \
             intentarlo; si se repite, mueve el archivo alumframe.db a mano desde la carpeta.</p>",
            escapar_html(&error.to_string())
        ))
        .into_response(),
    }
}
