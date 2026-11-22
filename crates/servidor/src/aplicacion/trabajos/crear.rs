//! El trabajo nace al aceptar el presupuesto, en la misma transacción (DATOS 3.6): una ventana
//! de taller por unidad de cada línea de ventana, con los pasos de Ajustes > Taller.

use rusqlite::Connection;

use super::congelado::congelar_linea;
use crate::aplicacion::lineas_de_presupuesto::a_json;
use crate::aplicacion::presupuestos::FichaDePresupuesto;
use crate::aplicacion::{Actor, autorizar};
use crate::bd::consultas::configuracion::Valor;
use crate::bd::consultas::trabajos::{self, NuevoTrabajo};
use crate::bd::consultas::{configuracion, pasos, presupuestos, ventanas_trabajo};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{anio_de_madrid, formatear_utc, leer_utc};
use dominio::acceso::Accion;

pub const CLAVE_PASOS: &str = "taller.pasos";

pub fn pasos_configurados(con: &Connection) -> Result<Vec<String>, ErrorApp> {
    let texto = configuracion::leer(con, CLAVE_PASOS)?.unwrap_or_default();
    Ok(dominio::taller::leer_pasos(&texto))
}

pub async fn leer_pasos_del_taller(estado: &EstadoApp) -> Result<Vec<String>, ErrorApp> {
    estado.bd.leer(pasos_configurados).await
}

/// Los trabajos que ya existen conservan sus pasos: los nuevos valen desde el siguiente.
pub async fn guardar_pasos_del_taller(
    estado: &EstadoApp,
    actor: &Actor,
    texto: String,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    let valor = dominio::taller::leer_pasos(&texto).join("\n");
    let (ahora, usuario_id) = (formatear_utc(estado.ahora()), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let nuevo = Valor {
                clave: CLAVE_PASOS,
                valor: &valor,
                ahora: &ahora,
                usuario_id,
            };
            configuracion::guardar(tx, &nuevo)
        })
        .await
}

/// Una línea con cantidad 3 son 3 ventanas (decisión 3 del plan de H4); 1,5 se redondea arriba.
fn unidades(cantidad_mil: i64) -> i64 {
    (cantidad_mil + 999) / 1000
}

pub fn crear_trabajo(
    con: &Connection,
    ficha: &FichaDePresupuesto,
    ahora: &str,
) -> Result<i64, ErrorApp> {
    let instante =
        leer_utc(ahora).ok_or_else(|| ErrorApp::Interno(format!("fecha no válida: {ahora}")))?;
    let anio = anio_de_madrid(instante);
    let p = &ficha.presupuesto;
    let direccion = ficha
        .direcciones
        .iter()
        .find(|d| Some(d.id) == p.direccion_montaje_id)
        .map(|d| d.datos.resumen());
    let id = trabajos::insertar(
        con,
        &NuevoTrabajo {
            anio,
            numero: presupuestos::siguiente_numero(con, "T", anio)?,
            presupuesto_id: p.id,
            cliente_id: ficha.cliente.id,
            cliente_nombre: &ficha.cliente.nombre_visible(),
            direccion_montaje: direccion.as_deref(),
            importe_cent: p.total_cent,
            creado_en: ahora,
        },
    )?;
    let nombres = pasos_configurados(con)?;
    let mut orden = 0;
    for linea in ficha.lineas.iter().filter(|l| l.tipo == "ventana") {
        let congelado = a_json(&congelar_linea(con, linea)?)?;
        for _ in 0..unidades(linea.cantidad_mil) {
            orden += 1;
            let datos = (linea.descripcion.as_str(), congelado.as_str());
            let ventana = ventanas_trabajo::insertar(con, id, (orden, linea.orden), datos)?;
            for (posicion, nombre) in (1..).zip(&nombres) {
                pasos::insertar(con, ventana, posicion, nombre)?;
            }
        }
    }
    Ok(id)
}

#[cfg(test)]
mod pruebas {
    use super::unidades;

    #[test]
    fn cada_unidad_de_la_linea_es_una_ventana() {
        assert_eq!(unidades(1000), 1);
        assert_eq!(unidades(3000), 3);
        assert_eq!(unidades(1500), 2);
    }
}
