use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{IntoResponse, Redirect, Response};
use dominio::clientes::{EntradaDeDireccion, validar_direccion};
use serde::Deserialize;

use super::clientes::{SECCION, bloque_vacio};
use crate::aplicacion::Actor;
use crate::aplicacion::clientes::{
    actualizar_direccion, anadir_direccion, archivar_cliente, borrar_cliente, cargar_cliente,
    quitar_direccion,
};
use crate::aplicacion::exportar_cliente::exportar_cliente;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    BloqueDireccion, ErroresDeCampo, FilaDePresupuesto, MENU, PaginaEditarDireccion,
    PaginaFichaCliente, direcciones_en_ficha, renderizar,
};

#[derive(Deserialize)]
pub struct AvisoDeFicha {
    aviso: Option<String>,
}

fn texto_de_aviso(clave: &str) -> Option<&'static str> {
    Some(match clave {
        "creado" => "Cliente guardado.",
        "guardado" => "Cambios guardados.",
        "direccion" => "Dirección de montaje guardada.",
        "quitada" => "Dirección de montaje quitada.",
        "archivado" => "Cliente archivado: ya no aparece en la lista.",
        "recuperado" => "Cliente recuperado.",
        _ => return None,
    })
}

async fn pagina_de_ficha(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    (nueva_direccion, confirmacion): (BloqueDireccion, Option<String>),
) -> Result<Response, ErrorApp> {
    let ficha = cargar_cliente(estado, actor, id).await?;
    let presupuestos =
        crate::aplicacion::presupuestos::presupuestos_del_cliente(estado, actor, id).await?;
    let con_errores = nueva_direccion.errores.hay();
    let pagina = PaginaFichaCliente {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        nombre: ficha.cliente.nombre_visible(),
        direcciones: direcciones_en_ficha(&ficha),
        cliente: ficha.cliente,
        nueva_direccion,
        aviso: con_errores.then(|| "Revisa la dirección: no se ha guardado.".to_owned()),
        confirmacion,
        presupuestos: presupuestos.iter().map(FilaDePresupuesto::from).collect(),
    };
    let estado_http = if con_errores {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_ficha(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Query(aviso): Query<AvisoDeFicha>,
) -> Result<Response, ErrorApp> {
    let confirmacion = aviso
        .aviso
        .as_deref()
        .and_then(texto_de_aviso)
        .map(str::to_owned);
    pagina_de_ficha(&estado, &actor, id, (bloque_vacio(0), confirmacion)).await
}

fn a_la_ficha(id: i64, aviso: &str) -> Response {
    Redirect::to(&format!("/clientes/{id}?aviso={aviso}")).into_response()
}

pub async fn archivar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    archivar_cliente(&estado, &actor, id, true).await?;
    Ok(a_la_ficha(id, "archivado"))
}

pub async fn recuperar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    archivar_cliente(&estado, &actor, id, false).await?;
    Ok(a_la_ficha(id, "recuperado"))
}

#[derive(Deserialize)]
pub struct ConfirmacionDeBorrado {
    confirmar: Option<String>,
}

pub async fn borrar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(confirmacion): Form<ConfirmacionDeBorrado>,
) -> Result<Response, ErrorApp> {
    if confirmacion.confirmar.as_deref() != Some("si") {
        return Err(ErrorApp::Validacion(
            "Marca la casilla de confirmación para borrar el cliente.".into(),
        ));
    }
    borrar_cliente(&estado, &actor, id).await?;
    Ok(Redirect::to("/clientes?aviso=borrado").into_response())
}

pub async fn exportar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let csv = exportar_cliente(&estado, &actor, id).await?;
    let disposicion = format!("attachment; filename=\"cliente-{id}.csv\"");
    Ok((
        [
            (CONTENT_TYPE, "text/csv; charset=utf-8".to_owned()),
            (CONTENT_DISPOSITION, disposicion),
        ],
        format!("\u{feff}{csv}"),
    )
        .into_response())
}

fn bloque_leido(
    campos: &Campos,
) -> (
    EntradaDeDireccion,
    Result<Option<dominio::clientes::DatosDeDireccion>, Vec<dominio::clientes::ErrorDeCampo>>,
) {
    let valores = campos.direccion(0);
    let resultado = validar_direccion(&valores);
    (valores, resultado)
}

pub async fn nueva_direccion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let (valores, resultado) = bloque_leido(&Campos::de(pares));
    match resultado {
        Ok(Some(datos)) => {
            anadir_direccion(&estado, &actor, id, datos).await?;
            Ok(a_la_ficha(id, "direccion"))
        }
        Ok(None) => Ok(a_la_ficha(id, "")),
        Err(errores) => {
            let bloque = BloqueDireccion {
                n: 0,
                valores,
                errores: ErroresDeCampo(errores),
            };
            pagina_de_ficha(&estado, &actor, id, (bloque, None)).await
        }
    }
}

pub async fn quitar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, direccion)): Path<(i64, i64)>,
) -> Result<Response, ErrorApp> {
    quitar_direccion(&estado, &actor, (id, direccion)).await?;
    Ok(a_la_ficha(id, "quitada"))
}

async fn pagina_de_direccion(
    estado: &EstadoApp,
    actor: &Actor,
    (id, direccion_id): (i64, i64),
    bloque: Option<BloqueDireccion>,
) -> Result<Response, ErrorApp> {
    let ficha = cargar_cliente(estado, actor, id).await?;
    let guardada = ficha
        .direcciones
        .iter()
        .find(|d| d.id == direccion_id)
        .ok_or(ErrorApp::NoEncontrado)?;
    let texto = |v: &Option<String>| v.clone().unwrap_or_default();
    let d = &guardada.datos;
    let con_errores = bloque.is_some();
    let direccion = bloque.unwrap_or_else(|| BloqueDireccion {
        n: 0,
        valores: EntradaDeDireccion {
            etiqueta: texto(&d.etiqueta),
            calle: texto(&d.calle),
            cp: texto(&d.cp),
            localidad: texto(&d.localidad),
            provincia: texto(&d.provincia),
            contacto_nombre: texto(&d.contacto_nombre),
            contacto_telefono: texto(&d.contacto_telefono),
            notas: texto(&d.notas),
        },
        errores: ErroresDeCampo::default(),
    });
    let pagina = PaginaEditarDireccion {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        cliente_id: id,
        direccion_id,
        nombre_cliente: ficha.cliente.nombre_visible(),
        direccion,
    };
    let estado_http = if con_errores {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_direccion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(ids): Path<(i64, i64)>,
) -> Result<Response, ErrorApp> {
    pagina_de_direccion(&estado, &actor, ids, None).await
}

pub async fn guardar_direccion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(ids): Path<(i64, i64)>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let (valores, resultado) = bloque_leido(&Campos::de(pares));
    match resultado {
        Ok(Some(datos)) => {
            actualizar_direccion(&estado, &actor, ids, datos).await?;
            Ok(a_la_ficha(ids.0, "direccion"))
        }
        Ok(None) => {
            quitar_direccion(&estado, &actor, ids).await?;
            Ok(a_la_ficha(ids.0, "quitada"))
        }
        Err(errores) => {
            let bloque = BloqueDireccion {
                n: 0,
                valores,
                errores: ErroresDeCampo(errores),
            };
            pagina_de_direccion(&estado, &actor, ids, Some(bloque)).await
        }
    }
}
