use dominio::acceso::Accion;
use dominio::series::{EstadoSerie, Informe, Serie, importar_serie as validar_archivo};
use rusqlite::Connection;

use super::{Actor, autorizar};
use crate::autenticacion::calcular_sha256;
use crate::bd::consultas::auditoria::{Apunte, registrar};
use crate::bd::consultas::series::{self, NuevaVersion};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;
use crate::series::leer_yaml;

const TAMANO_MAXIMO: usize = 256 * 1024;

#[derive(Debug, Clone)]
pub struct Previsualizacion {
    pub codigo: Option<String>,
    pub fabricante: Option<String>,
    pub estado: Option<&'static str>,
    pub tipologias: Vec<String>,
    pub errores: Vec<String>,
    pub avisos: Vec<String>,
    pub version_siguiente: i64,
}

impl Previsualizacion {
    pub fn se_puede_importar(&self) -> bool {
        self.errores.is_empty() && self.codigo.is_some()
    }
}

pub fn texto_de_estado(estado: EstadoSerie) -> &'static str {
    match estado {
        EstadoSerie::Borrador => "borrador",
        EstadoSerie::Validada => "validada",
        EstadoSerie::Demo => "demo",
    }
}

pub(super) fn analizar(texto: &str, permitir_demo: bool) -> (Option<Serie>, Informe) {
    let mut informe = Informe::default();
    if texto.len() > TAMANO_MAXIMO {
        informe.error("", "El archivo es demasiado grande (máximo 256 KB).");
        return (None, informe);
    }
    let valor = match leer_yaml(texto) {
        Ok(valor) => valor,
        Err(mensaje) => {
            informe.error("", mensaje);
            return (None, informe);
        }
    };
    match validar_archivo(&valor) {
        Ok((serie, informe)) if serie.estado == EstadoSerie::Demo && !permitir_demo => {
            let mut informe = informe;
            informe.error(
                "estado",
                "las series de demostración (datos ficticios) no se cargan en el uso real",
            );
            (None, informe)
        }
        Ok((serie, informe)) => (Some(serie), informe),
        Err(informe) => (None, informe),
    }
}

pub async fn previsualizar_serie(
    estado: &EstadoApp,
    actor: &Actor,
    texto: String,
) -> Result<Previsualizacion, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let (serie, mut informe) = analizar(&texto, estado.permite_datos_demo());
    let codigo = serie.as_ref().map(|s| s.codigo.clone());
    let sha = calcular_sha256(&texto);
    let anterior = match codigo.clone() {
        Some(codigo) => {
            estado
                .bd
                .leer(move |con| series::buscar_ultima(con, &codigo))
                .await?
        }
        None => None,
    };
    if anterior.as_ref().is_some_and(|v| v.sha256 == sha) {
        informe.error("", "Esta versión ya está importada: no hay cambios.");
    }
    Ok(Previsualizacion {
        codigo,
        fabricante: serie.as_ref().map(|s| s.fabricante.clone()),
        estado: serie.as_ref().map(|s| texto_de_estado(s.estado)),
        tipologias: serie
            .iter()
            .flat_map(|s| s.tipologias.iter().map(|t| t.nombre.clone()))
            .collect(),
        errores: informe.errores.iter().map(ToString::to_string).collect(),
        avisos: informe.avisos.iter().map(ToString::to_string).collect(),
        version_siguiente: anterior.map_or(1, |v| v.numero + 1),
    })
}

/// Vuelve a validar dentro de la transacción: lo que se guarda es exactamente lo que se comprobó.
pub async fn importar_serie(
    estado: &EstadoApp,
    actor: &Actor,
    texto: String,
) -> Result<(String, i64), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let (serie, informe) = analizar(&texto, estado.permite_datos_demo());
    let Some(serie) = serie.filter(|_| !informe.tiene_errores()) else {
        return Err(ErrorApp::Validacion(
            "La serie tiene errores; revisa la vista previa.".into(),
        ));
    };
    let avisos = i64::try_from(informe.avisos.len()).unwrap_or(i64::MAX);
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let ahora = formatear_utc(ahora);
            let importacion = Importacion {
                serie: &serie,
                texto: &texto,
                avisos,
                autor: Some(autor),
                ahora: &ahora,
            };
            guardar(tx, &importacion)
        })
        .await
}

pub(super) struct Importacion<'a> {
    pub serie: &'a Serie,
    pub texto: &'a str,
    pub avisos: i64,
    pub autor: Option<i64>,
    pub ahora: &'a str,
}

pub(super) fn guardar(
    tx: &Connection,
    importacion: &Importacion,
) -> Result<(String, i64), ErrorApp> {
    let Importacion {
        serie,
        texto,
        avisos,
        autor,
        ahora,
    } = *importacion;
    let sha = calcular_sha256(texto);
    if series::buscar_ultima(tx, &serie.codigo)?.is_some_and(|v| v.sha256 == sha) {
        return Err(ErrorApp::Validacion(
            "Esta versión ya está importada: no hay cambios.".into(),
        ));
    }
    let nueva = NuevaVersion {
        codigo: &serie.codigo,
        fabricante: &serie.fabricante,
        estado: texto_de_estado(serie.estado),
        origen_yaml: texto,
        sha256: &sha,
        avisos,
        creada_en: ahora,
    };
    let (version_id, numero) = series::insertar_version(tx, &nueva)?;
    series::insertar_referencias(tx, version_id, &serie.referencias)?;
    let apunte = Apunte {
        cuando_en: ahora,
        usuario_id: autor,
        accion: if autor.is_some() {
            "importar_serie"
        } else {
            "cargar_serie_incluida"
        },
        entidad: "serie_version",
        entidad_id: Some(version_id),
    };
    registrar(tx, &apunte)?;
    Ok((serie.codigo.clone(), numero))
}
