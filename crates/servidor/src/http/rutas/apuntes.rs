//! Rutas de apuntes (ADR-0025). Las reglas de quién puede qué están en `aplicacion::apuntes`.

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::{Accion, Rol};
use dominio::apuntes::{EntradaDeApunte, EstadoApunte, validar_apunte};
use serde::Deserialize;

use crate::aplicacion::Actor;
use crate::aplicacion::apuntes::{
    PeticionDeApunte, cambiar_apunte, crear_apunte, listar_apuntes, pasar_a_presupuesto,
};
use crate::aplicacion::clientes::cargar_cliente;
use crate::aplicacion::usuarios::listar_usuarios;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{MENU, Opcion, PaginaApuntes, PaginaNuevoApunte, TarjetaApunte, renderizar};

const SECCION: &str = "apuntes";

#[derive(Deserialize)]
pub struct FiltroApuntes {
    cerrados: Option<String>,
    hecho: Option<String>,
}

pub async fn mostrar_apuntes(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(filtro): Query<FiltroApuntes>,
) -> Result<Response, ErrorApp> {
    let abiertos = filtro.cerrados.is_none();
    let filas = listar_apuntes(&estado, &actor, abiertos, None).await?;
    let pagina = PaginaApuntes {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        abiertos,
        apuntes: filas
            .into_iter()
            .map(|f| TarjetaApunte::de(f, &actor))
            .collect(),
        aviso: None,
        confirmacion: filtro.hecho.as_deref().and_then(texto_de_confirmacion),
    };
    Ok(renderizar(&pagina)?.into_response())
}

fn texto_de_confirmacion(clave: &str) -> Option<String> {
    Some(
        match clave {
            "guardado" => "Apunte guardado.",
            "hecho" => "Marcado como hecho.",
            "reabierto" => "Vuelve a estar pendiente.",
            _ => return None,
        }
        .to_owned(),
    )
}

#[derive(Deserialize)]
pub struct DesdeCliente {
    cliente: Option<i64>,
}

pub async fn mostrar_nuevo(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Query(desde): Query<DesdeCliente>,
) -> Result<Response, ErrorApp> {
    formulario(
        &estado,
        &actor,
        desde.cliente,
        (EntradaDeApunte::default(), None),
    )
    .await
}

async fn formulario(
    estado: &EstadoApp,
    actor: &Actor,
    cliente: Option<i64>,
    (valores, aviso): (EntradaDeApunte, Option<String>),
) -> Result<Response, ErrorApp> {
    crate::aplicacion::autorizar(actor, Accion::TomarApuntes)?;
    let (cliente, direcciones) = match cliente {
        Some(id) => {
            let ficha = cargar_cliente(estado, actor, id).await?;
            let opciones = ficha
                .direcciones
                .iter()
                .map(|d| Opcion {
                    valor: d.id.to_string(),
                    texto: d.datos.resumen(),
                })
                .collect();
            (Some((id, ficha.cliente.nombre_visible())), opciones)
        }
        None => (None, Vec::new()),
    };
    let codigo = if aviso.is_some() {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    let pagina = PaginaNuevoApunte {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        cliente,
        direcciones,
        operarios: operarios_para_tareas(estado, actor).await?,
        valores,
        aviso,
    };
    Ok((codigo, renderizar(&pagina)?).into_response())
}

/// Solo el dueño manda tareas: a los operarios activos.
async fn operarios_para_tareas(estado: &EstadoApp, actor: &Actor) -> Result<Vec<Opcion>, ErrorApp> {
    if !actor.puede(Accion::GestionarApuntes) {
        return Ok(Vec::new());
    }
    Ok(listar_usuarios(estado, actor)
        .await?
        .into_iter()
        .filter(|u| u.rol == Rol::Operario && u.activo)
        .map(|u| Opcion {
            valor: u.id.to_string(),
            texto: u.nombre,
        })
        .collect())
}

pub async fn crear(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let entrada = EntradaDeApunte {
        texto: campos.texto("texto"),
        medidas: campos.texto("medidas"),
        contacto: campos.texto("contacto"),
    };
    let cliente = campos.entero("cliente");
    let resultado = match validar_apunte(&entrada) {
        Ok(datos) => {
            let peticion = PeticionDeApunte {
                cliente,
                direccion: campos.entero("direccion"),
                para: campos.entero("para"),
                datos,
            };
            crear_apunte(&estado, &actor, peticion).await
        }
        Err(error) => Err(error.into()),
    };
    match resultado {
        Ok(_) => Ok(Redirect::to("/apuntes?hecho=guardado").into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            formulario(&estado, &actor, cliente, (entrada, Some(aviso))).await
        }
        Err(otro) => Err(otro),
    }
}

/// Solo vuelve a pantallas conocidas: el valor llega de un formulario.
fn destino(campos: &Campos) -> &'static str {
    match campos.texto("volver").as_str() {
        "/mi-dia" => "/mi-dia",
        _ => "/apuntes",
    }
}

pub async fn marcar_hecho(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    cambiar(
        &estado,
        &actor,
        (id, &campos),
        (EstadoApunte::Hecho, "hecho"),
    )
    .await
}

pub async fn reabrir(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    cambiar(
        &estado,
        &actor,
        (id, &campos),
        (EstadoApunte::Pendiente, "reabierto"),
    )
    .await
}

async fn cambiar(
    estado: &EstadoApp,
    actor: &Actor,
    (id, campos): (i64, &Campos),
    (nuevo, aviso): (EstadoApunte, &str),
) -> Result<Response, ErrorApp> {
    let version = campos.entero("version").unwrap_or(0);
    let ruta = destino(campos);
    match cambiar_apunte(estado, actor, (id, version), nuevo).await {
        Ok(()) if ruta == "/apuntes" => {
            Ok(Redirect::to(&format!("/apuntes?hecho={aviso}")).into_response())
        }
        Ok(()) => Ok(Redirect::to(ruta).into_response()),
        Err(ErrorApp::Validacion(texto)) => pagina_con_aviso(estado, actor, texto).await,
        Err(otro) => Err(otro),
    }
}

async fn pagina_con_aviso(
    estado: &EstadoApp,
    actor: &Actor,
    aviso: String,
) -> Result<Response, ErrorApp> {
    let filas = listar_apuntes(estado, actor, true, None).await?;
    let pagina = PaginaApuntes {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        abiertos: true,
        apuntes: filas
            .into_iter()
            .map(|f| TarjetaApunte::de(f, actor))
            .collect(),
        aviso: Some(aviso),
        confirmacion: None,
    };
    Ok((StatusCode::CONFLICT, renderizar(&pagina)?).into_response())
}

pub async fn hacer_presupuesto(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let version = Campos::de(pares).entero("version").unwrap_or(0);
    match pasar_a_presupuesto(&estado, &actor, (id, version)).await {
        Ok(presupuesto) => {
            Ok(Redirect::to(&format!("/presupuestos/{presupuesto}/lineas/nueva")).into_response())
        }
        Err(ErrorApp::Validacion(texto)) => pagina_con_aviso(&estado, &actor, texto).await,
        Err(otro) => Err(otro),
    }
}
