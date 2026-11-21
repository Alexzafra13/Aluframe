use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::catalogo::{EntradaDeArticulo, validar_articulo};
use serde::Deserialize;

use super::catalogo_fichas::SECCION;
use crate::aplicacion::Actor;
use crate::aplicacion::catalogo::*;
use crate::aplicacion::tarifas::historial_de_precios;
use crate::bd::consultas::articulos::FiltroArticulos;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    ErroresDeCampo, FilaDeArticulo, MENU, PaginaArticulos, PaginaFormularioArticulo,
    entrada_de_articulo, opciones_de_proveedor, opciones_de_tipo, opciones_de_unidad, renderizar,
};

#[derive(Deserialize)]
pub struct FiltroDeArticulos {
    proveedor: Option<String>,
    q: Option<String>,
    archivados: Option<String>,
    guardado: Option<String>,
}

fn texto_de_confirmacion(clave: &str) -> Option<&'static str> {
    Some(match clave {
        "precios" => "Precios guardados como una versión nueva de la tarifa.",
        "ajuste" => "Ajuste aplicado como una versión nueva de la tarifa.",
        "sin-cambios" => "No había ningún precio distinto: no se ha creado versión.",
        "articulo" => "Artículo guardado.",
        "si" => "Cambios guardados.",
        _ => return None,
    })
}

pub async fn mostrar_articulos(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(filtro): Query<FiltroDeArticulos>,
) -> Result<Response, ErrorApp> {
    let proveedor_id = filtro.proveedor.as_deref().and_then(|p| p.parse().ok());
    let consulta = FiltroArticulos {
        proveedor_id,
        texto: filtro.q.unwrap_or_default(),
        archivados: filtro.archivados.is_some(),
    };
    let confirmacion = filtro.guardado.as_deref().and_then(texto_de_confirmacion);
    pagina_de_articulos(
        &estado,
        &actor,
        consulta,
        (None, confirmacion.map(str::to_owned)),
    )
    .await
}

/// La lista de artículos; también la usan los cambios de precio para mostrar sus errores.
pub async fn pagina_de_articulos(
    estado: &EstadoApp,
    actor: &Actor,
    consulta: FiltroArticulos,
    (aviso, confirmacion): (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let (proveedor_id, texto, archivados) = (
        consulta.proveedor_id,
        consulta.texto.clone(),
        consulta.archivados,
    );
    let articulos = listar_articulos(estado, actor, consulta).await?;
    let proveedores = listar_proveedores(estado, actor, false).await?;
    let estado_http = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaArticulos {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        articulos: articulos.iter().map(FilaDeArticulo::from).collect(),
        proveedores: opciones_de_proveedor(&proveedores),
        proveedor_elegido: proveedor_id.map(|p| p.to_string()).unwrap_or_default(),
        texto,
        archivados,
        aviso,
        confirmacion,
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

fn entrada_articulo(c: &Campos) -> EntradaDeArticulo {
    EntradaDeArticulo {
        referencia: c.texto("referencia"),
        descripcion: c.texto("descripcion"),
        tipo: c.texto("tipo"),
        unidad: c.texto("unidad"),
        longitud_barra_cm: c.texto("longitud_barra_cm"),
        unidades_por_caja: c.texto("unidades_por_caja"),
        con_acabado: c.marcado("con_acabado"),
        stock_minimo: c.texto("stock_minimo"),
    }
}

struct Formulario {
    id: Option<i64>,
    version: i64,
    proveedor_elegido: String,
    valores: EntradaDeArticulo,
    errores: ErroresDeCampo,
    aviso: Option<String>,
    archivado: bool,
}

async fn pagina_de_formulario(
    estado: &EstadoApp,
    actor: &Actor,
    f: Formulario,
) -> Result<Response, ErrorApp> {
    let proveedores = listar_proveedores(estado, actor, false).await?;
    let historial = match f.id {
        Some(id) => historial_de_precios(estado, actor, id).await?,
        None => Vec::new(),
    };
    let problemas = f.errores.hay() || f.aviso.is_some();
    let pagina = PaginaFormularioArticulo {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        id: f.id,
        version: f.version,
        proveedores: opciones_de_proveedor(&proveedores),
        proveedor_elegido: f.proveedor_elegido,
        tipos: opciones_de_tipo(),
        unidades: opciones_de_unidad(),
        valores: f.valores,
        errores: f.errores,
        aviso: f.aviso,
        archivado: f.archivado,
        historial,
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

pub async fn mostrar_alta(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(elegido): Query<ProveedorElegido>,
) -> Result<Response, ErrorApp> {
    let valores = EntradaDeArticulo {
        tipo: "perfil".into(),
        unidad: "barra".into(),
        longitud_barra_cm: "640".into(),
        ..EntradaDeArticulo::default()
    };
    let formulario = Formulario {
        id: None,
        version: 0,
        proveedor_elegido: elegido.proveedor.unwrap_or_default(),
        valores,
        errores: ErroresDeCampo::default(),
        aviso: None,
        archivado: false,
    };
    pagina_de_formulario(&estado, &actor, formulario).await
}

pub async fn procesar_alta(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let valores = entrada_articulo(&campos);
    let proveedor = campos.entero("proveedor");
    let mut formulario = Formulario {
        id: None,
        version: 0,
        proveedor_elegido: proveedor.map(|p| p.to_string()).unwrap_or_default(),
        valores: valores.clone(),
        errores: ErroresDeCampo::default(),
        aviso: None,
        archivado: false,
    };
    match (validar_articulo(&valores), proveedor) {
        (Err(errores), _) => formulario.errores = ErroresDeCampo(errores),
        (Ok(_), None) => formulario.aviso = Some("Elige un proveedor.".into()),
        (Ok(datos), Some(proveedor)) => {
            match crear_articulo(&estado, &actor, proveedor, datos).await {
                Ok(_) => {
                    return Ok(Redirect::to(&format!(
                        "/catalogo/articulos?proveedor={proveedor}&guardado=articulo"
                    ))
                    .into_response());
                }
                Err(ErrorApp::Validacion(aviso)) => formulario.aviso = Some(aviso),
                Err(otro) => return Err(otro),
            }
        }
    }
    pagina_de_formulario(&estado, &actor, formulario).await
}

pub async fn mostrar_articulo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let a = cargar_articulo(&estado, &actor, id).await?;
    let formulario = Formulario {
        id: Some(id),
        version: a.version,
        proveedor_elegido: a.proveedor_id.to_string(),
        valores: entrada_de_articulo(&a),
        errores: ErroresDeCampo::default(),
        aviso: None,
        archivado: a.archivado,
    };
    pagina_de_formulario(&estado, &actor, formulario).await
}

pub async fn guardar_articulo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let valores = entrada_articulo(&campos);
    let version = campos.entero("version").unwrap_or(0);
    let actual = cargar_articulo(&estado, &actor, id).await?;
    let mut formulario = Formulario {
        id: Some(id),
        version,
        proveedor_elegido: actual.proveedor_id.to_string(),
        valores: valores.clone(),
        errores: ErroresDeCampo::default(),
        aviso: None,
        archivado: actual.archivado,
    };
    match validar_articulo(&valores) {
        Err(errores) => formulario.errores = ErroresDeCampo(errores),
        Ok(datos) => match actualizar_articulo(&estado, &actor, (id, version), datos).await {
            Ok(()) => {
                return Ok(Redirect::to(&format!(
                    "/catalogo/articulos?proveedor={}&guardado=articulo",
                    actual.proveedor_id
                ))
                .into_response());
            }
            Err(e @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
                formulario.aviso = Some(e.to_string())
            }
            Err(otro) => return Err(otro),
        },
    }
    pagina_de_formulario(&estado, &actor, formulario).await
}
