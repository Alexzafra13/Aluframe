use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::tarifa::COLUMNAS_DE_TARIFA;
use dominio::unidades::Porcentaje;
use serde::Deserialize;

use super::catalogo_fichas::SECCION;
use crate::aplicacion::Actor;
use crate::aplicacion::catalogo::listar_proveedores;
use crate::aplicacion::importar_tarifa::{CabeceraDeTarifa, importar_tarifa, previsualizar_tarifa};
use crate::aplicacion::tarifas::{cargar_tarifa, deshacer_tarifa, listar_tarifas};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    FilaDeTarifa, MENU, PaginaImportarTarifa, PaginaTarifa, PaginaTarifas, PrecioEnPantalla,
    ValoresDeImportacion, VistaPreviaDeTarifa, opciones_de_proveedor, renderizar,
};

#[derive(Deserialize)]
pub struct Resultado {
    importada: Option<usize>,
    deshecha: Option<String>,
}

pub async fn mostrar_tarifas(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(resultado): Query<Resultado>,
) -> Result<Response, ErrorApp> {
    let tarifas = listar_tarifas(&estado, &actor).await?;
    let confirmacion = match (resultado.importada, resultado.deshecha) {
        (Some(n), _) => Some(format!(
            "Tarifa importada: {n} precios. Si algo no cuadra, pulsa «Deshacer»."
        )),
        (None, Some(_)) => Some("Versión deshecha: vuelve a estar vigente la anterior.".to_owned()),
        _ => None,
    };
    let pagina = PaginaTarifas {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        tarifas: tarifas.iter().map(FilaDeTarifa::from).collect(),
        aviso: None,
        confirmacion,
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn mostrar_tarifa(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let (tarifa, precios) = cargar_tarifa(&estado, &actor, id).await?;
    let pagina = PaginaTarifa {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        tarifa: FilaDeTarifa::from(&tarifa),
        precios: precios.iter().map(PrecioEnPantalla::from).collect(),
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn deshacer(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    deshacer_tarifa(&estado, &actor, id).await?;
    Ok(Redirect::to("/catalogo/tarifas?deshecha=si").into_response())
}

fn valores(campos: &Campos) -> ValoresDeImportacion {
    ValoresDeImportacion {
        proveedor: campos.texto("proveedor"),
        nombre: campos.texto("nombre"),
        fecha: campos.texto("fecha"),
        descuento: campos.texto("descuento"),
        texto: campos.texto("texto"),
    }
}

async fn pagina_de_importacion(
    estado: &EstadoApp,
    actor: &Actor,
    valores: ValoresDeImportacion,
    (previa, aviso): (Option<VistaPreviaDeTarifa>, Option<String>),
) -> Result<Response, ErrorApp> {
    let proveedores = listar_proveedores(estado, actor, false).await?;
    let problemas = aviso.is_some() || previa.as_ref().is_some_and(|p| !p.errores.is_empty());
    let pagina = PaginaImportarTarifa {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        proveedores: opciones_de_proveedor(&proveedores),
        valores,
        previa,
        aviso,
        plantilla: COLUMNAS_DE_TARIFA.join(";"),
    };
    let estado_http = if problemas {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

#[derive(Deserialize)]
pub struct ProveedorElegido {
    proveedor: Option<String>,
}

pub async fn mostrar_importacion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(elegido): Query<ProveedorElegido>,
) -> Result<Response, ErrorApp> {
    let valores = ValoresDeImportacion {
        proveedor: elegido.proveedor.unwrap_or_default(),
        ..ValoresDeImportacion::default()
    };
    pagina_de_importacion(&estado, &actor, valores, (None, None)).await
}

/// Lee la cabecera del paso 1; el nombre por defecto lleva la fecha para distinguir versiones.
fn leer_cabecera(
    estado: &EstadoApp,
    v: &ValoresDeImportacion,
) -> Result<CabeceraDeTarifa, ErrorApp> {
    let proveedor_id = v
        .proveedor
        .trim()
        .parse()
        .map_err(|_| ErrorApp::Validacion("Elige el proveedor de la tarifa.".into()))?;
    let descuento_compra_bp = match v.descuento.trim() {
        "" => None,
        texto => match Porcentaje::desde_texto_es(texto) {
            Ok(p) if (0..=10_000).contains(&p.puntos_basicos()) => Some(p.puntos_basicos()),
            _ => {
                return Err(ErrorApp::Validacion(
                    "El descuento de compra debe ser un porcentaje entre 0 y 100.".into(),
                ));
            }
        },
    };
    let fecha_tarifa = Some(v.fecha.trim().to_owned()).filter(|f| es_fecha_iso(f));
    let nombre = match v.nombre.trim() {
        "" => format!(
            "Importada el {}",
            crate::fecha::formatear_para_pantalla(estado.ahora())
        ),
        nombre => nombre.chars().take(120).collect(),
    };
    Ok(CabeceraDeTarifa {
        proveedor_id,
        nombre,
        fecha_tarifa,
        descuento_compra_bp,
    })
}

fn es_fecha_iso(texto: &str) -> bool {
    let b = texto.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

pub async fn revisar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let valores = valores(&Campos::de(pares));
    let resultado = match leer_cabecera(&estado, &valores) {
        Ok(cabecera) => {
            previsualizar_tarifa(
                &estado,
                &actor,
                cabecera.proveedor_id,
                valores.texto.clone(),
            )
            .await
        }
        Err(error) => Err(error),
    };
    match resultado {
        Ok(previa) => {
            pagina_de_importacion(&estado, &actor, valores, (Some(previa.into()), None)).await
        }
        Err(ErrorApp::Validacion(aviso)) => {
            pagina_de_importacion(&estado, &actor, valores, (None, Some(aviso))).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn confirmar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let (valores, sha) = (valores(&campos), campos.texto("sha256"));
    let cabecera = leer_cabecera(&estado, &valores)?;
    let proveedor_id = cabecera.proveedor_id;
    match importar_tarifa(&estado, &actor, (cabecera, valores.texto.clone(), sha)).await {
        Ok((_, n)) => Ok(Redirect::to(&format!("/catalogo/tarifas?importada={n}")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            let previa =
                previsualizar_tarifa(&estado, &actor, proveedor_id, valores.texto.clone()).await?;
            pagina_de_importacion(&estado, &actor, valores, (Some(previa.into()), Some(aviso)))
                .await
        }
        Err(otro) => Err(otro),
    }
}
