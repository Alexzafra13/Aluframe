#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, Response, header};
use servidor::aplicacion::Actor;
use servidor::aplicacion::primer_arranque::{DatosDelPropietario, crear_propietario_inicial};
use servidor::aplicacion::sesion::identificar;
use servidor::aplicacion::usuarios::crear_operario;
use servidor::bd::{BaseDeDatos, preparar_esquema};
use servidor::estado::EstadoApp;
use servidor::http::crear_router;
use servidor::reloj::RelojFijo;
use time::macros::datetime;
use tower::ServiceExt;

pub const CONTRASENA: &str = "contrasena-del-dueno";
pub const PIN: &str = "123456";

pub struct Contexto {
    pub estado: EstadoApp,
    pub reloj: Arc<RelojFijo>,
}

impl Contexto {
    pub async fn nuevo() -> Contexto {
        let bd = BaseDeDatos::en_memoria().unwrap();
        bd.con_escritor(|con| preparar_esquema(con, "2026-11-21T08:00:00Z").unwrap())
            .unwrap();
        let reloj = Arc::new(RelojFijo::new(datetime!(2026-11-21 08:00 UTC)));
        let estado = EstadoApp::nuevo(bd, reloj.clone()).await.unwrap();
        Contexto { estado, reloj }
    }

    pub fn router(&self) -> Router {
        crear_router(self.estado.clone())
    }

    pub async fn con_propietario(&self) -> (String, Actor) {
        let datos = DatosDelPropietario {
            nombre: "Manuel".into(),
            contrasena: CONTRASENA.into(),
            confirmacion: CONTRASENA.into(),
        };
        let token = crear_propietario_inicial(&self.estado, datos)
            .await
            .unwrap();
        let actor = identificar(&self.estado, &token).await.unwrap().unwrap();
        (token, actor)
    }

    pub async fn con_propietario_existente(&self) -> (String, Actor) {
        let solicitud = servidor::aplicacion::entrar::SolicitudDeEntrada {
            usuario_id: 1,
            clave: CONTRASENA.into(),
            ip: "127.0.0.1".into(),
            duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
        };
        match servidor::aplicacion::entrar::entrar(&self.estado, solicitud)
            .await
            .unwrap()
        {
            servidor::aplicacion::entrar::ResultadoDeEntrada::Correcta { token, .. } => {
                let actor = identificar(&self.estado, &token).await.unwrap().unwrap();
                (token, actor)
            }
            otro => panic!("el propietario no pudo entrar: {otro:?}"),
        }
    }

    pub async fn con_operario(&self, propietario: &Actor) -> i64 {
        crear_operario(&self.estado, propietario, "Juan", PIN.into())
            .await
            .unwrap()
    }

    pub async fn pedir(&self, peticion: Request<Body>) -> Response<Body> {
        self.router().oneshot(peticion).await.unwrap()
    }
}

pub fn get(ruta: &str, token: Option<&str>) -> Request<Body> {
    let mut peticion = Request::get(ruta);
    if let Some(token) = token {
        peticion = peticion.header(header::COOKIE, format!("alumframe_sesion={token}"));
    }
    peticion.body(Body::empty()).unwrap()
}

pub fn post(ruta: &str, token: Option<&str>, formulario: &str) -> Request<Body> {
    let mut peticion =
        Request::post(ruta).header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(token) = token {
        peticion = peticion.header(header::COOKIE, format!("alumframe_sesion={token}"));
    }
    peticion.body(Body::from(formulario.to_owned())).unwrap()
}

pub async fn texto(respuesta: Response<Body>) -> String {
    let bytes = to_bytes(respuesta.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

pub fn destino(respuesta: &Response<Body>) -> Option<&str> {
    respuesta
        .headers()
        .get(header::LOCATION)
        .and_then(|valor| valor.to_str().ok())
}

pub fn token_de_la_cookie(cookie: &str) -> String {
    let primer_par = cookie.split(';').next().unwrap();
    primer_par.split_once('=').unwrap().1.to_owned()
}

/// Codifica pares como un formulario `application/x-www-form-urlencoded`.
pub fn formulario(pares: &[(&str, &str)]) -> String {
    pares
        .iter()
        .map(|(clave, valor)| format!("{}={}", codificar(clave), codificar(valor)))
        .collect::<Vec<_>>()
        .join("&")
}

pub fn codificar(texto: &str) -> String {
    let mut codificado = String::new();
    for byte in texto.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                codificado.push(char::from(byte))
            }
            otro => codificado.push_str(&format!("%{otro:02X}")),
        }
    }
    codificado
}

pub fn get_htmx(ruta: &str, token: &str) -> Request<Body> {
    Request::get(ruta)
        .header(header::COOKIE, format!("alumframe_sesion={token}"))
        .header("hx-request", "true")
        .body(Body::empty())
        .unwrap()
}
