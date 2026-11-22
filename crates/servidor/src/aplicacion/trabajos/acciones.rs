//! Lo que cambia un trabajo: asignar, marcar pasos, avisar de que falta algo, cristal recibido,
//! entregar y anular. Cada cambio queda en la auditoría con quién y cuándo.

use dominio::acceso::Accion;
use dominio::taller::{
    AccionDePaso, EstadoDePaso, EstadoDeTrabajo, aplicar_accion, calcular_estado,
};
use rusqlite::Transaction;

use super::consultar::ventana_para;
use crate::aplicacion::auditar::{Cambio, anotar};
use crate::aplicacion::{Actor, autorizar};
use crate::bd::consultas::pasos::{self, ApunteDePaso};
use crate::bd::consultas::repetidas::{self, NuevaRepetida};
use crate::bd::consultas::{trabajos, usuarios, ventanas_trabajo};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

/// Ejecuta `operacion` en una transacción y anota el cambio sobre la entidad que devuelve.
async fn cambiar<T: Send + 'static>(
    estado: &EstadoApp,
    actor: &Actor,
    (permiso, accion, entidad): (Accion, &'static str, &'static str),
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
                entidad,
                entidad_id,
            };
            anotar(tx, &cambio)?;
            Ok(resultado)
        })
        .await
}

fn comprobar_usuario(tx: &Transaction, usuario: Option<i64>) -> Result<(), ErrorApp> {
    match usuario {
        Some(id) if usuarios::buscar_por_id(tx, id)?.is_none_or(|u| !u.activo) => Err(
            ErrorApp::Validacion("Ese operario no existe o está desactivado.".into()),
        ),
        _ => Ok(()),
    }
}

fn comprobar_fecha(fecha: Option<&str>) -> Result<(), ErrorApp> {
    let valida = fecha.is_none_or(|f| {
        time::Date::parse(f, time::macros::format_description!("[year]-[month]-[day]")).is_ok()
    });
    if valida {
        Ok(())
    } else {
        Err(ErrorApp::Validacion(
            "La fecha prevista no es válida.".into(),
        ))
    }
}

pub async fn asignar_trabajo(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
    (operario, fecha): (Option<i64>, Option<String>),
) -> Result<(), ErrorApp> {
    comprobar_fecha(fecha.as_deref())?;
    let permiso = (Accion::GestionarTrabajos, "asignar_trabajo", "trabajo");
    cambiar(estado, actor, permiso, move |tx, _| {
        comprobar_usuario(tx, operario)?;
        trabajos::cambiar_datos(tx, (id, version), (operario, fecha.as_deref()))?;
        Ok(((), id))
    })
    .await
}

pub async fn asignar_ventana(
    estado: &EstadoApp,
    actor: &Actor,
    ventana: i64,
    operario: Option<i64>,
) -> Result<(), ErrorApp> {
    let permiso = (
        Accion::GestionarTrabajos,
        "asignar_ventana",
        "ventana_trabajo",
    );
    cambiar(estado, actor, permiso, move |tx, _| {
        comprobar_usuario(tx, operario)?;
        ventanas_trabajo::asignar(tx, ventana, operario)?;
        Ok(((), ventana))
    })
    .await
}

/// Con el estado del trabajo recalculado a partir de todos sus pasos.
pub async fn cambiar_paso(
    estado: &EstadoApp,
    actor: &Actor,
    (paso, version): (i64, i64),
    accion: AccionDePaso,
) -> Result<i64, ErrorApp> {
    let quien = actor.clone();
    let permiso = (Accion::MarcarPasos, "cambiar_paso", "paso_ventana");
    cambiar(estado, actor, permiso, move |tx, ahora| {
        let fila = pasos::buscar(tx, paso)?.ok_or(ErrorApp::NoEncontrado)?;
        let (ventana, trabajo) = ventana_para(tx, &quien, fila.ventana_id)?;
        let actual = EstadoDePaso::desde_texto(&fila.estado).unwrap_or(EstadoDePaso::Pendiente);
        let nuevo =
            aplicar_accion(actual, accion).map_err(|e| ErrorApp::Validacion(e.to_string()))?;
        pasos::cambiar(tx, (paso, version), (nuevo.como_texto(), quien.id, ahora))?;
        let apunte = ApunteDePaso {
            paso,
            anterior: actual.como_texto(),
            nuevo: nuevo.como_texto(),
            nota: None,
            usuario: quien.id,
            cuando: ahora,
        };
        pasos::anotar(tx, &apunte)?;
        actualizar_estado(tx, trabajo.id, &trabajo.estado)?;
        Ok((ventana.fila.id, paso))
    })
    .await
}

pub fn actualizar_estado(tx: &Transaction, trabajo: i64, actual: &str) -> Result<(), ErrorApp> {
    let estados: Vec<EstadoDePaso> = pasos::de_trabajo(tx, trabajo)?
        .iter()
        .filter_map(|p| EstadoDePaso::desde_texto(&p.estado))
        .collect();
    let actual = EstadoDeTrabajo::desde_texto(actual).unwrap_or(EstadoDeTrabajo::Pendiente);
    let nuevo = calcular_estado(actual, &estados);
    trabajos::cambiar_estado(tx, trabajo, nuevo.como_texto())
}

/// «Falta algo» (falta cristal, material…): la ventana queda marcada y el aviso llega al Inicio.
/// Sin nota, se quita el aviso.
pub async fn anotar_falta(
    estado: &EstadoApp,
    actor: &Actor,
    ventana: i64,
    nota: Option<String>,
) -> Result<(), ErrorApp> {
    let nota = nota
        .map(|n| n.trim().chars().take(200).collect::<String>())
        .filter(|n| !n.is_empty());
    let quien = actor.clone();
    let permiso = (Accion::MarcarPasos, "falta_algo", "ventana_trabajo");
    cambiar(estado, actor, permiso, move |tx, _| {
        ventana_para(tx, &quien, ventana)?;
        ventanas_trabajo::anotar_falta(tx, ventana, nota.as_deref())?;
        Ok(((), ventana))
    })
    .await
}

pub async fn marcar_cristal(
    estado: &EstadoApp,
    actor: &Actor,
    ventana: i64,
    recibido: bool,
) -> Result<(), ErrorApp> {
    let permiso = (
        Accion::GestionarTrabajos,
        "cristal_recibido",
        "ventana_trabajo",
    );
    cambiar(estado, actor, permiso, move |tx, _| {
        ventanas_trabajo::marcar_cristal(tx, ventana, recibido)?;
        Ok(((), ventana))
    })
    .await
}

/// Entregar (decisión 6 del plan: a mano hasta H6) o anular. Lo anulado no vuelve.
pub async fn cerrar_trabajo(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    nuevo: EstadoDeTrabajo,
) -> Result<(), ErrorApp> {
    let accion = match nuevo {
        EstadoDeTrabajo::Entregado => "entregar_trabajo",
        EstadoDeTrabajo::Anulado => "anular_trabajo",
        _ => return Err(ErrorApp::Validacion("Ese cambio no se hace a mano.".into())),
    };
    let permiso = (Accion::GestionarTrabajos, accion, "trabajo");
    cambiar(estado, actor, permiso, move |tx, _| {
        let trabajo = trabajos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        if matches!(trabajo.estado.as_str(), "anulado" | "entregado") {
            return Err(ErrorApp::Validacion(format!(
                "El trabajo ya está {}.",
                trabajo.estado
            )));
        }
        trabajos::cambiar_estado(tx, id, nuevo.como_texto())?;
        Ok(((), id))
    })
    .await
}

/// «Repetir pieza» (corte mal, rayón, golpe…): el operario de la ventana o el propietario, mientras
/// la orden no esté terminada. La pieza de más entra en el plan y se descuenta al terminar.
pub async fn repetir_pieza(
    estado: &EstadoApp,
    actor: &Actor,
    (ventana, numero, cantidad): (i64, i64, i64),
    motivo: String,
) -> Result<(), ErrorApp> {
    let motivo: String = motivo.trim().chars().take(200).collect();
    if motivo.is_empty() || !(1..=20).contains(&cantidad) {
        return Err(ErrorApp::Validacion(
            "Elige el motivo y cuántas piezas (de 1 a 20).".into(),
        ));
    }
    let quien = actor.clone();
    let permiso = (Accion::MarcarPasos, "repetir_pieza", "ventana_trabajo");
    cambiar(estado, actor, permiso, move |tx, ahora| {
        let (v, trabajo) = ventana_para(tx, &quien, ventana)?;
        if trabajo.orden_terminada {
            return Err(ErrorApp::Validacion("La orden ya está terminada.".into()));
        }
        let pieza = v
            .congelado
            .resumen
            .piezas
            .iter()
            .find(|p| i64::from(p.numero) == numero)
            .ok_or_else(|| ErrorApp::Validacion("Esa pieza no está en la ventana.".into()))?;
        let nueva = NuevaRepetida {
            ventana,
            numero,
            perfil: &pieza.perfil,
            longitud_dmm: pieza.longitud_dmm,
            cantidad,
            motivo: &motivo,
            usuario: quien.id,
            cuando: ahora,
        };
        repetidas::insertar(tx, &nueva)?;
        Ok(((), ventana))
    })
    .await
}
