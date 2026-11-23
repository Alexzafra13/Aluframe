//! Pantallas del operario: «Mi día» y cada ventana con sus pasos (PANTALLAS 13). Sin precios.

use axum::Form;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::Accion;
use dominio::taller::AccionDePaso;

use crate::aplicacion::Actor;
use crate::aplicacion::apuntes::mis_tareas;
use crate::aplicacion::trabajos::acciones::{anotar_falta, cambiar_paso, repetir_pieza};
use crate::aplicacion::trabajos::consultar::{ventanas_de_operario, ventanas_del_resto_del_taller};
use crate::aplicacion::trabajos::operario::abrir_ventana;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    HojaDeVentana, PaginaMiDia, PaginaVentanaOperario, TarjetaApunte, TarjetaVentana,
    VentanaEnPantalla, renderizar,
};

pub async fn mostrar_mi_dia(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    let ventanas = ventanas_de_operario(&estado, &actor).await?;
    let resto = if actor.puede(Accion::VerTodosLosTrabajos) && !actor.rol.es_propietario() {
        ventanas_del_resto_del_taller(&estado, &actor).await?
    } else {
        Vec::new()
    };
    let tarjetas = |lista: &[(_, _)]| {
        lista
            .iter()
            .map(|(v, t)| TarjetaVentana::de(v, t))
            .collect()
    };
    let tareas = mis_tareas(&estado, &actor)
        .await?
        .into_iter()
        .map(|f| TarjetaApunte::de(f, &actor))
        .collect();
    let pagina = PaginaMiDia {
        usuario: (&actor).into(),
        tarjetas: tarjetas(&ventanas),
        resto: tarjetas(&resto),
        tareas,
    };
    Ok(renderizar(&pagina)?.into_response())
}

async fn pagina_de_ventana(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    aviso: Option<String>,
) -> Result<Response, ErrorApp> {
    let abierta = abrir_ventana(estado, actor, id).await?;
    let v = &abierta.ventana;
    let codigo = if aviso.is_some() {
        StatusCode::CONFLICT
    } else {
        StatusCode::OK
    };
    let pagina = PaginaVentanaOperario {
        usuario: actor.into(),
        codigo: abierta.trabajo.codigo(),
        cliente: abierta.trabajo.cliente_nombre.clone(),
        ventana: VentanaEnPantalla::from(v),
        hoja: HojaDeVentana::desde(&v.fila.descripcion, &v.congelado),
        anterior: abierta.anterior,
        siguiente: abierta.siguiente,
        aviso,
    };
    Ok((codigo, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_ventana(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    pagina_de_ventana(&estado, &actor, id, None).await
}

/// Si otra persona tocó el paso antes, se enseña la ventana actualizada con el aviso.
pub async fn cambiar_el_paso(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(paso): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let accion = AccionDePaso::desde_texto(&campos.texto("accion"))
        .ok_or_else(|| ErrorApp::Validacion("Acción desconocida.".into()))?;
    let version = campos.entero("version").unwrap_or(0);
    let ventana = campos.entero("ventana");
    match cambiar_paso(&estado, &actor, (paso, version), accion).await {
        Ok(ventana) => Ok(Redirect::to(&format!("/mi-dia/{ventana}")).into_response()),
        Err(error @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
            let Some(ventana) = ventana else {
                return Err(error);
            };
            let texto = "Otra persona ha cambiado este paso: revisa cómo está ahora.".to_owned();
            pagina_de_ventana(&estado, &actor, ventana, Some(texto)).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn avisar_falta(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let elegida = campos.texto("nota");
    let nota = if elegida.trim().is_empty() {
        campos.texto("otra")
    } else {
        elegida
    };
    if nota.trim().is_empty() {
        let texto = "Elige un motivo o escribe qué falta.".to_owned();
        return pagina_de_ventana(&estado, &actor, id, Some(texto)).await;
    }
    anotar_falta(&estado, &actor, id, Some(nota)).await?;
    Ok(Redirect::to(&format!("/mi-dia/{id}")).into_response())
}

pub async fn repetir(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let datos = (
        id,
        campos.entero("numero").unwrap_or(0),
        campos.entero("cantidad").unwrap_or(0),
    );
    let resultado = repetir_pieza(&estado, &actor, datos, campos.texto("motivo")).await;
    let trabajo = campos.texto("volver") == "trabajo";
    match resultado {
        Ok(()) if trabajo => {
            let destino = abrir_ventana(&estado, &actor, id).await?.trabajo.id;
            Ok(Redirect::to(&format!("/trabajos/{destino}?aviso=repetida")).into_response())
        }
        Ok(()) => Ok(Redirect::to(&format!("/mi-dia/{id}")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            pagina_de_ventana(&estado, &actor, id, Some(aviso)).await
        }
        Err(otro) => Err(otro),
    }
}
