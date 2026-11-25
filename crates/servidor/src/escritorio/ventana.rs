//! La ventana de la aplicación de escritorio: Edge (o Chrome/Chromium) en modo aplicación, sin
//! pestañas ni barra de direcciones (ADR-0015). Con un perfil propio, el proceso que se lanza es
//! el de la ventana: cuando termina, la ventana se ha cerrado y el programa puede cerrarse.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// Busca un navegador que admita `--app`. `ALUMFRAME_NAVEGADOR` lo fija (pruebas y casos raros).
pub fn buscar_navegador() -> Option<PathBuf> {
    if let Some(elegido) = std::env::var_os("ALUMFRAME_NAVEGADOR") {
        return Some(PathBuf::from(elegido));
    }
    candidatos().into_iter().find(|ruta| ruta.is_file())
}

#[cfg(windows)]
fn candidatos() -> Vec<PathBuf> {
    let mut rutas = Vec::new();
    for base in ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"] {
        if let Some(carpeta) = std::env::var_os(base) {
            let carpeta = PathBuf::from(carpeta);
            rutas.push(carpeta.join(r"Microsoft\Edge\Application\msedge.exe"));
            rutas.push(carpeta.join(r"Google\Chrome\Application\chrome.exe"));
        }
    }
    rutas
}

#[cfg(not(windows))]
fn candidatos() -> Vec<PathBuf> {
    let nombres = [
        "microsoft-edge",
        "chromium",
        "chromium-browser",
        "google-chrome",
    ];
    let rutas = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&rutas)
        .flat_map(|carpeta| nombres.iter().map(move |n| carpeta.join(n)))
        .collect()
}

/// Abre `url` en una ventana propia. Devuelve el proceso de la ventana para esperar a que se
/// cierre, o `None` si no hay navegador compatible y se abrió el predeterminado.
pub fn abrir_ventana(url: &str, perfil: &Path) -> std::io::Result<Option<Child>> {
    let Some(navegador) = buscar_navegador() else {
        abrir_con_el_sistema(url)?;
        return Ok(None);
    };
    let ventana = Command::new(navegador)
        .arg(format!("--app={url}"))
        .arg(format!("--user-data-dir={}", perfil.display()))
        .args([
            "--no-first-run",
            "--no-default-browser-check",
            "--window-size=1280,860",
        ])
        .spawn()?;
    Ok(Some(ventana))
}

fn abrir_con_el_sistema(url: &str) -> std::io::Result<()> {
    let mut orden = if cfg!(windows) {
        let mut orden = Command::new("cmd");
        orden.args(["/C", "start", "", url]);
        orden
    } else if cfg!(target_os = "macos") {
        let mut orden = Command::new("open");
        orden.arg(url);
        orden
    } else {
        let mut orden = Command::new("xdg-open");
        orden.arg(url);
        orden
    };
    orden.spawn().map(|_| ())
}

/// Sin consola, un fallo al arrancar se enseña en una ventana con el motivo.
pub fn mostrar_error(mensaje: &str, perfil: &Path) {
    let html = format!(
        "<!doctype html><meta charset=utf-8><title>Aluframe</title>\
         <body style=\"font-family:sans-serif;margin:2rem;background:#ECE9E0;color:#15191D\">\
         <h1>Aluframe no ha podido arrancar</h1><p>{}</p>\
         <p>Si se repite, avisa a quien mantiene el programa.</p>",
        escapar_html(mensaje)
    );
    let url = format!("data:text/html;charset=utf-8,{}", codificar_url(&html));
    if let Err(error) = abrir_ventana(&url, perfil) {
        tracing::error!("no se pudo mostrar el error: {error}");
    }
}

pub(crate) fn escapar_html(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn codificar_url(texto: &str) -> String {
    texto
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(b).to_string()
            }
            otro => format!("%{otro:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_mensaje_de_error_no_puede_inyectar_html() {
        assert_eq!(escapar_html("<b>&"), "&lt;b&gt;&amp;");
        assert_eq!(codificar_url("a b<"), "a%20b%3C");
    }
}
