//! Albarán de entrega de un trabajo (DATOS 3.8): numerado (A-AAAA-NNNN), con la fotografía de lo
//! entregado y su huella; no se modifica. Al crearlo, el trabajo queda como entregado.

use serde_json::json;

use super::cambiar;
use crate::aplicacion::Actor;
use crate::aplicacion::trabajos::consultar::cargar as cargar_trabajo;
use crate::autenticacion::calcular_sha256;
pub use crate::bd::consultas::albaranes::{FilaAlbaran, de_trabajo};
use crate::bd::consultas::albaranes::{buscar, insertar};
use crate::bd::consultas::{presupuestos, trabajos};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{anio_de_madrid, fecha_de_madrid, leer_utc};

pub async fn crear_albaran(
    estado: &EstadoApp,
    actor: &Actor,
    trabajo: i64,
) -> Result<i64, ErrorApp> {
    cambiar(
        estado,
        actor,
        ("crear_albaran", "albaran"),
        move |tx, ahora| {
            let ficha = cargar_trabajo(tx, trabajo)?;
            let t = &ficha.trabajo;
            if t.estado == "anulado" {
                return Err(ErrorApp::Validacion("El trabajo está anulado.".into()));
            }
            let instante =
                leer_utc(ahora).ok_or_else(|| ErrorApp::Interno("fecha no válida".into()))?;
            let (anio, fecha) = (anio_de_madrid(instante), fecha_de_madrid(instante, 0));
            let numero = presupuestos::siguiente_numero(tx, "A", anio)?;
            let ventanas: Vec<_> = ficha
                .ventanas
                .iter()
                .map(|v| json!({ "orden": v.fila.orden, "descripcion": v.fila.descripcion }))
                .collect();
            let foto = json!({
                "codigo": format!("A-{anio}-{numero:04}"),
                "fecha": fecha,
                "trabajo": t.codigo(),
                "cliente": t.cliente_nombre,
                "direccion_montaje": t.direccion_montaje,
                "ventanas": ventanas,
            })
            .to_string();
            let huella = calcular_sha256(&foto);
            let id = insertar(tx, (anio, numero, trabajo), &fecha, (&foto, &huella))?;
            if t.estado != "entregado" {
                trabajos::cambiar_estado(tx, trabajo, "entregado")?;
            }
            Ok((id, id))
        },
    )
    .await
}

pub async fn buscar_albaran(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FilaAlbaran, ErrorApp> {
    crate::aplicacion::autorizar(actor, dominio::acceso::Accion::VerFacturas)?;
    estado
        .bd
        .leer(move |con| buscar(con, id)?.ok_or(ErrorApp::NoEncontrado))
        .await
}
