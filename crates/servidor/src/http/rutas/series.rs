use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::series::Serie;
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::importar_serie::{importar_serie, previsualizar_serie};
use crate::aplicacion::series::{cargar_serie, leer_medida, listar_series, probar_ventana};
use crate::aplicacion::taller::{AjustesDeTaller, leer_ajustes_de_taller};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{
    FilaSerie, MENU, PaginaImportarSerie, PaginaProbarVentana, PaginaSeries, VistaPrueba,
    renderizar,
};

const SECCION: &str = "catalogo";

#[derive(Deserialize)]
pub struct Importada {
    importada: Option<String>,
    version: Option<i64>,
}

#[derive(Deserialize)]
pub struct ArchivoDeSerie {
    texto: String,
}

#[derive(Deserialize)]
pub struct MedidasDePrueba {
    tipologia: Option<String>,
    ancho: Option<String>,
    alto: Option<String>,
}

pub async fn mostrar_series(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(importada): Query<Importada>,
) -> Result<Response, ErrorApp> {
    let series = listar_series(&estado, &actor).await?;
    let confirmacion = importada.importada.map(|codigo| {
        format!(
            "Serie {codigo} importada como versión {}.",
            importada.version.unwrap_or(1)
        )
    });
    let pagina = PaginaSeries {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        series: series.iter().map(FilaSerie::from).collect(),
        confirmacion,
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn mostrar_importacion(ActorActual(actor): ActorActual) -> Result<Response, ErrorApp> {
    pagina_de_importacion(&actor, String::new(), None)
}

pub async fn revisar_importacion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(archivo): Form<ArchivoDeSerie>,
) -> Result<Response, ErrorApp> {
    let vista = previsualizar_serie(&estado, &actor, archivo.texto.clone()).await?;
    pagina_de_importacion(&actor, archivo.texto, Some(vista))
}

pub async fn confirmar_importacion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(archivo): Form<ArchivoDeSerie>,
) -> Result<Response, ErrorApp> {
    match importar_serie(&estado, &actor, archivo.texto.clone()).await {
        Ok((codigo, version)) => {
            let destino = format!("/catalogo/series?importada={codigo}&version={version}");
            Ok(Redirect::to(&destino).into_response())
        }
        Err(ErrorApp::Validacion(_)) => {
            let vista = previsualizar_serie(&estado, &actor, archivo.texto.clone()).await?;
            let pagina = pagina_de_importacion(&actor, archivo.texto, Some(vista))?;
            Ok((StatusCode::UNPROCESSABLE_ENTITY, pagina).into_response())
        }
        Err(otro) => Err(otro),
    }
}

fn pagina_de_importacion(
    actor: &Actor,
    texto: String,
    previsualizacion: Option<crate::aplicacion::importar_serie::Previsualizacion>,
) -> Result<Response, ErrorApp> {
    let pagina = PaginaImportarSerie {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        texto,
        previsualizacion,
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn mostrar_prueba(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(codigo): Path<String>,
    Query(medidas): Query<MedidasDePrueba>,
) -> Result<Response, ErrorApp> {
    let guardada = cargar_serie(&estado, &actor, codigo).await?;
    let serie = &guardada.serie;
    let primera = serie
        .tipologias
        .first()
        .map(|t| t.codigo.clone())
        .unwrap_or_default();
    let elegida = medidas
        .tipologia
        .filter(|t| serie.tipologia(t).is_some())
        .unwrap_or(primera);
    let (ancho, alto) = (
        medidas.ancho.unwrap_or_default(),
        medidas.alto.unwrap_or_default(),
    );
    let taller = leer_ajustes_de_taller(&estado).await?;
    let (prueba, aviso) = calcular_prueba(serie, &elegida, (&ancho, &alto), &taller)?;
    let pagina = PaginaProbarVentana {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        codigo: serie.codigo.clone(),
        version: guardada.numero_version,
        tipologias: serie
            .tipologias
            .iter()
            .map(|t| (t.codigo.clone(), t.nombre.clone()))
            .collect(),
        elegida,
        ancho,
        alto,
        aviso,
        prueba,
    };
    Ok(renderizar(&pagina)?.into_response())
}

type PruebaOAviso = (Option<VistaPrueba>, Option<String>);

fn calcular_prueba(
    serie: &Serie,
    tipologia: &str,
    (ancho, alto): (&str, &str),
    taller: &AjustesDeTaller,
) -> Result<PruebaOAviso, ErrorApp> {
    if ancho.is_empty() || alto.is_empty() {
        return Ok((None, None));
    }
    let calculo = leer_medida(ancho, "ancho")
        .and_then(|a| leer_medida(alto, "alto").map(|h| (a, h)))
        .and_then(|(a, h)| probar_ventana(serie, (tipologia, a, h), taller));
    match calculo {
        Ok(prueba) => Ok((Some(VistaPrueba::from(&prueba)), None)),
        Err(ErrorApp::Validacion(mensaje)) => Ok((None, Some(mensaje))),
        Err(otro) => Err(otro),
    }
}
