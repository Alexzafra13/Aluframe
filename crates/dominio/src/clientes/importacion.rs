use std::collections::{HashMap, HashSet};

use super::{DatosDeCliente, EntradaDeCliente, validar_cliente};
use crate::csv::{FilaCsv, TablaCsv, leer_csv};
use crate::informe::Informe;
use crate::nif::Nif;

pub const COLUMNAS_DE_CLIENTES: [&str; 12] = [
    "tipo",
    "nombre",
    "nif",
    "email",
    "telefono",
    "calle",
    "cp",
    "localidad",
    "provincia",
    "pais",
    "forma_de_pago",
    "notas",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilaDeCliente {
    pub linea: usize,
    pub datos: DatosDeCliente,
}

/// Todas las columnas son opcionales, pero al menos una debe estar en la cabecera.
pub fn leer_clientes(texto: &str) -> (Vec<FilaDeCliente>, Informe) {
    let mut informe = Informe::default();
    let tabla = match leer_csv(texto) {
        Ok(tabla) => tabla,
        Err(error) => {
            informe.error("", error.to_string());
            return (Vec::new(), informe);
        }
    };
    for sobrante in tabla.columnas_desconocidas(&COLUMNAS_DE_CLIENTES) {
        informe.aviso(
            "cabecera",
            format!("la columna «{sobrante}» no se usa y se ignora"),
        );
    }
    if COLUMNAS_DE_CLIENTES
        .iter()
        .all(|c| tabla.columna(c).is_none())
    {
        let plantilla = COLUMNAS_DE_CLIENTES.join(";");
        informe.error(
            "cabecera",
            format!("no hay ninguna columna conocida. La plantilla es: {plantilla}"),
        );
        return (Vec::new(), informe);
    }
    if tabla.filas.is_empty() {
        informe.error("", "el archivo solo tiene la cabecera: no hay clientes");
    }
    let filas = leer_filas(&tabla, &mut informe);
    (filas, informe)
}

fn leer_filas(tabla: &TablaCsv, informe: &mut Informe) -> Vec<FilaDeCliente> {
    let mut filas = Vec::with_capacity(tabla.filas.len());
    let mut nifs: HashMap<String, usize> = HashMap::new();
    for fila in &tabla.filas {
        let donde = format!("línea {}", fila.linea);
        match validar_cliente(&entrada(tabla, fila)) {
            Err(errores) => {
                for e in errores {
                    informe.error(&donde, format!("{}: {}", e.campo, e.mensaje));
                }
            }
            Ok(datos) => {
                let nif = datos.nif.as_ref().map(|n| n.como_texto().to_owned());
                if let Some(anterior) = nif.and_then(|n| nifs.insert(n, fila.linea)) {
                    informe.error(&donde, format!("el NIF ya aparece en la línea {anterior}"));
                    continue;
                }
                filas.push(FilaDeCliente {
                    linea: fila.linea,
                    datos,
                });
            }
        }
    }
    filas
}

fn entrada(tabla: &TablaCsv, fila: &FilaCsv) -> EntradaDeCliente {
    let campo = |nombre: &str| fila.campo(tabla.columna(nombre)).unwrap_or_default();
    EntradaDeCliente {
        tipo: campo("tipo"),
        nombre: campo("nombre"),
        nif: campo("nif"),
        email: campo("email"),
        telefono: campo("telefono"),
        fiscal_calle: campo("calle"),
        fiscal_cp: campo("cp"),
        fiscal_localidad: campo("localidad"),
        fiscal_provincia: campo("provincia"),
        fiscal_pais: campo("pais"),
        forma_de_pago: campo("forma_de_pago"),
        notas: campo("notas"),
    }
}

/// Un NIF que ya está en la base de datos no se duplica ni actualiza la ficha: se avisa y se salta.
pub fn apartar_existentes(
    filas: Vec<FilaDeCliente>,
    nifs_existentes: &HashSet<String>,
    informe: &mut Informe,
) -> Vec<FilaDeCliente> {
    filas
        .into_iter()
        .filter(|fila| {
            let repetido = fila
                .datos
                .nif
                .as_ref()
                .map(Nif::como_texto)
                .filter(|n| nifs_existentes.contains(*n));
            if let Some(nif) = repetido {
                let mensaje = format!("ya hay un cliente con el NIF {nif}: esta fila se salta");
                informe.aviso(&format!("línea {}", fila.linea), mensaje);
            }
            repetido.is_none()
        })
        .collect()
}
