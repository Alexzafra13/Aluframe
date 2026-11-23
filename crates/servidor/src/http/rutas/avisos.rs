//! Lo que necesita atención del propietario, reunido una vez para el Inicio y la campana de la
//! barra superior: copias, «falta algo» de los operarios, cobros vencidos, stock bajo y
//! presupuestos sin respuesta.

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use dominio::acceso::Accion;

use crate::aplicacion::apuntes::apuntes_para_el_dueno;
use crate::aplicacion::copias::pantalla::aviso_de_copias;
use crate::aplicacion::facturas::listar_por_cobrar;
use crate::aplicacion::presupuestos::listar_presupuestos;
use crate::aplicacion::stock::saldos;
use crate::aplicacion::trabajos::consultar::listar_trabajos;
use crate::aplicacion::{Actor, autorizar};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fecha_de_madrid;
use crate::http::extractores::ActorActual;
use crate::vistas::{
    AvisoDelTaller, AvisosReunidos, Campana, avisos_de_apuntes, estado_de_factura, euros,
    filas_de_stock, renderizar, tablero,
};

/// `falta_algo` llega hecho porque el Inicio ya ha cargado los trabajos para el tablero.
pub async fn reunir_avisos(
    estado: &EstadoApp,
    actor: &Actor,
    falta_algo: Vec<AvisoDelTaller>,
) -> Result<AvisosReunidos, ErrorApp> {
    autorizar(actor, Accion::VerPanelDelPropietario)?;
    let (cobros_vencidos, sin_respuesta) = avisos_de_cobro(estado, actor).await?;
    Ok(AvisosReunidos {
        copias: aviso_de_copias(estado),
        apuntes: avisos_de_apuntes(&apuntes_para_el_dueno(estado, actor).await?),
        falta_algo,
        cobros_vencidos,
        stock_bajo: stock_bajo(estado, actor).await?,
        sin_respuesta,
    })
}

async fn stock_bajo(estado: &EstadoApp, actor: &Actor) -> Result<Vec<AvisoDelTaller>, ErrorApp> {
    Ok(filas_de_stock(&saldos(estado, actor).await?)
        .into_iter()
        .filter(|f| f.bajo_minimo)
        .map(|f| AvisoDelTaller {
            texto: format!(
                "{} · {}: quedan {} (mínimo {})",
                f.referencia, f.descripcion, f.enteras, f.minimo
            ),
            enlace: format!("/stock/{}", f.articulo),
        })
        .collect())
}

/// Facturas vencidas sin cobrar entera y presupuestos enviados sin respuesta (PANTALLAS 3).
async fn avisos_de_cobro(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<(Vec<AvisoDelTaller>, Vec<AvisoDelTaller>), ErrorApp> {
    let hoy = fecha_de_madrid(estado.ahora(), 0);
    let vencidos = listar_por_cobrar(estado, actor)
        .await?
        .iter()
        .filter(|f| estado_de_factura(f, &hoy) == "Vencida")
        .map(|f| AvisoDelTaller {
            texto: format!(
                "{} · {}: faltan {}",
                f.codigo(),
                dominio::clientes::nombre_visible(f.cliente_id, f.cliente_nombre.as_deref()),
                euros(f.importes().pendiente().centimos())
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

/// La campana de la barra superior: se pide con HTMX al cargar cada página.
pub async fn mostrar_campana(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
) -> Result<Response, ErrorApp> {
    let fichas = listar_trabajos(&estado, &actor, false).await?;
    let (_, falta_algo) = tablero(&fichas);
    let avisos = reunir_avisos(&estado, &actor, falta_algo).await?;
    Ok(renderizar(&Campana::desde(avisos))?.into_response())
}
