//! La ventana que abre el operario en «Mi día», con la anterior y la siguiente del mismo
//! trabajo. Al abrirla, el aviso «Medida cambiada» se da por visto.

use dominio::acceso::Accion;

use super::consultar::{VentanaDeTrabajo, ventana_para};
use crate::aplicacion::{Actor, autorizar};
use crate::bd::consultas::trabajos::FilaTrabajo;
use crate::bd::consultas::ventanas_trabajo;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub struct VentanaAbierta {
    pub ventana: VentanaDeTrabajo,
    pub trabajo: FilaTrabajo,
    pub anterior: Option<i64>,
    pub siguiente: Option<i64>,
}

pub async fn abrir_ventana(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<VentanaAbierta, ErrorApp> {
    autorizar(actor, Accion::VerMiDia)?;
    let quien = actor.clone();
    let abierta = estado
        .bd
        .leer(move |con| {
            let (ventana, trabajo) = ventana_para(con, &quien, id)?;
            let mismas: Vec<i64> = ventanas_trabajo::de_trabajo(con, trabajo.id)?
                .into_iter()
                .filter(|v| ventana_para(con, &quien, v.id).is_ok())
                .map(|v| v.id)
                .collect();
            let posicion = mismas.iter().position(|v| *v == id).unwrap_or_default();
            Ok(VentanaAbierta {
                anterior: posicion.checked_sub(1).and_then(|p| mismas.get(p)).copied(),
                siguiente: mismas.get(posicion + 1).copied(),
                ventana,
                trabajo,
            })
        })
        .await?;
    if abierta.ventana.fila.medida_cambiada && !actor.rol.es_propietario() {
        estado
            .bd
            .escribir(move |tx| ventanas_trabajo::marcar_vista(tx, id))
            .await?;
    }
    Ok(abierta)
}
