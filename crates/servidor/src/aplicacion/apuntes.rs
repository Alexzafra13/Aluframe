//! Apuntes (ADR-0025). Todos los escriben; el operario ve los suyos y las tareas que le mandan,
//! y solo marca como hechas esas tareas. El dueño los ve todos, manda tareas y los pasa a
//! presupuesto.

use dominio::acceso::{Accion, Rol};
use dominio::apuntes::{DatosDeApunte, EstadoApunte};
use rusqlite::Transaction;

use super::auditar::{Cambio, anotar};
use super::presupuestos::insertar_presupuesto;
use super::{Actor, autorizar};
use crate::bd::consultas::apuntes::{self, Cierre, FilaApunte, FiltroDeApuntes, NuevoApunte};
use crate::bd::consultas::{clientes, direcciones, usuarios};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

const OTRO_CAMBIO: &str = "Otra persona cambió este apunte a la vez. Revisa cómo ha quedado.";

/// Lo que se quiere apuntar. `para`: un operario (tarea; solo lo manda el dueño) o nadie
/// (para el dueño).
pub struct PeticionDeApunte {
    pub cliente: Option<i64>,
    pub direccion: Option<i64>,
    pub para: Option<i64>,
    pub datos: DatosDeApunte,
}

pub async fn crear_apunte(
    estado: &EstadoApp,
    actor: &Actor,
    peticion: PeticionDeApunte,
) -> Result<i64, ErrorApp> {
    autorizar(actor, Accion::TomarApuntes)?;
    if peticion.para.is_some() {
        autorizar(actor, Accion::GestionarApuntes)?;
    }
    let ve_archivados = actor.puede(Accion::GestionarClientes);
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            comprobar_destino(tx, &peticion, ve_archivados)?;
            let cuando = formatear_utc(ahora);
            let nuevo = NuevoApunte {
                autor_id: autor,
                para_id: peticion.para,
                cliente_id: peticion.cliente,
                direccion_id: peticion.direccion,
                datos: &peticion.datos,
                creado_en: &cuando,
            };
            let id = apuntes::insertar(tx, &nuevo)?;
            anotar(tx, &cambio(ahora, autor, "crear_apunte", id))?;
            Ok(id)
        })
        .await
}

fn cambio(ahora: time::OffsetDateTime, autor: i64, accion: &str, id: i64) -> Cambio<'_> {
    Cambio {
        ahora,
        autor,
        accion,
        entidad: "apunte",
        entidad_id: id,
    }
}

/// Que el operario exista y esté activo, y que el cliente y su dirección cuadren.
fn comprobar_destino(
    tx: &Transaction,
    p: &PeticionDeApunte,
    ve_archivados: bool,
) -> Result<(), ErrorApp> {
    if let Some(para) = p.para {
        usuarios::buscar_por_id(tx, para)?
            .filter(|u| u.rol == Rol::Operario && u.activo)
            .ok_or_else(|| {
                ErrorApp::Validacion("Ese operario no existe o no está activo.".into())
            })?;
    }
    match (p.cliente, p.direccion) {
        (None, Some(_)) => Err(ErrorApp::Validacion(
            "Para indicar una dirección de montaje, elige antes el cliente.".into(),
        )),
        (Some(cliente), direccion) => {
            clientes::buscar(tx, cliente)?
                .filter(|c| ve_archivados || !c.archivado)
                .ok_or(ErrorApp::NoEncontrado)?;
            let es_suya = |id| -> Result<bool, ErrorApp> {
                Ok(direcciones::listar(tx, cliente)?.iter().any(|d| d.id == id))
            };
            match direccion {
                Some(id) if !es_suya(id)? => Err(ErrorApp::Validacion(
                    "La dirección de montaje no es de este cliente.".into(),
                )),
                _ => Ok(()),
            }
        }
        (None, None) => Ok(()),
    }
}

/// Pendientes (`abiertos`) o cerrados; el dueño ve todos, el operario los suyos.
pub async fn listar_apuntes(
    estado: &EstadoApp,
    actor: &Actor,
    abiertos: bool,
    cliente: Option<i64>,
) -> Result<Vec<FilaApunte>, ErrorApp> {
    autorizar(actor, Accion::TomarApuntes)?;
    let filtro = FiltroDeApuntes {
        abiertos,
        de_usuario: (!actor.puede(Accion::GestionarApuntes)).then_some(actor.id),
        cliente,
    };
    estado
        .bd
        .leer(move |con| apuntes::listar(con, &filtro))
        .await
}

/// Las tareas que el dueño le ha mandado, para su «Mi día».
pub async fn mis_tareas(estado: &EstadoApp, actor: &Actor) -> Result<Vec<FilaApunte>, ErrorApp> {
    autorizar(actor, Accion::TomarApuntes)?;
    let usuario = actor.id;
    estado
        .bd
        .leer(move |con| apuntes::tareas_de(con, usuario))
        .await
}

/// Los pendientes para el dueño, para la campana y el Inicio.
pub async fn apuntes_para_el_dueno(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<FilaApunte>, ErrorApp> {
    autorizar(actor, Accion::GestionarApuntes)?;
    estado.bd.leer(apuntes::pendientes_para_el_dueno).await
}

/// Marcar como hecho o reabrir. El operario solo puede con las tareas que le mandan.
pub async fn cambiar_apunte(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
    nuevo: EstadoApunte,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::TomarApuntes)?;
    let gestiona = actor.puede(Accion::GestionarApuntes);
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let apunte = apuntes::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
            if !gestiona && apunte.para_id != Some(autor) {
                return Err(ErrorApp::NoEncontrado);
            }
            let actual = estado_de(&apunte)?;
            let siguiente = actual.pasar_a(nuevo)?;
            if siguiente == EstadoApunte::Presupuestado {
                return Err(ErrorApp::NoEncontrado);
            }
            let cuando = formatear_utc(ahora);
            let cerrado = (siguiente != EstadoApunte::Pendiente).then_some(cuando.as_str());
            let cierre = Cierre {
                id,
                version,
                estado: siguiente.como_texto(),
                cuando: cerrado,
                quien: cerrado.map(|_| autor),
                presupuesto_id: None,
            };
            if !apuntes::cambiar_estado(tx, &cierre)? {
                return Err(ErrorApp::Validacion(OTRO_CAMBIO.into()));
            }
            anotar(tx, &cambio(ahora, autor, siguiente.como_texto(), id))
        })
        .await
}

fn estado_de(apunte: &FilaApunte) -> Result<EstadoApunte, ErrorApp> {
    EstadoApunte::desde_texto(&apunte.estado).ok_or_else(|| {
        ErrorApp::Interno(format!("estado de apunte desconocido: {}", apunte.estado))
    })
}

/// Crea el borrador del presupuesto con el cliente y la dirección del apunte y lo deja como
/// «presupuestado», todo en la misma transacción. Devuelve el presupuesto.
pub async fn pasar_a_presupuesto(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
) -> Result<i64, ErrorApp> {
    autorizar(actor, Accion::GestionarApuntes)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    super::presupuestos::cambiar(estado, actor, "crear_presupuesto", move |tx, cuando| {
        let apunte = apuntes::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        estado_de(&apunte)?.pasar_a(EstadoApunte::Presupuestado)?;
        let cliente = apunte.cliente_id.ok_or_else(|| {
            ErrorApp::Validacion("Para hacer el presupuesto, el apunte necesita un cliente.".into())
        })?;
        let direccion = apunte.direccion_id;
        let presupuesto = insertar_presupuesto(tx, (cliente, direccion), (ahora, cuando, autor))?;
        let cierre = Cierre {
            id,
            version,
            estado: EstadoApunte::Presupuestado.como_texto(),
            cuando: Some(cuando),
            quien: Some(autor),
            presupuesto_id: Some(presupuesto),
        };
        if !apuntes::cambiar_estado(tx, &cierre)? {
            return Err(ErrorApp::Validacion(OTRO_CAMBIO.into()));
        }
        anotar(tx, &cambio(ahora, autor, "presupuestar_apunte", id))?;
        Ok((presupuesto, presupuesto))
    })
    .await
}
