//! Rutas del stock: saldos, entradas, ajustes, mínimos y lista de la compra (PANTALLAS 9).

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::stock::{TipoDeMovimiento, formatear_cantidad};
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::catalogo::{cargar_articulo, listar_acabados, listar_articulos};
use crate::aplicacion::stock::compra::lista_de_compra;
use crate::aplicacion::stock::{
    MovimientoManual, cambiar_minimo, movimientos_de, registrar, saldos,
};
use crate::bd::consultas::articulos::FiltroArticulos;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    MENU, MovimientoEnPantalla, Opcion, PaginaArticuloStock, PaginaCompra, PaginaStock,
    filas_de_stock, renderizar,
};

#[derive(Deserialize)]
pub struct FiltroStock {
    bajo: Option<String>,
    aviso: Option<String>,
}

fn texto_de_confirmacion(clave: &str) -> Option<String> {
    let texto = match clave {
        "entrada" => "Entrada anotada.",
        "ajuste" => "Ajuste anotado.",
        "minimo" => "Mínimo guardado.",
        _ => return None,
    };
    Some(texto.to_owned())
}

async fn pagina_de_stock(
    estado: &EstadoApp,
    actor: &Actor,
    solo_bajo: bool,
    (aviso, confirmacion): (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let mut filas = filas_de_stock(&saldos(estado, actor).await?);
    if solo_bajo {
        filas.retain(|f| f.bajo_minimo);
    }
    let articulos = listar_articulos(estado, actor, FiltroArticulos::default()).await?;
    let acabados = listar_acabados(estado, actor).await?;
    let codigo = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaStock {
        usuario: actor.into(),
        menu: MENU,
        seccion: "stock",
        filas,
        solo_bajo,
        articulos: articulos
            .into_iter()
            .map(|a| Opcion {
                valor: a.id.to_string(),
                texto: format!("{} · {}", a.referencia, a.descripcion),
            })
            .collect(),
        acabados: acabados
            .into_iter()
            .filter(|a| !a.archivado)
            .map(|a| Opcion {
                valor: a.id.to_string(),
                texto: a.nombre,
            })
            .collect(),
        aviso,
        confirmacion,
    };
    Ok((codigo, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_stock(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(filtro): Query<FiltroStock>,
) -> Result<Response, ErrorApp> {
    let confirmacion = filtro.aviso.as_deref().and_then(texto_de_confirmacion);
    pagina_de_stock(&estado, &actor, filtro.bajo.is_some(), (None, confirmacion)).await
}

fn leer_movimiento(pares: Vec<(String, String)>) -> MovimientoManual {
    let campos = Campos::de(pares);
    MovimientoManual {
        articulo: campos.entero("articulo").unwrap_or(0),
        acabado: campos.entero("acabado"),
        largo_cm: campos.texto("largo"),
        cantidad: campos.texto("cantidad"),
        motivo: campos.texto("motivo"),
    }
}

async fn anotar(
    estado: &EstadoApp,
    actor: &Actor,
    tipo: TipoDeMovimiento,
    pares: Vec<(String, String)>,
) -> Result<Response, ErrorApp> {
    let clave = if tipo == TipoDeMovimiento::Entrada {
        "entrada"
    } else {
        "ajuste"
    };
    match registrar(estado, actor, tipo, leer_movimiento(pares)).await {
        Ok(()) => Ok(Redirect::to(&format!("/stock?aviso={clave}")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            pagina_de_stock(estado, actor, false, (Some(aviso), None)).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn anotar_entrada(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    anotar(&estado, &actor, TipoDeMovimiento::Entrada, pares).await
}

pub async fn anotar_ajuste(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    anotar(&estado, &actor, TipoDeMovimiento::Ajuste, pares).await
}

pub async fn mostrar_articulo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Query(filtro): Query<FiltroStock>,
) -> Result<Response, ErrorApp> {
    let articulo = cargar_articulo(&estado, &actor, id).await?;
    let filas = filas_de_stock(&saldos(&estado, &actor).await?)
        .into_iter()
        .filter(|f| f.articulo == id)
        .collect();
    let movimientos = movimientos_de(&estado, &actor, id).await?;
    let pagina = PaginaArticuloStock {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "stock",
        id,
        referencia: articulo.referencia,
        descripcion: articulo.descripcion,
        unidad: articulo.unidad,
        minimo: formatear_cantidad(articulo.stock_minimo_mil),
        filas,
        movimientos: movimientos.iter().map(MovimientoEnPantalla::from).collect(),
        confirmacion: filtro.aviso.as_deref().and_then(texto_de_confirmacion),
    };
    Ok(renderizar(&pagina)?.into_response())
}

pub async fn guardar_minimo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    cambiar_minimo(&estado, &actor, id, Campos::de(pares).texto("minimo")).await?;
    Ok(Redirect::to(&format!("/stock/{id}?aviso=minimo")).into_response())
}

pub async fn mostrar_compra(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    let lineas = lista_de_compra(&estado, &actor).await?;
    let pagina = PaginaCompra {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "stock",
        lineas: lineas.iter().map(Into::into).collect(),
    };
    Ok(renderizar(&pagina)?.into_response())
}
