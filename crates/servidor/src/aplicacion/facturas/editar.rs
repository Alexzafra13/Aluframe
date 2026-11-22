//! Editar el borrador: datos, líneas libres y descuento. Los totales e IVA se calculan en un
//! único sitio (`dominio::precio::calcular_totales`), igual que en el presupuesto.

use dominio::factura::importe_de_linea;
use dominio::precio::{LineaParaTotales, calcular_totales};
use dominio::unidades::{Dinero, Porcentaje, PrecioUnitario};
use rusqlite::Connection;

use super::{cambiar, exigir_borrador};
use crate::aplicacion::Actor;
use crate::bd::consultas::facturas::{self, FilaLineaFactura};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub fn recalcular_totales(con: &Connection, id: i64) -> Result<(), ErrorApp> {
    let factura = facturas::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
    let lineas: Vec<LineaParaTotales> = facturas::lineas(con, id)?
        .iter()
        .map(|l| LineaParaTotales {
            importe: Dinero::desde_centimos(l.base_cent),
            tipo_iva: Porcentaje::desde_puntos_basicos(l.tipo_iva_bp),
        })
        .collect();
    let totales = calcular_totales(
        &lineas,
        Porcentaje::desde_puntos_basicos(factura.descuento_bp),
    )
    .ok_or_else(|| ErrorApp::Validacion("Los importes son demasiado grandes.".into()))?;
    let importes = (
        totales.base.centimos(),
        totales.iva.centimos(),
        totales.total.centimos(),
    );
    facturas::guardar_totales(con, id, importes)
}

pub struct LineaNueva {
    pub descripcion: String,
    pub cantidad_mil: i64,
    pub precio: PrecioUnitario,
    pub descuento: Porcentaje,
    pub iva: Porcentaje,
}

pub async fn anadir_linea(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    l: LineaNueva,
) -> Result<(), ErrorApp> {
    let descripcion: String = l.descripcion.trim().chars().take(300).collect();
    if descripcion.is_empty() || l.cantidad_mil == 0 {
        return Err(ErrorApp::Validacion(
            "Escribe la descripción y una cantidad distinta de cero.".into(),
        ));
    }
    let base = importe_de_linea(l.precio, l.cantidad_mil, l.descuento)
        .ok_or_else(|| ErrorApp::Validacion("El importe es demasiado grande.".into()))?;
    cambiar(estado, actor, ("linea_factura", "factura"), move |tx, _| {
        exigir_borrador(&facturas::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?)?;
        let fila = FilaLineaFactura {
            id: 0,
            orden: 0,
            descripcion,
            cantidad_mil: l.cantidad_mil,
            precio_unitario_pu: l.precio.diezmilesimas(),
            descuento_bp: l.descuento.puntos_basicos(),
            tipo_iva_bp: l.iva.puntos_basicos(),
            base_cent: base.centimos(),
        };
        facturas::insertar_linea(tx, id, &fila)?;
        recalcular_totales(tx, id)?;
        Ok(((), id))
    })
    .await
}

pub async fn quitar_linea(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
    linea: i64,
) -> Result<(), ErrorApp> {
    cambiar(estado, actor, ("linea_factura", "factura"), move |tx, _| {
        exigir_borrador(&facturas::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?)?;
        facturas::quitar_linea(tx, id, linea)?;
        recalcular_totales(tx, id)?;
        Ok(((), id))
    })
    .await
}

pub struct DatosDeFactura {
    pub fecha_operacion: Option<String>,
    pub vencimiento: Option<String>,
    pub notas: Option<String>,
    pub descuento: Porcentaje,
}

pub async fn guardar_datos(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
    d: DatosDeFactura,
) -> Result<(), ErrorApp> {
    for fecha in [&d.fecha_operacion, &d.vencimiento].into_iter().flatten() {
        let formato = time::macros::format_description!("[year]-[month]-[day]");
        time::Date::parse(fecha, formato)
            .map_err(|_| ErrorApp::Validacion("Revisa las fechas.".into()))?;
    }
    cambiar(estado, actor, ("datos_factura", "factura"), move |tx, _| {
        let datos = (
            d.fecha_operacion.as_deref(),
            d.vencimiento.as_deref(),
            d.notas.as_deref(),
            d.descuento.puntos_basicos(),
        );
        facturas::cambiar_datos(tx, (id, version), datos)?;
        recalcular_totales(tx, id)?;
        Ok(((), id))
    })
    .await
}

/// Un borrador sin número se puede descartar: no deja hueco en la numeración.
pub async fn descartar_borrador(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        actor,
        ("descartar_factura", "factura"),
        move |tx, _| {
            exigir_borrador(&facturas::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?)?;
            facturas::borrar_borrador(tx, id)?;
            Ok(((), id))
        },
    )
    .await
}
