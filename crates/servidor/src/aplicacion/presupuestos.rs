use dominio::acceso::Accion;
use rusqlite::{Connection, Transaction};

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::clientes::{self, FilaCliente};
use crate::bd::consultas::direcciones::{self, FilaDireccion};
use crate::bd::consultas::lineas_presupuesto::{self, FilaLinea};
use crate::bd::consultas::presupuestos::{self, Cabecera, FilaPresupuesto, NuevoPresupuesto};
use crate::bd::consultas::versiones_presupuesto::{self, FilaVersion};
use crate::bd::consultas::{configuracion, tarifas};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{anio_de_madrid, fecha_de_madrid, formatear_utc};

pub struct FichaDePresupuesto {
    pub presupuesto: FilaPresupuesto,
    pub cliente: FilaCliente,
    pub direcciones: Vec<FilaDireccion>,
    pub lineas: Vec<FilaLinea>,
    pub versiones: Vec<FilaVersion>,
    /// Alguna línea se calculó con una tarifa que ya no es la vigente.
    pub tarifas_cambiadas: bool,
    /// El trabajo que nació al aceptarlo.
    pub trabajo_id: Option<i64>,
}

impl FichaDePresupuesto {
    pub fn es_editable(&self) -> bool {
        self.presupuesto.estado == "borrador"
    }
}

pub async fn listar_presupuestos(
    estado: &EstadoApp,
    actor: &Actor,
    filtro: Option<String>,
) -> Result<Vec<FilaPresupuesto>, ErrorApp> {
    autorizar(actor, Accion::VerPresupuestos)?;
    estado
        .bd
        .leer(move |con| presupuestos::listar(con, filtro.as_deref(), None))
        .await
}

pub async fn presupuestos_del_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    cliente_id: i64,
) -> Result<Vec<FilaPresupuesto>, ErrorApp> {
    autorizar(actor, Accion::VerPresupuestos)?;
    estado
        .bd
        .leer(move |con| presupuestos::listar(con, None, Some(cliente_id)))
        .await
}

pub fn cargar(con: &Connection, id: i64) -> Result<FichaDePresupuesto, ErrorApp> {
    let presupuesto = presupuestos::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
    let cliente = clientes::buscar(con, presupuesto.cliente_id)?.ok_or(ErrorApp::NoEncontrado)?;
    let lineas = lineas_presupuesto::listar(con, presupuesto.version_id)?;
    let mut tarifas_cambiadas = false;
    for linea in lineas.iter().filter(|l| l.tipo == "ventana") {
        let usadas: Vec<i64> = serde_json::from_str(&linea.tarifas_json).unwrap_or_default();
        for tarifa in usadas {
            tarifas_cambiadas |=
                tarifas::buscar(con, tarifa)?.is_none_or(|t| t.estado != "vigente");
        }
    }
    Ok(FichaDePresupuesto {
        direcciones: direcciones::listar(con, cliente.id)?,
        versiones: versiones_presupuesto::listar(con, id)?,
        tarifas_cambiadas: tarifas_cambiadas && presupuesto.estado == "borrador",
        trabajo_id: crate::bd::consultas::trabajos::de_presupuesto(con, id)?,
        presupuesto,
        cliente,
        lineas,
    })
}

pub async fn cargar_presupuesto(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FichaDePresupuesto, ErrorApp> {
    autorizar(actor, Accion::VerPresupuestos)?;
    estado.bd.leer(move |con| cargar(con, id)).await
}

/// Ejecuta un cambio en una transacción y deja su apunte de auditoría.
pub async fn cambiar<T: Send + 'static>(
    estado: &EstadoApp,
    actor: &Actor,
    accion: &'static str,
    operacion: impl FnOnce(&Transaction, &str) -> Result<(T, i64), ErrorApp> + Send + 'static,
) -> Result<T, ErrorApp> {
    autorizar(actor, Accion::GestionarPresupuestos)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let (resultado, id) = operacion(tx, &formatear_utc(ahora))?;
            anotar(
                tx,
                &Cambio {
                    ahora,
                    autor,
                    accion,
                    entidad: "presupuesto",
                    entidad_id: id,
                },
            )?;
            Ok(resultado)
        })
        .await
}

pub async fn crear_presupuesto(
    estado: &EstadoApp,
    actor: &Actor,
    cliente_id: i64,
    direccion: Option<i64>,
) -> Result<i64, ErrorApp> {
    let (ahora, autor) = (estado.ahora(), actor.id);
    cambiar(estado, actor, "crear_presupuesto", move |tx, cuando| {
        clientes::buscar(tx, cliente_id)?.ok_or(ErrorApp::NoEncontrado)?;
        comprobar_direccion(tx, cliente_id, direccion)?;
        let validez = configuracion::leer(tx, "presupuesto.validez_dias")?
            .and_then(|d| d.parse::<i64>().ok())
            .map(|dias| fecha_de_madrid(ahora, dias));
        let condiciones =
            configuracion::leer(tx, "presupuesto.condiciones")?.filter(|c| !c.trim().is_empty());
        let anio = anio_de_madrid(ahora);
        let nuevo = NuevoPresupuesto {
            anio,
            numero: presupuestos::siguiente_numero(tx, "P", anio)?,
            cliente_id,
            direccion_montaje_id: direccion,
            validez_hasta: validez.as_deref(),
            condiciones: condiciones.as_deref(),
            creado_por: autor,
            ahora: cuando,
        };
        let id = presupuestos::insertar(tx, &nuevo)?;
        Ok((id, id))
    })
    .await
}

fn comprobar_direccion(
    con: &Connection,
    cliente_id: i64,
    direccion: Option<i64>,
) -> Result<(), ErrorApp> {
    let Some(id) = direccion else { return Ok(()) };
    if direcciones::listar(con, cliente_id)?
        .iter()
        .any(|d| d.id == id)
    {
        Ok(())
    } else {
        Err(ErrorApp::Validacion(
            "La dirección de montaje no es de este cliente.".into(),
        ))
    }
}

/// Solo un borrador se modifica; lo enviado se cambia con una versión nueva.
pub fn exigir_borrador(presupuesto: &FilaPresupuesto) -> Result<(), ErrorApp> {
    if presupuesto.estado == "borrador" {
        Ok(())
    } else {
        Err(ErrorApp::Validacion(format!(
            "El presupuesto está {}: crea una versión nueva para cambiarlo.",
            presupuesto.estado
        )))
    }
}

pub struct DatosDeCabecera {
    pub direccion_montaje_id: Option<i64>,
    pub validez_hasta: Option<String>,
    pub condiciones: Option<String>,
    pub descuento_bp: i64,
}

pub async fn guardar_cabecera(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
    datos: DatosDeCabecera,
) -> Result<(), ErrorApp> {
    cambiar(estado, actor, "editar_presupuesto", move |tx, _| {
        let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        exigir_borrador(&presupuesto)?;
        comprobar_direccion(tx, presupuesto.cliente_id, datos.direccion_montaje_id)?;
        let cabecera = Cabecera {
            direccion_montaje_id: datos.direccion_montaje_id,
            validez_hasta: datos.validez_hasta.as_deref(),
            condiciones: datos.condiciones.as_deref(),
            descuento_bp: datos.descuento_bp,
        };
        presupuestos::actualizar_cabecera(tx, (id, version), &cabecera)?;
        super::lineas_de_presupuesto::recalcular_totales(tx, id)?;
        Ok(((), id))
    })
    .await
}

/// Enviar, rechazar o volver a borrador con una versión nueva.
pub async fn cambiar_estado(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    nuevo: &'static str,
) -> Result<(), ErrorApp> {
    let accion = match nuevo {
        "enviado" => "enviar_presupuesto",
        "rechazado" => "rechazar_presupuesto",
        _ => return Err(ErrorApp::NoEncontrado),
    };
    cambiar(estado, actor, accion, move |tx, ahora| {
        let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        let permitido = match nuevo {
            "enviado" => presupuesto.estado == "borrador",
            _ => matches!(presupuesto.estado.as_str(), "borrador" | "enviado"),
        };
        if !permitido {
            return Err(ErrorApp::Validacion(format!(
                "Un presupuesto {} no puede pasar a {nuevo}.",
                presupuesto.estado
            )));
        }
        presupuestos::cambiar_estado(tx, id, nuevo, ahora)?;
        Ok(((), id))
    })
    .await
}

pub async fn nueva_version(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    motivo: Option<String>,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        actor,
        "nueva_version_presupuesto",
        move |tx, ahora| {
            let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
            if presupuesto.estado == "aceptado" {
                return Err(ErrorApp::Validacion(
                    "Un presupuesto aceptado no cambia.".into(),
                ));
            }
            versiones_presupuesto::crear_copia(tx, id, (motivo.as_deref(), ahora))?;
            presupuestos::cambiar_estado(tx, id, "borrador", ahora)?;
            Ok(((), id))
        },
    )
    .await
}
