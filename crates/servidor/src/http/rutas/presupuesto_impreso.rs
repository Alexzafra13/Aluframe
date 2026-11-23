use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use dominio::unidades::Dinero;
use serde_json::Value;

use crate::aplicacion::ajustes_presupuesto::leer_ajustes_de_presupuesto;
use crate::aplicacion::datos_de_empresa::CabeceraDeEmpresa;
use crate::aplicacion::lineas_de_presupuesto::{ivas_de, totales_de};
use crate::aplicacion::logo_empresa::leer_logo_de_empresa;
use crate::aplicacion::presupuestos::{FichaDePresupuesto, cargar_presupuesto};
use crate::aplicacion::resumen_de_linea::ResumenDeVentana;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fechas_de_situacion;
use crate::http::extractores::ActorActual;
use crate::vistas::{LineaImpresa, PaginaPresupuestoImpreso, euros, fecha_corta, renderizar};

/// Lo congelado al aceptar manda sobre la ficha actual del cliente.
fn cliente_de_la_copia(copia: &str) -> Option<Vec<String>> {
    let valor: Value = serde_json::from_str(copia).ok()?;
    let cliente = &valor["cliente"];
    let campo = |c: &str| cliente[c].as_str().map(str::to_owned);
    let mut datos = vec![campo("nombre").unwrap_or_default()];
    datos.extend(con_etiquetas(
        campo("nif"),
        campo("telefono"),
        campo("email"),
    ));
    let fiscal = cliente["direccion_fiscal"].as_array().into_iter().flatten();
    datos.extend(fiscal.filter_map(|v| v.as_str().map(str::to_owned)));
    Some(datos)
}

fn con_etiquetas(
    nif: Option<String>,
    telefono: Option<String>,
    email: Option<String>,
) -> Vec<String> {
    [
        nif.map(|n| format!("NIF {n}")),
        telefono.map(|t| format!("Tel. {t}")),
        email,
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn datos_del_cliente(ficha: &FichaDePresupuesto) -> Vec<String> {
    if let Some(datos) = ficha
        .presupuesto
        .copia_json
        .as_deref()
        .and_then(cliente_de_la_copia)
    {
        return datos;
    }
    let c = &ficha.cliente;
    let mut datos = vec![c.nombre_visible()];
    datos.extend(con_etiquetas(
        c.nif.clone(),
        c.telefono.clone(),
        c.email.clone(),
    ));
    let poblacion = [&c.fiscal_cp, &c.fiscal_localidad]
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let poblacion = match (&c.fiscal_provincia, poblacion.is_empty()) {
        (Some(p), false) => format!("{poblacion} ({p})"),
        (Some(p), true) => p.clone(),
        (None, _) => poblacion,
    };
    let direccion: Vec<String> = c
        .fiscal_calle
        .iter()
        .cloned()
        .chain(Some(poblacion).filter(|p| !p.is_empty()))
        .collect();
    if !direccion.is_empty() {
        datos.push(direccion.join(", "));
    }
    datos
}

fn lineas_impresas(ficha: &FichaDePresupuesto) -> Vec<LineaImpresa> {
    ficha
        .lineas
        .iter()
        .map(|l| LineaImpresa {
            orden: l.orden,
            es_ventana: l.tipo == "ventana",
            titulo: tramos(&l.descripcion).0,
            detalles: tramos(&l.descripcion).1,
            cantidad: dominio::unidades::formatear_es(i128::from(l.cantidad_mil), 3, 0),
            precio: euros(l.precio_cent),
            dibujo: ResumenDeVentana::leer(l.resultado_json.as_deref()).dibujo,
        })
        .collect()
}

/// «Corredera 2 hojas · Aluprom 21 · 120 × 150 cm» se imprime con el título y un renglón por dato.
fn tramos(descripcion: &str) -> (String, Vec<String>) {
    let mut partes = descripcion.split(" · ").map(str::to_owned);
    (partes.next().unwrap_or_default(), partes.collect())
}

/// Hoja A4 sin costes ni márgenes (SPEC, apartado 10).
pub async fn mostrar(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let ficha = cargar_presupuesto(&estado, &actor, id).await?;
    let textos = leer_ajustes_de_presupuesto(&estado).await?.textos;
    let empresa = CabeceraDeEmpresa::desde(&textos, leer_logo_de_empresa(&estado).await?);
    let p = &ficha.presupuesto;
    let totales = totales_de(&ficha.lineas, p.descuento_bp);
    let descuento = if totales.descuento == Dinero::CERO {
        String::new()
    } else {
        format!("−{}", totales.descuento.formatear())
    };
    let direccion = ficha
        .direcciones
        .iter()
        .find(|d| Some(d.id) == p.direccion_montaje_id);
    let pagina = PaginaPresupuestoImpreso {
        id,
        codigo: p.codigo(),
        fecha: fecha_corta(p.creado_en.get(..10).unwrap_or_default()),
        validez: p
            .validez_hasta
            .as_deref()
            .map(fecha_corta)
            .unwrap_or_default(),
        empresa,
        cliente: datos_del_cliente(&ficha),
        direccion_montaje: direccion.map(|d| d.datos.resumen()).unwrap_or_default(),
        lineas: lineas_impresas(&ficha),
        bases: ivas_de(&totales),
        descuento,
        base: totales.base.formatear(),
        total: euros(p.total_cent),
        condiciones: p.condiciones.clone().unwrap_or_default(),
        provisional: !p.completo,
        estado: ficha
            .situacion(&fechas_de_situacion(estado.ahora()))
            .nombre(),
    };
    Ok(renderizar(&pagina)?.into_response())
}
