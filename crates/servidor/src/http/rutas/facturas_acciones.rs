//! Lo que cambia una factura (borrador o cobros) y lo que se hace desde el trabajo: factura,
//! señal y albarán.

use axum::Form;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use dominio::stock::leer_cantidad_mil;
use dominio::unidades::{Porcentaje, PrecioUnitario};

use super::facturas::pagina_de_factura;
use super::trabajos::pagina_del_trabajo;
use crate::aplicacion::Actor;
use crate::aplicacion::facturas::albaranes::crear_albaran;
use crate::aplicacion::facturas::cobros::{
    DatosDeCobro, DestinoDelCobro, anular_cobro, registrar_cobro,
};
use crate::aplicacion::facturas::crear::{factura_de_trabajo, factura_libre, rectificativa};
use crate::aplicacion::facturas::editar::{
    DatosDeFactura, LineaNueva, anadir_linea, descartar_borrador, guardar_datos, quitar_linea,
};
use crate::aplicacion::facturas::emitir::emitir_factura;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;

type Pares = Form<Vec<(String, String)>>;

/// Vuelve a la factura con su confirmación, o la enseña con el aviso.
async fn tras(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    (resultado, clave): (Result<(), ErrorApp>, &str),
) -> Result<Response, ErrorApp> {
    match resultado {
        Ok(()) => Ok(Redirect::to(&format!("/facturas/{id}?aviso={clave}")).into_response()),
        Err(error @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
            pagina_de_factura(estado, actor, id, (Some(error.to_string()), None)).await
        }
        Err(otro) => Err(otro),
    }
}

fn opcional(texto: String) -> Option<String> {
    Some(texto.trim().to_owned()).filter(|t| !t.is_empty())
}

fn porcentaje(texto: &str) -> Result<Porcentaje, ErrorApp> {
    if texto.trim().is_empty() {
        return Ok(Porcentaje::CERO);
    }
    Porcentaje::desde_texto_es(texto.trim())
        .map_err(|_| ErrorApp::Validacion("Revisa el descuento (%).".into()))
}

fn leer_linea(c: &Campos) -> Result<LineaNueva, ErrorApp> {
    let invalido = |q: &str| ErrorApp::Validacion(format!("Revisa {q}."));
    Ok(LineaNueva {
        descripcion: c.texto("descripcion"),
        cantidad_mil: leer_cantidad_mil(&c.texto("cantidad"))
            .map_err(|_| invalido("la cantidad"))?,
        precio: PrecioUnitario::desde_texto_es(c.texto("precio").trim())
            .map_err(|_| invalido("el precio"))?,
        descuento: porcentaje(&c.texto("descuento"))?,
        iva: Porcentaje::desde_puntos_basicos(c.entero("iva").unwrap_or(2_100)),
    })
}

pub async fn nueva_libre(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    let cliente = Campos::de(p)
        .entero("cliente")
        .ok_or(ErrorApp::NoEncontrado)?;
    let id = factura_libre(&estado, &actor, cliente).await?;
    Ok(Redirect::to(&format!("/facturas/{id}?aviso=creada")).into_response())
}

pub async fn anadir(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    let resultado = match leer_linea(&Campos::de(p)) {
        Ok(linea) => anadir_linea(&estado, &actor, id, linea).await,
        Err(error) => Err(error),
    };
    tras(&estado, &actor, id, (resultado, "guardada")).await
}

pub async fn quitar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, linea)): Path<(i64, i64)>,
) -> Result<Response, ErrorApp> {
    let resultado = quitar_linea(&estado, &actor, id, linea).await;
    tras(&estado, &actor, id, (resultado, "guardada")).await
}

pub async fn datos(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    let c = Campos::de(p);
    let resultado = match porcentaje(&c.texto("descuento")) {
        Ok(descuento) => {
            let d = DatosDeFactura {
                fecha_operacion: opcional(c.texto("fecha_operacion")),
                vencimiento: opcional(c.texto("vencimiento")),
                notas: opcional(c.texto("notas")),
                descuento,
            };
            guardar_datos(&estado, &actor, (id, c.entero("version").unwrap_or(0)), d).await
        }
        Err(error) => Err(error),
    };
    tras(&estado, &actor, id, (resultado, "guardada")).await
}

pub async fn emitir(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    let confirmado = Campos::de(p).texto("confirmar") == "si";
    let resultado = emitir_factura(&estado, &actor, id, confirmado)
        .await
        .map(|_| ());
    tras(&estado, &actor, id, (resultado, "emitida")).await
}

pub async fn descartar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    descartar_borrador(&estado, &actor, id).await?;
    Ok(Redirect::to("/facturas").into_response())
}

fn datos_de_cobro(c: &Campos) -> DatosDeCobro {
    DatosDeCobro {
        fecha: c.texto("fecha"),
        importe: c.texto("importe"),
        medio: c.texto("medio"),
        nota: c.texto("nota"),
    }
}

pub async fn cobrar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    let resultado = registrar_cobro(
        &estado,
        &actor,
        DestinoDelCobro::Factura(id),
        datos_de_cobro(&Campos::de(p)),
    )
    .await;
    tras(&estado, &actor, id, (resultado, "cobro")).await
}

pub async fn rectificar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    match rectificativa(&estado, &actor, id, Campos::de(p).texto("motivo")).await {
        Ok(nueva) => Ok(Redirect::to(&format!("/facturas/{nueva}?aviso=creada")).into_response()),
        Err(error @ ErrorApp::Validacion(_)) => {
            pagina_de_factura(&estado, &actor, id, (Some(error.to_string()), None)).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn anular(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(cobro): Path<i64>,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    let destino = match anular_cobro(&estado, &actor, cobro, Campos::de(p).texto("motivo")).await? {
        (_, Some(factura)) => format!("/facturas/{factura}?aviso=anulado"),
        (Some(trabajo), None) => format!("/trabajos/{trabajo}?aviso=anulado"),
        (None, None) => "/facturas".to_owned(),
    };
    Ok(Redirect::to(&destino).into_response())
}

pub async fn facturar_trabajo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(trabajo): Path<i64>,
) -> Result<Response, ErrorApp> {
    let id = factura_de_trabajo(&estado, &actor, trabajo).await?;
    Ok(Redirect::to(&format!("/facturas/{id}")).into_response())
}

pub async fn senal(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(trabajo): Path<i64>,
    Form(p): Pares,
) -> Result<Response, ErrorApp> {
    match registrar_cobro(
        &estado,
        &actor,
        DestinoDelCobro::Trabajo(trabajo),
        datos_de_cobro(&Campos::de(p)),
    )
    .await
    {
        Ok(()) => Ok(Redirect::to(&format!("/trabajos/{trabajo}?aviso=senal")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            pagina_del_trabajo(&estado, &actor, trabajo, (Some(aviso), None)).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn albaran(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(trabajo): Path<i64>,
) -> Result<Response, ErrorApp> {
    let id = crear_albaran(&estado, &actor, trabajo).await?;
    Ok(Redirect::to(&format!("/albaranes/{id}/imprimir")).into_response())
}
