use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::unidades::Porcentaje;
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::aceptar_presupuesto::motivos_para_no_aceptar;
use crate::aplicacion::ajustes_presupuesto::leer_ajustes_de_presupuesto;
use crate::aplicacion::clientes::{ConsultaDeClientes, listar_clientes};
use crate::aplicacion::lineas_de_presupuesto::{ivas_de, totales_de};
use crate::aplicacion::presupuestos::{cargar_presupuesto, crear_presupuesto, listar_presupuestos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::vistas::{
    FilaDePresupuesto, MENU, Opcion, PaginaPresupuesto, PaginaPresupuestos, TarjetaDeLinea,
    desglosar, nombre_de_estado, porcentaje_en_campo, porcentaje_texto, renderizar,
};

pub const SECCION: &str = "presupuestos";

#[derive(Deserialize)]
pub struct Filtro {
    estado: Option<String>,
}

pub async fn mostrar_lista(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(filtro): Query<Filtro>,
) -> Result<Response, ErrorApp> {
    let elegido = filtro
        .estado
        .filter(|e| ["borrador", "enviado", "aceptado", "rechazado"].contains(&e.as_str()));
    let filas = listar_presupuestos(&estado, &actor, elegido.clone()).await?;
    let consulta = ConsultaDeClientes {
        texto: String::new(),
        archivados: false,
        pagina: 1,
    };
    let clientes = listar_clientes(
        &estado,
        &actor,
        ConsultaDeClientes {
            pagina: 1,
            ..consulta
        },
    )
    .await?;
    let pagina = PaginaPresupuestos {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        filas: filas.iter().map(FilaDePresupuesto::from).collect(),
        filtro: elegido.unwrap_or_default(),
        clientes: clientes
            .clientes
            .iter()
            .map(|c| Opcion {
                valor: c.id.to_string(),
                texto: c.nombre_visible(),
            })
            .collect(),
    };
    Ok(renderizar(&pagina)?.into_response())
}

#[derive(Deserialize)]
pub struct Nuevo {
    cliente: i64,
    direccion: Option<String>,
}

pub async fn crear(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(nuevo): Form<Nuevo>,
) -> Result<Response, ErrorApp> {
    let direccion = nuevo.direccion.and_then(|d| d.parse().ok());
    let id = crear_presupuesto(&estado, &actor, nuevo.cliente, direccion).await?;
    Ok(Redirect::to(&format!("/presupuestos/{id}/lineas/nueva")).into_response())
}

#[derive(Deserialize)]
pub struct Aviso {
    aviso: Option<String>,
}

fn texto_de_aviso(clave: &str) -> Option<&'static str> {
    Some(match clave {
        "linea" => "Guardado en el presupuesto.",
        "quitada" => "Línea quitada.",
        "datos" => "Datos guardados.",
        "enviado" => "Marcado como enviado.",
        "rechazado" => "Presupuesto rechazado.",
        "version" => "Versión nueva creada en borrador.",
        "aceptado" => "Presupuesto aceptado y congelado.",
        "recalculado" => "Precios recalculados con la tarifa vigente.",
        _ => return None,
    })
}

pub async fn mostrar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Query(aviso): Query<Aviso>,
) -> Result<Response, ErrorApp> {
    let confirmacion = aviso
        .aviso
        .as_deref()
        .and_then(texto_de_aviso)
        .map(str::to_owned);
    pagina(&estado, &actor, id, (None, confirmacion)).await
}

/// La ficha del presupuesto; también se usa para mostrar un error sin perder el contexto.
pub async fn pagina(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    (aviso, confirmacion): (Option<String>, Option<String>),
) -> Result<Response, ErrorApp> {
    let ficha = cargar_presupuesto(estado, actor, id).await?;
    let ajustes = leer_ajustes_de_presupuesto(estado).await?;
    let p = &ficha.presupuesto;
    let ivas = ivas_de(&totales_de(&ficha.lineas, p.descuento_bp));
    let estado_http = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaPresupuesto {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        codigo: p.codigo(),
        cliente: ficha.cliente.nombre_visible(),
        nombre_estado: nombre_de_estado(&p.estado),
        lineas: ficha.lineas.iter().map(TarjetaDeLinea::from).collect(),
        desglose: desglosar(&ficha.lineas),
        ivas,
        motivos: motivos_para_no_aceptar(&ficha),
        direcciones: ficha
            .direcciones
            .iter()
            .map(|d| Opcion {
                valor: d.id.to_string(),
                texto: d.datos.resumen(),
            })
            .collect(),
        opciones_iva: opciones_de_iva(ajustes.iva_general, ajustes.iva_reducido),
        descuento: porcentaje_en_campo(p.descuento_bp),
        ficha,
        aviso,
        confirmacion,
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

pub fn opciones_de_iva(general: Porcentaje, reducido: Porcentaje) -> Vec<Opcion> {
    let mut opciones = vec![Opcion {
        valor: general.puntos_basicos().to_string(),
        texto: format!("General {}", porcentaje_texto(general.puntos_basicos())),
    }];
    if reducido != general {
        opciones.push(Opcion {
            valor: reducido.puntos_basicos().to_string(),
            texto: format!("Reducido {}", porcentaje_texto(reducido.puntos_basicos())),
        });
    }
    opciones
}

pub fn a_la_ficha(id: i64, aviso: &str) -> Response {
    Redirect::to(&format!("/presupuestos/{id}?aviso={aviso}")).into_response()
}
