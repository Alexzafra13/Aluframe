//! Proveedores y acabados: lista con alta en la misma pantalla y página de edición.

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::catalogo::{EntradaDeAcabado, EntradaDeProveedor, validar_acabado, validar_proveedor};
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::catalogo::*;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    ErroresDeCampo, MENU, PaginaAcabado, PaginaAcabados, PaginaProveedor, PaginaProveedores,
    entrada_de_proveedor, porcentaje_en_campo, renderizar,
};

pub const SECCION: &str = "catalogo";

#[derive(Deserialize)]
pub struct Guardado {
    guardado: Option<String>,
}

fn con_estado(problemas: bool, html: axum::response::Html<String>) -> Response {
    let estado = if problemas {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (estado, html).into_response()
}

fn entrada_proveedor(c: &Campos) -> EntradaDeProveedor {
    EntradaDeProveedor {
        nombre: c.texto("nombre"),
        nif: c.texto("nif"),
        contacto: c.texto("contacto"),
        condiciones_pago: c.texto("condiciones_pago"),
        descuento_habitual: c.texto("descuento_habitual"),
        notas: c.texto("notas"),
    }
}

async fn lista_de_proveedores(
    estado: &EstadoApp,
    actor: &Actor,
    (valores, errores, aviso): (EntradaDeProveedor, ErroresDeCampo, Option<String>),
    confirmacion: Option<String>,
) -> Result<Response, ErrorApp> {
    let problemas = errores.hay() || aviso.is_some();
    let pagina = PaginaProveedores {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        proveedores: listar_proveedores(estado, actor, true).await?,
        valores,
        errores,
        aviso,
        confirmacion,
    };
    Ok(con_estado(problemas, renderizar(&pagina)?))
}

pub async fn mostrar_proveedores(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(guardado): Query<Guardado>,
) -> Result<Response, ErrorApp> {
    let confirmacion = guardado.guardado.map(|_| "Proveedor guardado.".to_owned());
    let vacio = (
        EntradaDeProveedor::default(),
        ErroresDeCampo::default(),
        None,
    );
    lista_de_proveedores(&estado, &actor, vacio, confirmacion).await
}

pub async fn alta_de_proveedor(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let valores = entrada_proveedor(&Campos::de(pares));
    let resultado = match validar_proveedor(&valores) {
        Err(errores) => Err((ErroresDeCampo(errores), None)),
        Ok(datos) => match crear_proveedor(&estado, &actor, datos).await {
            Ok(_) => Ok(()),
            Err(ErrorApp::Validacion(aviso)) => Err((ErroresDeCampo::default(), Some(aviso))),
            Err(otro) => return Err(otro),
        },
    };
    match resultado {
        Ok(()) => Ok(Redirect::to("/catalogo/proveedores?guardado=si").into_response()),
        Err((errores, aviso)) => {
            lista_de_proveedores(&estado, &actor, (valores, errores, aviso), None).await
        }
    }
}

pub async fn mostrar_proveedor(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let p = cargar_proveedor(&estado, &actor, id).await?;
    let pagina = PaginaProveedor {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        id,
        version: p.version,
        archivado: p.archivado,
        valores: entrada_de_proveedor(&p),
        errores: ErroresDeCampo::default(),
        aviso: None,
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn guardar_proveedor(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let (valores, version) = (
        entrada_proveedor(&campos),
        campos.entero("version").unwrap_or(0),
    );
    let (errores, aviso) = match validar_proveedor(&valores) {
        Err(errores) => (ErroresDeCampo(errores), None),
        Ok(datos) => match actualizar_proveedor(&estado, &actor, (id, version), datos).await {
            Ok(()) => return Ok(Redirect::to("/catalogo/proveedores?guardado=si").into_response()),
            Err(e @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
                (ErroresDeCampo::default(), Some(e.to_string()))
            }
            Err(otro) => return Err(otro),
        },
    };
    let archivado = cargar_proveedor(&estado, &actor, id).await?.archivado;
    let pagina = PaginaProveedor {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        id,
        version,
        archivado,
        valores,
        errores,
        aviso,
    };
    Ok(con_estado(true, renderizar(&pagina)?))
}

fn entrada_acabado(c: &Campos) -> EntradaDeAcabado {
    EntradaDeAcabado {
        nombre: c.texto("nombre"),
        es_base: c.marcado("es_base"),
        recargo: c.texto("recargo"),
    }
}

async fn lista_de_acabados(
    estado: &EstadoApp,
    actor: &Actor,
    (valores, errores, aviso): (EntradaDeAcabado, ErroresDeCampo, Option<String>),
    confirmacion: Option<String>,
) -> Result<Response, ErrorApp> {
    let problemas = errores.hay() || aviso.is_some();
    let pagina = PaginaAcabados {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        acabados: listar_acabados(estado, actor).await?,
        valores,
        errores,
        aviso,
        confirmacion,
    };
    Ok(con_estado(problemas, renderizar(&pagina)?))
}

pub async fn mostrar_acabados(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(guardado): Query<Guardado>,
) -> Result<Response, ErrorApp> {
    let confirmacion = guardado.guardado.map(|_| "Acabado guardado.".to_owned());
    let vacio = (EntradaDeAcabado::default(), ErroresDeCampo::default(), None);
    lista_de_acabados(&estado, &actor, vacio, confirmacion).await
}

pub async fn alta_de_acabado(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let valores = entrada_acabado(&Campos::de(pares));
    let (errores, aviso) = match validar_acabado(&valores) {
        Err(errores) => (ErroresDeCampo(errores), None),
        Ok(datos) => match crear_acabado(&estado, &actor, datos).await {
            Ok(_) => return Ok(Redirect::to("/catalogo/acabados?guardado=si").into_response()),
            Err(ErrorApp::Validacion(aviso)) => (ErroresDeCampo::default(), Some(aviso)),
            Err(otro) => return Err(otro),
        },
    };
    lista_de_acabados(&estado, &actor, (valores, errores, aviso), None).await
}

pub async fn mostrar_acabado(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let a = cargar_acabado(&estado, &actor, id).await?;
    let valores = EntradaDeAcabado {
        nombre: a.nombre,
        es_base: a.es_base,
        recargo: porcentaje_en_campo(a.recargo_bp),
    };
    let pagina = PaginaAcabado {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        id,
        version: a.version,
        archivado: a.archivado,
        valores,
        errores: ErroresDeCampo::default(),
        aviso: None,
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn guardar_acabado(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let (valores, version) = (
        entrada_acabado(&campos),
        campos.entero("version").unwrap_or(0),
    );
    let (errores, aviso) = match validar_acabado(&valores) {
        Err(errores) => (ErroresDeCampo(errores), None),
        Ok(datos) => match actualizar_acabado(&estado, &actor, (id, version), datos).await {
            Ok(()) => return Ok(Redirect::to("/catalogo/acabados?guardado=si").into_response()),
            Err(e @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
                (ErroresDeCampo::default(), Some(e.to_string()))
            }
            Err(otro) => return Err(otro),
        },
    };
    let archivado = cargar_acabado(&estado, &actor, id).await?.archivado;
    let pagina = PaginaAcabado {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        id,
        version,
        archivado,
        valores,
        errores,
        aviso,
    };
    Ok(con_estado(true, renderizar(&pagina)?))
}

/// Archivar o recuperar cualquier ficha del catálogo y volver a su lista.
pub async fn cambiar_archivo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((tipo, id, accion)): Path<(String, i64, String)>,
) -> Result<Response, ErrorApp> {
    let entidad = match tipo.as_str() {
        "proveedores" => "proveedor",
        "acabados" => "acabado",
        "articulos" => "articulo",
        _ => return Err(ErrorApp::NoEncontrado),
    };
    let archivado = match accion.as_str() {
        "archivar" => true,
        "recuperar" => false,
        _ => return Err(ErrorApp::NoEncontrado),
    };
    archivar(&estado, &actor, (entidad, id), archivado).await?;
    Ok(Redirect::to(&format!("/catalogo/{tipo}?guardado=si")).into_response())
}
