//! El Inicio del propietario: tablero de trabajos y avisos (PANTALLAS 3, design/02-inicio).

use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use dominio::acceso::{Accion, Rol};

use crate::aplicacion::copias::pantalla::aviso_de_copias;
use crate::aplicacion::facturas::listar_facturas;
use crate::aplicacion::presupuestos::listar_presupuestos;
use crate::aplicacion::stock::saldos;
use crate::aplicacion::trabajos::consultar::listar_trabajos;
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fecha_de_madrid;
use crate::http::extractores::ActorActual;
use crate::vistas::{
    AvisoDelTaller, MENU, PaginaInicio, estado_de_factura, euros, filas_de_stock, renderizar,
    tablero,
};

/// Facturas vencidas sin cobrar entera y presupuestos enviados sin respuesta (PANTALLAS 3).
async fn avisos_de_cobro(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<(Vec<AvisoDelTaller>, Vec<AvisoDelTaller>), ErrorApp> {
    let hoy = fecha_de_madrid(estado.ahora(), 0);
    let vencidos = listar_facturas(estado, actor)
        .await?
        .iter()
        .filter(|f| estado_de_factura(f, &hoy) == "Vencida")
        .map(|f| AvisoDelTaller {
            texto: format!(
                "{} · {}: faltan {}",
                f.codigo(),
                dominio::clientes::nombre_visible(f.cliente_id, f.cliente_nombre.as_deref()),
                euros(f.total_cent - f.cobrado_cent)
            ),
            enlace: format!("/facturas/{}", f.id),
        })
        .collect();
    let enviados = listar_presupuestos(estado, actor, Some("enviado".into()))
        .await?
        .iter()
        .map(|p| AvisoDelTaller {
            texto: format!(
                "{} · {}",
                p.codigo(),
                dominio::clientes::nombre_visible(p.cliente_id, p.cliente_nombre.as_deref())
            ),
            enlace: format!("/presupuestos/{}", p.id),
        })
        .collect();
    Ok((vencidos, enviados))
}

pub async fn mostrar_inicio(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    if actor.rol == Rol::Operario {
        return Ok(Redirect::to("/mi-dia").into_response());
    }
    autorizar(&actor, Accion::VerPanelDelPropietario)?;
    let fichas = listar_trabajos(&estado, &actor, false).await?;
    let (columnas, avisos) = tablero(&fichas);
    let stock_bajo = filas_de_stock(&saldos(&estado, &actor).await?)
        .into_iter()
        .filter(|f| f.bajo_minimo)
        .map(|f| AvisoDelTaller {
            texto: format!(
                "{} · {}: quedan {} (mínimo {})",
                f.referencia, f.descripcion, f.enteras, f.minimo
            ),
            enlace: format!("/stock/{}", f.articulo),
        })
        .collect();
    let (cobros_vencidos, sin_respuesta) = avisos_de_cobro(&estado, &actor).await?;
    let pagina = PaginaInicio {
        usuario: (&actor).into(),
        menu: MENU,
        seccion: "inicio",
        columnas,
        avisos,
        stock_bajo,
        cobros_vencidos,
        sin_respuesta,
        aviso_de_copias: aviso_de_copias(&estado),
    };
    Ok(renderizar(&pagina)?.into_response())
}
