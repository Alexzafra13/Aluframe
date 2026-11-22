use askama::Template;

#[derive(Template)]
#[template(path = "escritorio_conexion.html")]
pub struct PaginaConexion {
    pub servidor: String,
    pub en_servidor: bool,
    pub aviso: Option<String>,
}

/// Tras elegir el servidor: la ventana pasa a su dirección (una redirección tras un formulario la
/// bloquearía la política `form-action 'self'`).
#[derive(Template)]
#[template(path = "escritorio_conectando.html")]
pub struct PaginaConectando {
    pub direccion: String,
}
