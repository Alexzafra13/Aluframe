//! Aplicación de escritorio (ADR-0015): el mismo programa en una ventana propia, que trabaja en
//! este equipo o contra el servidor del taller.

pub mod latido;
pub mod preferencias;
pub mod ventana;

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

/// Carpeta de la aplicación (datos, perfil de la ventana, registro y preferencias). Por usuario:
/// en Windows, `%LOCALAPPDATA%\AlumFrame`. `ALUMFRAME_CARPETA_APP` la fija para las pruebas.
pub fn carpeta_de_la_aplicacion() -> PathBuf {
    if let Some(elegida) = std::env::var_os("ALUMFRAME_CARPETA_APP") {
        return PathBuf::from(elegida);
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local).join("AlumFrame");
    }
    if let Some(datos) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(datos).join("alumframe");
    }
    std::env::var_os("HOME").map_or_else(
        || PathBuf::from("alumframe"),
        |casa| PathBuf::from(casa).join(".local/share/alumframe"),
    )
}

/// ¿Hay ya un AlumFrame contestando en ese puerto? Así un segundo clic en el icono solo abre la
/// ventana, y otro programa en el puerto no se confunde con el nuestro.
pub fn responde_alumframe(puerto: u16) -> bool {
    let consultar = || -> std::io::Result<bool> {
        let direccion = SocketAddr::from((Ipv4Addr::LOCALHOST, puerto));
        let espera = Duration::from_secs(2);
        let mut conexion = TcpStream::connect_timeout(&direccion, espera)?;
        conexion.set_read_timeout(Some(espera))?;
        conexion.write_all(b"GET /health HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")?;
        let mut respuesta = Vec::new();
        conexion.read_to_end(&mut respuesta)?;
        Ok(String::from_utf8_lossy(&respuesta).contains("\"estado\":\"ok\""))
    };
    consultar().unwrap_or(false)
}
