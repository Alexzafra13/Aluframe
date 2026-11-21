use dominio::precio::{LineaParaTotales, calcular_totales};
use dominio::unidades::{Dinero, Porcentaje, PrecioUnitario, Unidad};
use rusqlite::Connection;

use super::Actor;
use super::ajustes_presupuesto::{self, AjustesDePresupuesto};
use super::calculo_de_ventana::{CalculoDeVentana, EntradaDeVentana, calcular};
use super::presupuestos::{cambiar, exigir_borrador};
use super::resumen_de_linea::ResumenDeVentana;
use super::taller::{self, AjustesDeTaller};
use crate::bd::consultas::lineas_presupuesto::{self, FilaLinea};
use crate::bd::consultas::presupuestos;
use crate::bd::consultas::versiones_presupuesto::{self, Totales};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

fn ajustes(con: &Connection) -> Result<(AjustesDeTaller, AjustesDePresupuesto), ErrorApp> {
    Ok((
        taller::leer_de_la_base(con)?,
        ajustes_presupuesto::leer_de_la_base(con)?,
    ))
}

/// El cálculo en vivo del editor: no guarda nada.
pub async fn calcular_ventana_sin_guardar(
    estado: &EstadoApp,
    actor: &Actor,
    entrada: EntradaDeVentana,
) -> Result<CalculoDeVentana, ErrorApp> {
    super::autorizar(actor, dominio::acceso::Accion::GestionarClientes)?;
    estado
        .bd
        .leer(move |con| {
            let (taller, presupuesto) = ajustes(con)?;
            calcular(con, &entrada, (&taller, &presupuesto))
        })
        .await
}

/// Totales de la versión vigente: el único sitio que los guarda (con `dominio::precio`).
pub fn recalcular_totales(con: &Connection, presupuesto_id: i64) -> Result<(), ErrorApp> {
    let presupuesto = presupuestos::buscar(con, presupuesto_id)?.ok_or(ErrorApp::NoEncontrado)?;
    let lineas = lineas_presupuesto::listar(con, presupuesto.version_id)?;
    let para_totales: Vec<LineaParaTotales> = lineas
        .iter()
        .map(|l| LineaParaTotales {
            importe: Dinero::desde_centimos(l.precio_cent),
            tipo_iva: Porcentaje::desde_puntos_basicos(l.tipo_iva_bp),
        })
        .collect();
    let totales = calcular_totales(
        &para_totales,
        Porcentaje::desde_puntos_basicos(presupuesto.descuento_bp),
    )
    .ok_or_else(|| ErrorApp::Validacion("El total es demasiado grande.".into()))?;
    let completo = !lineas.is_empty() && lineas.iter().all(|l| !l.incompleta && !l.revisar_limites);
    let guardar = Totales {
        base_cent: totales.base.centimos(),
        iva_cent: totales.iva.centimos(),
        total_cent: totales.total.centimos(),
        completo,
    };
    versiones_presupuesto::guardar_totales(con, presupuesto.version_id, &guardar)
}

fn fila_de_ventana(
    version_id: i64,
    orden: i64,
    entrada: &EntradaDeVentana,
    (calculo, iva, limites_revisados): (&CalculoDeVentana, Porcentaje, bool),
) -> Result<FilaLinea, ErrorApp> {
    let resumen = ResumenDeVentana::from(calculo);
    let unidad = calculo.serie.unidad;
    let dmm = |v| {
        dominio::unidades::Longitud::desde_decimal(v, unidad)
            .map(|l| l.decimas_de_mm())
            .ok()
    };
    Ok(FilaLinea {
        version_id,
        orden,
        tipo: "ventana".into(),
        descripcion: calculo.descripcion.clone(),
        cantidad_mil: i64::from(entrada.cantidad) * 1_000,
        serie_version_id: Some(entrada.serie_version_id),
        tipologia: Some(entrada.tipologia.clone()),
        ancho_dmm: dmm(entrada.ancho),
        alto_dmm: dmm(entrada.alto),
        acabado_id: entrada.acabado_id,
        vidrio_articulo_id: entrada.vidrio_articulo_id,
        opciones_json: a_json(&entrada.opciones)?,
        resultado_json: Some(a_json(&resumen)?),
        faltan_json: a_json(&calculo.precio.faltan)?,
        incompleta: !calculo.precio.es_final(),
        revisar_limites: !calculo.resultado.fuera_de_limites.is_empty() && !limites_revisados,
        tarifas_json: a_json(&calculo.tarifas)?,
        coste_cent: calculo.precio.coste_unitario.centimos(),
        mano_de_obra_cent: entrada.mano_de_obra.centimos(),
        descuento_bp: entrada.descuento.puntos_basicos(),
        tipo_iva_bp: iva.puntos_basicos(),
        precio_unitario_pu: None,
        precio_cent: calculo.precio.precio.centimos(),
        ..FilaLinea::default()
    })
}

pub fn a_json<T: serde::Serialize>(valor: &T) -> Result<String, ErrorApp> {
    serde_json::to_string(valor).map_err(|e| ErrorApp::Interno(format!("json: {e}")))
}

/// Guarda una ventana (orden 0 = nueva) y devuelve su número de línea.
pub async fn guardar_ventana(
    estado: &EstadoApp,
    actor: &Actor,
    (id, orden): (i64, i64),
    (entrada, iva, limites_revisados): (EntradaDeVentana, Porcentaje, bool),
) -> Result<i64, ErrorApp> {
    cambiar(estado, actor, "guardar_linea", move |tx, _| {
        let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        exigir_borrador(&presupuesto)?;
        let (taller, ajustes) = ajustes(tx)?;
        let calculo = calcular(tx, &entrada, (&taller, &ajustes))?;
        let fila = fila_de_ventana(
            presupuesto.version_id,
            orden,
            &entrada,
            (&calculo, iva, limites_revisados),
        )?;
        let orden = lineas_presupuesto::guardar(tx, &fila)?;
        recalcular_totales(tx, id)?;
        Ok((orden, id))
    })
    .await
}

pub struct LineaLibre {
    pub descripcion: String,
    pub cantidad_mil: i64,
    pub precio_unitario: PrecioUnitario,
    pub descuento: Porcentaje,
    pub iva: Porcentaje,
}

pub async fn guardar_libre(
    estado: &EstadoApp,
    actor: &Actor,
    (id, orden): (i64, i64),
    linea: LineaLibre,
) -> Result<i64, ErrorApp> {
    let bruto = linea
        .precio_unitario
        .importe(linea.cantidad_mil)
        .ok_or_else(|| ErrorApp::Validacion("El importe es demasiado grande.".into()))?;
    let menos = Porcentaje::desde_puntos_basicos(-linea.descuento.puntos_basicos());
    let precio = dominio::precio::aplicar_porcentaje(bruto, menos)
        .ok_or_else(|| ErrorApp::Validacion("El importe es demasiado grande.".into()))?;
    cambiar(estado, actor, "guardar_linea", move |tx, _| {
        let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        exigir_borrador(&presupuesto)?;
        let fila = FilaLinea {
            version_id: presupuesto.version_id,
            orden,
            tipo: "libre".into(),
            descripcion: linea.descripcion,
            cantidad_mil: linea.cantidad_mil,
            opciones_json: "{}".into(),
            faltan_json: "[]".into(),
            tarifas_json: "[]".into(),
            descuento_bp: linea.descuento.puntos_basicos(),
            tipo_iva_bp: linea.iva.puntos_basicos(),
            precio_unitario_pu: Some(linea.precio_unitario.diezmilesimas()),
            precio_cent: precio.centimos(),
            ..FilaLinea::default()
        };
        let orden = lineas_presupuesto::guardar(tx, &fila)?;
        recalcular_totales(tx, id)?;
        Ok((orden, id))
    })
    .await
}

pub async fn borrar_linea(
    estado: &EstadoApp,
    actor: &Actor,
    (id, orden): (i64, i64),
) -> Result<(), ErrorApp> {
    cambiar(estado, actor, "borrar_linea", move |tx, _| {
        let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        exigir_borrador(&presupuesto)?;
        lineas_presupuesto::borrar(tx, presupuesto.version_id, orden)?;
        recalcular_totales(tx, id)?;
        Ok(((), id))
    })
    .await
}

/// Entrada de una línea ya guardada, para recalcularla o editarla.
pub fn entrada_de(linea: &FilaLinea, unidad: Unidad) -> Option<EntradaDeVentana> {
    let medida = |dmm: Option<i64>| {
        dominio::unidades::Longitud::desde_decimas_de_mm(dmm?)
            .a_decimal(unidad)
            .ok()
    };
    Some(EntradaDeVentana {
        serie_version_id: linea.serie_version_id?,
        tipologia: linea.tipologia.clone()?,
        ancho: medida(linea.ancho_dmm)?,
        alto: medida(linea.alto_dmm)?,
        opciones: serde_json::from_str(&linea.opciones_json).unwrap_or_default(),
        acabado_id: linea.acabado_id,
        vidrio_articulo_id: linea.vidrio_articulo_id,
        cantidad: u32::try_from(linea.cantidad_mil / 1_000)
            .unwrap_or(1)
            .max(1),
        mano_de_obra: Dinero::desde_centimos(linea.mano_de_obra_cent),
        descuento: Porcentaje::desde_puntos_basicos(linea.descuento_bp),
    })
}

/// Vuelve a calcular todas las ventanas con la tarifa vigente (botón «Recalcular precios»).
pub async fn recalcular_precios(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<(), ErrorApp> {
    cambiar(estado, actor, "recalcular_presupuesto", move |tx, _| {
        let presupuesto = presupuestos::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        exigir_borrador(&presupuesto)?;
        let (taller, ajustes) = ajustes(tx)?;
        for linea in lineas_presupuesto::listar(tx, presupuesto.version_id)?
            .iter()
            .filter(|l| l.tipo == "ventana")
        {
            let unidad = Unidad::Cm;
            let Some(entrada) = entrada_de(linea, unidad) else {
                continue;
            };
            let calculo = calcular(tx, &entrada, (&taller, &ajustes))?;
            let revisados = !linea.revisar_limites;
            let fila = fila_de_ventana(
                presupuesto.version_id,
                linea.orden,
                &entrada,
                (
                    &calculo,
                    Porcentaje::desde_puntos_basicos(linea.tipo_iva_bp),
                    revisados,
                ),
            )?;
            lineas_presupuesto::guardar(tx, &fila)?;
        }
        recalcular_totales(tx, id)?;
        Ok(((), id))
    })
    .await
}

pub fn unidad_de_serie(con: &Connection, linea: &FilaLinea) -> Result<Unidad, ErrorApp> {
    let id = linea.serie_version_id.ok_or(ErrorApp::NoEncontrado)?;
    let version =
        crate::bd::consultas::series::buscar_version(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
    Ok(super::series::reconstruir(version)?.serie.unidad)
}

/// Totales con su base y cuota por tipo de IVA, para mostrarlos (los guardados salen de aquí).
pub fn totales_de(lineas: &[FilaLinea], descuento_bp: i64) -> dominio::precio::Totales {
    let importes: Vec<LineaParaTotales> = lineas
        .iter()
        .map(|l| LineaParaTotales {
            importe: Dinero::desde_centimos(l.precio_cent),
            tipo_iva: Porcentaje::desde_puntos_basicos(l.tipo_iva_bp),
        })
        .collect();
    calcular_totales(&importes, Porcentaje::desde_puntos_basicos(descuento_bp)).unwrap_or_default()
}

/// (tipo, base, cuota) ya formateados.
pub fn ivas_de(totales: &dominio::precio::Totales) -> Vec<(String, String, String)> {
    totales
        .bases
        .iter()
        .map(|b| {
            (
                b.tipo.formatear(false),
                b.base.formatear(),
                b.cuota.formatear(),
            )
        })
        .collect()
}
