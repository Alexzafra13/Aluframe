#![forbid(unsafe_code)]

use std::io::BufRead;

use anyhow::{Context, bail};
use servidor::aplicacion::restablecer_contrasena::restablecer_contrasena;
use servidor::arranque::{cargar_configuracion, preparar_estado, servir_http};

const AYUDA: &str = "Uso: alumframe [orden]

Sin orden      arranca el servidor (puerto 9090 por defecto)
preparar-base  crea o comprueba la base de datos y termina
restablecer-contrasena <nombre>
               cambia la contraseña del propietario desde la consola
version        muestra la versión";

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let argumentos: Vec<String> = std::env::args().skip(1).collect();
    let tiempo_de_ejecucion = tokio::runtime::Runtime::new()?;
    match argumentos
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => tiempo_de_ejecucion.block_on(servir()),
        ["preparar-base"] => tiempo_de_ejecucion.block_on(preparar_base()),
        ["restablecer-contrasena", nombre] => {
            tiempo_de_ejecucion.block_on(restablecer((*nombre).to_owned()))
        }
        ["version" | "--version"] => {
            println!("alumframe {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            println!("{AYUDA}");
            Ok(())
        }
    }
}

async fn servir() -> anyhow::Result<()> {
    let config = cargar_configuracion()?;
    let estado = preparar_estado(&config).await?;
    servir_http(&config, estado)
        .await
        .with_context(|| format!("no se pudo servir en el puerto {}", config.puerto))?;
    tracing::info!("AlumFrame detenido");
    Ok(())
}

async fn preparar_base() -> anyhow::Result<()> {
    let config = cargar_configuracion()?;
    preparar_estado(&config).await?;
    println!(
        "Base de datos preparada en {}",
        config.ruta_base_de_datos().display()
    );
    Ok(())
}

async fn restablecer(nombre: String) -> anyhow::Result<()> {
    let config = cargar_configuracion()?;
    let estado = preparar_estado(&config).await?;
    println!("Nueva contraseña para «{nombre}» (al menos 10 caracteres):");
    let primera = leer_linea()?;
    println!("Repítela:");
    if leer_linea()? != primera {
        bail!("las contraseñas no coinciden");
    }
    restablecer_contrasena(&estado, nombre, primera).await?;
    println!("Contraseña cambiada. Las sesiones abiertas de ese usuario se han cerrado.");
    Ok(())
}

fn leer_linea() -> anyhow::Result<String> {
    let mut linea = String::new();
    std::io::stdin().lock().read_line(&mut linea)?;
    Ok(linea.trim_end_matches(['\r', '\n']).to_owned())
}
