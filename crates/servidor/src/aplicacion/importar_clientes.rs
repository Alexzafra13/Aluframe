use dominio::acceso::Accion;
use dominio::clientes::{FilaDeCliente, apartar_existentes, leer_clientes};
use dominio::informe::Informe;
use rusqlite::Connection;

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::autenticacion::calcular_sha256;
use crate::bd::consultas::clientes;
use crate::bd::consultas::importaciones::{self, FilaImportacion, NuevaImportacion};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

pub struct ClienteAImportar {
    pub linea: usize,
    pub nombre: String,
    pub nif: String,
    pub telefono: String,
    pub localidad: String,
}

pub struct PrevisualizacionDeClientes {
    pub clientes: Vec<ClienteAImportar>,
    pub errores: Vec<String>,
    pub avisos: Vec<String>,
    pub sha256: String,
}

impl PrevisualizacionDeClientes {
    pub fn se_puede_importar(&self) -> bool {
        self.errores.is_empty() && !self.clientes.is_empty()
    }
}

/// Lee el archivo y aparta los NIF que ya están: lo mismo en la vista previa y al confirmar.
fn analizar(con: &Connection, texto: &str) -> Result<(Vec<FilaDeCliente>, Informe), ErrorApp> {
    let (filas, mut informe) = leer_clientes(texto);
    let existentes = clientes::nifs_existentes(con)?;
    let filas = apartar_existentes(filas, &existentes, &mut informe);
    if !informe.tiene_errores() && filas.is_empty() {
        informe.error("", "no queda ningún cliente nuevo que importar");
    }
    Ok((filas, informe))
}

pub async fn previsualizar_clientes(
    estado: &EstadoApp,
    actor: &Actor,
    texto: String,
) -> Result<PrevisualizacionDeClientes, ErrorApp> {
    autorizar(actor, Accion::GestionarClientes)?;
    let sha256 = calcular_sha256(&texto);
    let (filas, informe) = estado.bd.leer(move |con| analizar(con, &texto)).await?;
    let texto_de = |v: &Option<String>| v.clone().unwrap_or_default();
    let clientes = filas
        .iter()
        .map(|f| ClienteAImportar {
            linea: f.linea,
            nombre: f
                .datos
                .nombre
                .clone()
                .unwrap_or_else(|| "(sin nombre)".into()),
            nif: f
                .datos
                .nif
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            telefono: texto_de(&f.datos.telefono),
            localidad: texto_de(&f.datos.fiscal_localidad),
        })
        .collect();
    Ok(PrevisualizacionDeClientes {
        clientes,
        errores: informe.errores.iter().map(ToString::to_string).collect(),
        avisos: informe.avisos.iter().map(ToString::to_string).collect(),
        sha256,
    })
}

/// Vuelve a analizar dentro de la transacción y exige que el archivo sea el revisado.
pub async fn importar_clientes(
    estado: &EstadoApp,
    actor: &Actor,
    texto: String,
    sha_revisado: String,
) -> Result<(i64, usize), ErrorApp> {
    autorizar(actor, Accion::GestionarClientes)?;
    if calcular_sha256(&texto) != sha_revisado {
        return Err(ErrorApp::Validacion(
            "El archivo no es el que se revisó. Vuelve a revisarlo antes de confirmar.".into(),
        ));
    }
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let (filas, informe) = analizar(tx, &texto)?;
            if informe.tiene_errores() {
                return Err(ErrorApp::Validacion(
                    "El archivo tiene errores; revisa la vista previa.".into(),
                ));
            }
            let importacion = guardar(tx, &filas, (&sha_revisado, autor, ahora))?;
            Ok((importacion, filas.len()))
        })
        .await
}

fn guardar(
    tx: &Connection,
    filas: &[FilaDeCliente],
    (sha256, autor, ahora): (&str, i64, time::OffsetDateTime),
) -> Result<i64, ErrorApp> {
    let cuando = formatear_utc(ahora);
    let nueva = NuevaImportacion {
        tipo: "clientes",
        sha256,
        filas: i64::try_from(filas.len()).unwrap_or(i64::MAX),
        usuario_id: autor,
        importada_en: &cuando,
    };
    let importacion = importaciones::insertar(tx, &nueva)?;
    for fila in filas {
        clientes::insertar(tx, &fila.datos, &cuando, Some(importacion))?;
    }
    let cambio = Cambio {
        ahora,
        autor,
        accion: "importar_clientes",
        entidad: "importacion",
        entidad_id: importacion,
    };
    anotar(tx, &cambio)?;
    Ok(importacion)
}

/// Quita los clientes importados que nadie ha tocado; los modificados se quedan.
pub async fn deshacer_importacion_de_clientes(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<(usize, usize), ErrorApp> {
    autorizar(actor, Accion::GestionarClientes)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let importacion = importaciones::buscar(tx, id)?
                .filter(|i| i.tipo == "clientes")
                .ok_or(ErrorApp::NoEncontrado)?;
            importaciones::marcar_deshecha(tx, importacion.id, &formatear_utc(ahora))?;
            let resultado = clientes::quitar_importados_sin_cambios(tx, id)?;
            let cambio = Cambio {
                ahora,
                autor,
                accion: "deshacer_importacion",
                entidad: "importacion",
                entidad_id: id,
            };
            anotar(tx, &cambio)?;
            Ok(resultado)
        })
        .await
}

pub async fn ultimas_importaciones(
    estado: &EstadoApp,
    actor: &Actor,
    tipo: &'static str,
) -> Result<Vec<FilaImportacion>, ErrorApp> {
    autorizar(actor, Accion::GestionarClientes)?;
    estado
        .bd
        .leer(move |con| importaciones::ultimas(con, tipo, 5))
        .await
}
