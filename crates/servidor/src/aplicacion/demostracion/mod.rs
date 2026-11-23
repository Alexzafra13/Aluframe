//! La demostración (ADR-0028): el mismo programa con una base aparte, llena de datos
//! inventados, para probar y aprender sin tocar los datos de verdad. Se entra y se sale desde la
//! pantalla de entrada; la base lleva una marca y nunca se mezcla con la real.

mod catalogo;
mod clientes;
mod entrada;
mod taller;
mod ventas;

pub use entrada::{QuienEntra, entrar_sin_clave};

use dominio::acceso::Accion;

use super::acabados_incluidos::cargar_acabados_incluidos;
use super::primer_arranque::{DatosDelPropietario, crear_propietario_inicial};
use super::series_incluidas::cargar_series_incluidas;
use super::sesion::identificar;
use super::{Actor, autorizar};
use crate::bd::consultas::configuracion::marcar_como_demostracion;
use crate::bd::{BaseDeDatos, copia, preparar_esquema};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

/// Los usuarios necesitan una clave para existir, pero en la demostración se entra sin ella
/// (`entrada.rs`) y no se enseña.
pub const DUENO: &str = "Dueño (demo)";
const CONTRASENA: &str = "demostracion";
pub const OPERARIO: &str = "Juan (demo)";
const PIN: &str = "123456";

/// En la demostración no hay copias de seguridad ni se exportan o importan datos.
pub fn exigir_datos_reales(estado: &EstadoApp) -> Result<(), ErrorApp> {
    if estado.es_demostracion() {
        Err(ErrorApp::NoEncontrado)
    } else {
        Ok(())
    }
}

/// La primera vez que alguien entra, se rellena (una sola vez aunque entren dos a la vez).
pub async fn asegurar_rellena(demo: &EstadoApp) -> Result<(), ErrorApp> {
    let _turno = demo.turno_de_demostracion().lock().await;
    if demo.hay_usuarios() {
        return Ok(());
    }
    rellenar(demo).await
}

/// «Empezar de nuevo»: se rellena una base nueva en memoria y sustituye de una vez a la de la
/// demostración, con la misma restauración que usa la importación de datos.
pub async fn empezar_de_nuevo(demo: &EstadoApp, actor: &Actor) -> Result<(), ErrorApp> {
    if !demo.es_demostracion() {
        return Err(ErrorApp::NoEncontrado);
    }
    autorizar(actor, Accion::GestionarDatos)?;
    let _turno = demo.turno_de_demostracion().lock().await;
    let ahora = formatear_utc(demo.ahora());
    let bd = BaseDeDatos::en_memoria()?;
    bd.con_escritor(|con| preparar_esquema(con, &ahora))?
        .map_err(|e| ErrorApp::Interno(e.to_string()))?;
    let nueva = EstadoApp::nuevo(bd, demo.reloj())
        .await?
        .como_demostracion();
    cargar_series_incluidas(&nueva).await?;
    cargar_acabados_incluidos(&nueva).await?;
    rellenar(&nueva).await?;
    let carpeta = demo.carpeta_copias().to_path_buf();
    std::fs::create_dir_all(&carpeta).map_err(|e| ErrorApp::Interno(e.to_string()))?;
    let archivo = carpeta.join("demostracion-nueva.alumframe");
    let _ = std::fs::remove_file(&archivo);
    let destino = archivo.clone();
    nueva
        .bd
        .con_escritor_entero(move |con| copia::exportar(con, &destino))
        .await?;
    let origen = archivo.clone();
    demo.bd
        .con_escritor_entero(move |con| copia::restaurar(con, &origen, &ahora))
        .await?;
    let _ = std::fs::remove_file(&archivo);
    demo.fijar_hay_usuarios(true);
    Ok(())
}

async fn rellenar(estado: &EstadoApp) -> Result<(), ErrorApp> {
    let ahora = formatear_utc(estado.ahora());
    estado
        .bd
        .escribir(move |tx| marcar_como_demostracion(tx, &ahora))
        .await?;
    let dueno = crear_dueno(estado).await?;
    let operario = catalogo::preparar(estado, &dueno).await?;
    let clientes = clientes::crear(estado, &dueno).await?;
    ventas::crear(estado, (&dueno, &operario), &clientes).await
}

async fn crear_dueno(estado: &EstadoApp) -> Result<Actor, ErrorApp> {
    let datos = DatosDelPropietario {
        nombre: DUENO.into(),
        contrasena: CONTRASENA.into(),
        confirmacion: CONTRASENA.into(),
    };
    let token = crear_propietario_inicial(estado, datos).await?;
    identificar(estado, &token)
        .await?
        .ok_or_else(|| ErrorApp::Interno("la demostración no pudo crear al dueño".into()))
}
