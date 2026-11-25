use axum::Form;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use dominio::clientes::{
    DatosDeDireccion, EntradaDeCliente, EntradaDeDireccion, validar_alta_de_cliente,
    validar_direccion,
};
use serde::Deserialize;

pub use super::clientes_fragmentos::bloque_vacio;
use crate::aplicacion::Actor;
use crate::aplicacion::clientes::{
    ConsultaDeClientes, actualizar_cliente, cargar_cliente, crear_cliente, listar_clientes,
};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    BloqueDireccion, ErroresDeCampo, FragmentoListaClientes, MENU, PaginaClientes,
    PaginaFormularioCliente, VistaListaClientes, renderizar,
};

pub const SECCION: &str = "clientes";

#[derive(Deserialize)]
pub struct FiltroClientes {
    q: Option<String>,
    archivados: Option<String>,
    pagina: Option<i64>,
    importados: Option<usize>,
    importacion: Option<i64>,
    aviso: Option<String>,
    quitados: Option<usize>,
    conservados: Option<usize>,
}

/// Un trozo de página (la búsqueda mientras se escribe). La navegación sin recarga también
/// llega por HTMX, pero pide la página entera (`HX-Boosted`, ADR-0036).
pub fn es_htmx(cabeceras: &HeaderMap) -> bool {
    cabeceras.contains_key("hx-request") && !cabeceras.contains_key("hx-boosted")
}

pub async fn mostrar_clientes(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    cabeceras: HeaderMap,
    Query(filtro): Query<FiltroClientes>,
) -> Result<Response, ErrorApp> {
    let texto = filtro.q.clone().unwrap_or_default();
    let archivados = filtro.archivados.is_some();
    let consulta = ConsultaDeClientes {
        texto: texto.clone(),
        archivados,
        pagina: filtro.pagina.unwrap_or(1),
    };
    let lista = listar_clientes(&estado, &actor, consulta).await?;
    let lista = VistaListaClientes::nueva(&lista, texto, archivados);
    if es_htmx(&cabeceras) {
        return Ok(renderizar(&FragmentoListaClientes {
            usuario: (&actor).into(),
            lista,
        })?
        .into_response());
    }
    let confirmacion = texto_de_confirmacion(&filtro);
    let pagina = PaginaClientes {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: SECCION,
        lista,
        confirmacion,
        deshacer_importacion: filtro.importacion.filter(|_| filtro.importados.is_some()),
    };
    Ok(renderizar(&pagina)?.into_response())
}

struct Formulario {
    id: Option<i64>,
    version: i64,
    valores: EntradaDeCliente,
    errores: ErroresDeCampo,
    direcciones: Vec<BloqueDireccion>,
    misma_direccion: bool,
    aviso: Option<String>,
}

fn pagina_de_formulario(actor: &Actor, f: Formulario) -> Result<Response, ErrorApp> {
    let estado_http =
        if f.aviso.is_some() || f.errores.hay() || f.direcciones.iter().any(|d| d.errores.hay()) {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::OK
        };
    let pagina = PaginaFormularioCliente {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        id: f.id,
        version: f.version,
        valores: f.valores,
        errores: f.errores,
        direcciones: f.direcciones,
        misma_direccion: f.misma_direccion,
        aviso: f.aviso,
    };
    Ok((estado_http, renderizar(&pagina)?).into_response())
}

pub async fn mostrar_alta(ActorActual(actor): ActorActual) -> Result<Response, ErrorApp> {
    crate::aplicacion::autorizar(&actor, dominio::acceso::Accion::EditarClientes)?;
    let formulario = Formulario {
        id: None,
        version: 0,
        valores: EntradaDeCliente {
            fiscal_pais: "España".into(),
            ..EntradaDeCliente::default()
        },
        errores: ErroresDeCampo::default(),
        direcciones: vec![bloque_vacio(0)],
        misma_direccion: false,
        aviso: None,
    };
    pagina_de_formulario(&actor, formulario)
}

/// Valida los bloques de dirección; los vacíos se descartan sin aviso.
fn leer_direcciones(campos: &Campos) -> (Vec<DatosDeDireccion>, Vec<BloqueDireccion>, bool) {
    let mut validas = Vec::new();
    let mut bloques = Vec::new();
    let mut hay_errores = false;
    for (posicion, n) in campos.bloques_de_direccion().into_iter().enumerate() {
        let valores = campos.direccion(n);
        let errores = match validar_direccion(&valores) {
            Ok(Some(datos)) => {
                validas.push(datos);
                Vec::new()
            }
            Ok(None) => Vec::new(),
            Err(errores) => {
                hay_errores = true;
                errores
            }
        };
        bloques.push(BloqueDireccion {
            n: posicion,
            valores,
            errores: ErroresDeCampo(errores),
        });
    }
    (validas, bloques, hay_errores)
}

fn direccion_fiscal_como_montaje(entrada: &EntradaDeCliente) -> Option<DatosDeDireccion> {
    let copia = EntradaDeDireccion {
        etiqueta: "Dirección fiscal".into(),
        calle: entrada.fiscal_calle.clone(),
        cp: entrada.fiscal_cp.clone(),
        localidad: entrada.fiscal_localidad.clone(),
        provincia: entrada.fiscal_provincia.clone(),
        ..EntradaDeDireccion::default()
    };
    validar_direccion(&copia).ok().flatten()
}

pub async fn procesar_alta(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let valores = campos.cliente();
    let misma_direccion = campos.marcado("misma_direccion");
    let (mut direcciones, bloques, errores_en_direcciones) = leer_direcciones(&campos);
    let mut formulario = Formulario {
        id: None,
        version: 0,
        valores: valores.clone(),
        errores: ErroresDeCampo::default(),
        direcciones: bloques,
        misma_direccion,
        aviso: None,
    };
    let datos = match validar_alta_de_cliente(&valores) {
        Ok(datos) if !errores_en_direcciones => datos,
        Ok(_) => return pagina_de_formulario(&actor, formulario),
        Err(errores) => {
            formulario.errores = ErroresDeCampo(errores);
            return pagina_de_formulario(&actor, formulario);
        }
    };
    if misma_direccion {
        direcciones.splice(0..0, direccion_fiscal_como_montaje(&valores));
    }
    match crear_cliente(&estado, &actor, datos, direcciones).await {
        Ok(id) if campos.marcado("y_presupuesto") => {
            let presupuesto =
                crate::aplicacion::presupuestos::crear_presupuesto(&estado, &actor, id, None)
                    .await?;
            Ok(Redirect::to(&format!("/presupuestos/{presupuesto}/lineas/nueva")).into_response())
        }
        Ok(id) => Ok(Redirect::to(&format!("/clientes/{id}?aviso=creado")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            formulario.aviso = Some(aviso);
            pagina_de_formulario(&actor, formulario)
        }
        Err(otro) => Err(otro),
    }
}

pub async fn mostrar_edicion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let ficha = cargar_cliente(&estado, &actor, id).await?;
    let c = ficha.cliente;
    let texto = |v: Option<String>| v.unwrap_or_default();
    let valores = EntradaDeCliente {
        tipo: c.tipo,
        nombre: texto(c.nombre),
        nif: texto(c.nif),
        email: texto(c.email),
        telefono: texto(c.telefono),
        fiscal_calle: texto(c.fiscal_calle),
        fiscal_cp: texto(c.fiscal_cp),
        fiscal_localidad: texto(c.fiscal_localidad),
        fiscal_provincia: texto(c.fiscal_provincia),
        fiscal_pais: texto(c.fiscal_pais),
        forma_de_pago: texto(c.forma_de_pago),
        notas: texto(c.notas),
    };
    let formulario = Formulario {
        id: Some(id),
        version: c.version,
        valores,
        errores: ErroresDeCampo::default(),
        direcciones: Vec::new(),
        misma_direccion: false,
        aviso: None,
    };
    pagina_de_formulario(&actor, formulario)
}

pub async fn procesar_edicion(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares);
    let valores = campos.cliente();
    let version = campos.entero("version").unwrap_or(0);
    let mut formulario = Formulario {
        id: Some(id),
        version,
        valores: valores.clone(),
        errores: ErroresDeCampo::default(),
        direcciones: Vec::new(),
        misma_direccion: false,
        aviso: None,
    };
    let datos = match validar_alta_de_cliente(&valores) {
        Ok(datos) => datos,
        Err(errores) => {
            formulario.errores = ErroresDeCampo(errores);
            return pagina_de_formulario(&actor, formulario);
        }
    };
    match actualizar_cliente(&estado, &actor, (id, version), datos).await {
        Ok(()) => Ok(Redirect::to(&format!("/clientes/{id}?aviso=guardado")).into_response()),
        Err(error @ (ErrorApp::Validacion(_) | ErrorApp::Conflicto)) => {
            formulario.aviso = Some(error.to_string());
            pagina_de_formulario(&actor, formulario)
        }
        Err(otro) => Err(otro),
    }
}

fn texto_de_confirmacion(filtro: &FiltroClientes) -> Option<String> {
    if let Some(n) = filtro.importados {
        return Some(format!("Importados {n} clientes."));
    }
    if let Some(quitados) = filtro.quitados {
        let mut texto = format!("Importación deshecha: {quitados} clientes quitados.");
        if let Some(n) = filtro.conservados.filter(|n| *n > 0) {
            texto.push_str(&format!(
                " {n} se conservan porque se modificaron después de importarlos."
            ));
        }
        return Some(texto);
    }
    (filtro.aviso.as_deref() == Some("borrado"))
        .then(|| "Cliente borrado definitivamente.".to_owned())
}
