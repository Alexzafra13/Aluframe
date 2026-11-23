use dominio::acceso::Accion;
use dominio::clientes::{DatosDeCliente, DatosDeDireccion, nombre_visible};
use dominio::nif::Nif;
use dominio::texto::normalizar_para_buscar;
use rusqlite::Transaction;

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::clientes::{self, FilaCliente, Pagina};
use crate::bd::consultas::direcciones::{self, FilaDireccion};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

pub const POR_PAGINA: i64 = 50;

pub struct ConsultaDeClientes {
    pub texto: String,
    pub archivados: bool,
    pub pagina: i64,
}

pub struct ListaDeClientes {
    pub clientes: Vec<FilaCliente>,
    pub total: i64,
    pub pagina: i64,
    pub paginas: i64,
}

pub struct FichaDeCliente {
    pub cliente: FilaCliente,
    pub direcciones: Vec<FilaDireccion>,
}

impl FilaCliente {
    pub fn nombre_visible(&self) -> String {
        nombre_visible(self.id, self.nombre.as_deref())
    }
}

pub async fn listar_clientes(
    estado: &EstadoApp,
    actor: &Actor,
    consulta: ConsultaDeClientes,
) -> Result<ListaDeClientes, ErrorApp> {
    autorizar(actor, Accion::VerClientes)?;
    let pagina = consulta.pagina.max(1);
    let filtro = Pagina {
        filtro: normalizar_para_buscar(&consulta.texto),
        // Los archivados son cosa del dueño (protección de datos).
        archivados: consulta.archivados && actor.puede(Accion::GestionarClientes),
        limite: POR_PAGINA,
        desplazamiento: (pagina - 1) * POR_PAGINA,
    };
    let (clientes, total) = estado
        .bd
        .leer(move |con| clientes::listar(con, &filtro))
        .await?;
    Ok(ListaDeClientes {
        clientes,
        total,
        pagina,
        paginas: ((total + POR_PAGINA - 1) / POR_PAGINA).max(1),
    })
}

/// Todos los clientes activos para elegir uno (nuevo presupuesto, factura libre), sin paginar:
/// con la primera página sola, el que no salía parecía no existir y se daba de alta dos veces.
pub async fn clientes_para_elegir(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<(i64, String)>, ErrorApp> {
    autorizar(actor, Accion::VerClientes)?;
    let filas = estado.bd.leer(clientes::para_elegir).await?;
    Ok(filas
        .into_iter()
        .map(|(id, nombre)| (id, nombre_visible(id, nombre.as_deref())))
        .collect())
}

pub async fn cargar_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FichaDeCliente, ErrorApp> {
    autorizar(actor, Accion::VerClientes)?;
    let ve_archivados = actor.puede(Accion::GestionarClientes);
    estado
        .bd
        .leer(move |con| {
            let cliente = clientes::buscar(con, id)?
                .filter(|c| ve_archivados || !c.archivado)
                .ok_or(ErrorApp::NoEncontrado)?;
            let direcciones = direcciones::listar(con, id)?;
            Ok(FichaDeCliente {
                cliente,
                direcciones,
            })
        })
        .await
}

/// Ejecuta un cambio de clientes en una transacción y deja su apunte de auditoría.
/// `permiso`: dar de alta y corregir lo puede un operario con permiso; archivar y borrar, solo
/// el dueño (protección de datos).
async fn cambiar<T: Send + 'static>(
    estado: &EstadoApp,
    (actor, permiso): (&Actor, Accion),
    accion: &'static str,
    operacion: impl FnOnce(&Transaction, &str) -> Result<(T, i64), ErrorApp> + Send + 'static,
) -> Result<T, ErrorApp> {
    autorizar(actor, permiso)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let (resultado, entidad_id) = operacion(tx, &formatear_utc(ahora))?;
            let cambio = Cambio {
                ahora,
                autor,
                accion,
                entidad: "cliente",
                entidad_id,
            };
            anotar(tx, &cambio)?;
            Ok(resultado)
        })
        .await
}

fn comprobar_nif_libre(
    tx: &Transaction,
    nif: Option<&Nif>,
    excepto: Option<i64>,
) -> Result<(), ErrorApp> {
    let Some(nif) = nif else { return Ok(()) };
    match clientes::buscar_por_nif(tx, nif.como_texto(), excepto)? {
        Some(otro) => Err(ErrorApp::Validacion(format!(
            "Ya existe un cliente con el NIF {nif}: {} (n.º {}).",
            otro.nombre_visible(),
            otro.id
        ))),
        None => Ok(()),
    }
}

pub async fn crear_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    datos: DatosDeCliente,
    nuevas_direcciones: Vec<DatosDeDireccion>,
) -> Result<i64, ErrorApp> {
    cambiar(
        estado,
        (actor, Accion::EditarClientes),
        "crear_cliente",
        move |tx, ahora| {
            comprobar_nif_libre(tx, datos.nif.as_ref(), None)?;
            let id = clientes::insertar(tx, &datos, ahora, None)?;
            for direccion in &nuevas_direcciones {
                direcciones::insertar(tx, id, direccion)?;
            }
            Ok((id, id))
        },
    )
    .await
}

pub async fn actualizar_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
    datos: DatosDeCliente,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        (actor, Accion::EditarClientes),
        "editar_cliente",
        move |tx, _| {
            comprobar_nif_libre(tx, datos.nif.as_ref(), Some(id))?;
            clientes::actualizar(tx, id, version, &datos)?;
            Ok(((), id))
        },
    )
    .await
}

pub async fn archivar_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    archivar: bool,
) -> Result<(), ErrorApp> {
    let accion = if archivar {
        "archivar_cliente"
    } else {
        "recuperar_cliente"
    };
    cambiar(
        estado,
        (actor, Accion::GestionarClientes),
        accion,
        move |tx, _| {
            clientes::cambiar_archivado(tx, id, archivar)?;
            Ok(((), id))
        },
    )
    .await
}

/// Derecho de supresión (RGPD). Con presupuestos no se borra; anonimizar llega con las
/// facturas (H6), que obligan a conservar los datos fiscales.
pub async fn borrar_cliente(estado: &EstadoApp, actor: &Actor, id: i64) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        (actor, Accion::GestionarClientes),
        "borrar_cliente",
        move |tx, _| {
            if crate::bd::consultas::presupuestos::tiene_presupuestos(tx, id)? {
                return Err(ErrorApp::Validacion(
                    "Este cliente tiene presupuestos y no se puede borrar: archívalo.".into(),
                ));
            }
            clientes::borrar(tx, id)?;
            Ok(((), id))
        },
    )
    .await
}

pub async fn anadir_direccion(
    estado: &EstadoApp,
    actor: &Actor,
    cliente_id: i64,
    datos: DatosDeDireccion,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        (actor, Accion::EditarClientes),
        "anadir_direccion",
        move |tx, _| {
            clientes::buscar(tx, cliente_id)?.ok_or(ErrorApp::NoEncontrado)?;
            direcciones::insertar(tx, cliente_id, &datos)?;
            Ok(((), cliente_id))
        },
    )
    .await
}

pub async fn actualizar_direccion(
    estado: &EstadoApp,
    actor: &Actor,
    ids: (i64, i64),
    datos: DatosDeDireccion,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        (actor, Accion::EditarClientes),
        "editar_direccion",
        move |tx, _| {
            direcciones::actualizar(tx, ids, &datos)?;
            Ok(((), ids.0))
        },
    )
    .await
}

pub async fn quitar_direccion(
    estado: &EstadoApp,
    actor: &Actor,
    (cliente_id, id): (i64, i64),
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        (actor, Accion::EditarClientes),
        "quitar_direccion",
        move |tx, _| {
            direcciones::archivar(tx, cliente_id, id)?;
            Ok(((), cliente_id))
        },
    )
    .await
}
