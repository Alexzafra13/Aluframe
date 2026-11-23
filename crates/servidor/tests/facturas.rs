#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use comun::Contexto;
use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use dominio::factura::{EstadoDeCobro, ImportesDeFactura, estado_de_cobro, importe_de_linea};
use dominio::unidades::{Dinero, Porcentaje, PrecioUnitario};
use servidor::aplicacion::Actor;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::facturas::albaranes::crear_albaran;
use servidor::aplicacion::facturas::cargar_factura;
use servidor::aplicacion::facturas::cobros::{
    DatosDeCobro, DestinoDelCobro, anular_cobro, registrar_cobro,
};
use servidor::aplicacion::facturas::crear::{factura_de_trabajo, factura_libre, rectificativa};
use servidor::aplicacion::facturas::editar::{LineaNueva, anadir_linea, descartar_borrador};
use servidor::aplicacion::facturas::emitir::emitir_factura;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;
use servidor::aplicacion::trabajos::consultar::cargar_trabajo;
use servidor::estado::EstadoApp;

/// Trabajo 1 aceptado (cliente 1, sin NIF).
async fn con_trabajo(contexto: &Contexto) -> (EstadoApp, Actor) {
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    let linea = (
        ventana(150, 120),
        Porcentaje::desde_puntos_basicos(2_100),
        false,
    );
    guardar_ventana(&estado, &actor, (id, 0), linea)
        .await
        .unwrap();
    aceptar_presupuesto(&estado, &actor, id).await.unwrap();
    (estado, actor)
}

fn cobro(importe: &str, medio: &str) -> DatosDeCobro {
    DatosDeCobro {
        fecha: "2026-11-22".into(),
        importe: importe.into(),
        medio: medio.into(),
        nota: String::new(),
    }
}

fn linea(descripcion: &str, euros: &str) -> LineaNueva {
    LineaNueva {
        descripcion: descripcion.into(),
        cantidad_mil: 1_000,
        precio: PrecioUnitario::desde_texto_es(euros).unwrap(),
        descuento: Porcentaje::CERO,
        iva: Porcentaje::desde_puntos_basicos(2_100),
    }
}

async fn emitida_con_senal(estado: &EstadoApp, actor: &Actor) -> i64 {
    let senal = cobro("100", "bizum");
    registrar_cobro(estado, actor, DestinoDelCobro::Trabajo(1), senal)
        .await
        .unwrap();
    let id = factura_de_trabajo(estado, actor, 1).await.unwrap();
    emitir_factura(estado, actor, id, true).await.unwrap();
    id
}

#[tokio::test]
async fn la_factura_del_trabajo_copia_el_importe_aceptado_y_avisa_sin_nif() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let importe = cargar_trabajo(&estado, &actor, 1)
        .await
        .unwrap()
        .trabajo
        .importe_cent;
    let id = factura_de_trabajo(&estado, &actor, 1).await.unwrap();
    assert_eq!(
        factura_de_trabajo(&estado, &actor, 1).await.unwrap(),
        id,
        "no se duplica"
    );
    let borrador = cargar_factura(&estado, &actor, id).await.unwrap();
    assert_eq!(borrador.factura.total_cent, importe, "el importe aceptado");
    assert_eq!(borrador.factura.numero, None, "un borrador no tiene número");
    assert!(
        emitir_factura(&estado, &actor, id, false).await.is_err(),
        "sin NIF avisa antes"
    );
    assert_eq!(
        emitir_factura(&estado, &actor, id, true).await.unwrap(),
        "F-2026-0001"
    );
}

#[tokio::test]
async fn la_senal_cuenta_al_emitir_y_los_cobros_parciales_la_dejan_pagada() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let id = emitida_con_senal(&estado, &actor).await;
    let emitida = cargar_factura(&estado, &actor, id).await.unwrap().factura;
    assert_eq!(
        emitida.cobrado_cent, 10_000,
        "la señal cuenta como pagado a cuenta"
    );
    let importes = ImportesDeFactura {
        total: Dinero::desde_centimos(emitida.total_cent),
        rectificado: Dinero::CERO,
        cobrado: Dinero::desde_centimos(10_000),
    };
    assert_eq!(
        estado_de_cobro(importes, (None, "2026-11-22")),
        EstadoDeCobro::Parcial
    );
    let resto = emitida.total_cent - 10_000;
    let texto = format!("{},{:02}", resto / 100, resto % 100);
    registrar_cobro(
        &estado,
        &actor,
        DestinoDelCobro::Factura(id),
        cobro(&texto, "tarjeta"),
    )
    .await
    .unwrap();
    let pagada = cargar_factura(&estado, &actor, id).await.unwrap().factura;
    assert_eq!(pagada.cobrado_cent, pagada.total_cent);
    let senal = registrar_cobro(
        &estado,
        &actor,
        DestinoDelCobro::Trabajo(1),
        cobro("5", "efectivo"),
    )
    .await;
    assert!(senal.is_err(), "con factura emitida, el cobro va en ella");
}

async fn libre_con_linea(estado: &EstadoApp, actor: &Actor) -> i64 {
    let id = factura_libre(estado, actor, 1).await.unwrap();
    anadir_linea(estado, actor, id, linea("Instalación", "80"))
        .await
        .unwrap();
    id
}

/// Criterio de aceptación de H6: la numeración no tiene huecos (un borrador descartado no gasta
/// número; se numera al emitir, en el orden en que se emiten).
#[tokio::test]
async fn la_numeracion_no_tiene_huecos() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let descartada = libre_con_linea(&estado, &actor).await;
    let primera = libre_con_linea(&estado, &actor).await;
    let segunda = libre_con_linea(&estado, &actor).await;
    descartar_borrador(&estado, &actor, descartada)
        .await
        .unwrap();
    assert_eq!(
        emitir_factura(&estado, &actor, segunda, true)
            .await
            .unwrap(),
        "F-2026-0001"
    );
    assert_eq!(
        emitir_factura(&estado, &actor, primera, true)
            .await
            .unwrap(),
        "F-2026-0002"
    );
}

/// Criterios de aceptación de H6: no se puede editar una emitida y la rectificativa referencia
/// a la original.
#[tokio::test]
async fn una_emitida_no_se_edita_y_se_corrige_con_una_rectificativa() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let original = libre_con_linea(&estado, &actor).await;
    emitir_factura(&estado, &actor, original, true)
        .await
        .unwrap();
    let editar = anadir_linea(&estado, &actor, original, linea("Otra", "10")).await;
    assert!(editar.is_err(), "una emitida no se edita");
    let sql = move |tx: &rusqlite::Transaction| {
        Ok(tx.execute(
            "UPDATE factura SET total_cent = 1 WHERE id = ?1",
            [original],
        )?)
    };
    assert!(
        estado.bd.escribir(sql).await.is_err(),
        "ni siquiera desde la base de datos"
    );
    assert!(
        rectificativa(&estado, &actor, original, " ".into())
            .await
            .is_err(),
        "pide motivo"
    );
    let r = rectificativa(&estado, &actor, original, "Precio mal".into())
        .await
        .unwrap();
    let ficha = cargar_factura(&estado, &actor, r).await.unwrap();
    assert_eq!(ficha.factura.rectifica_factura_id, Some(original));
    let total = cargar_factura(&estado, &actor, original)
        .await
        .unwrap()
        .factura
        .total_cent;
    assert_eq!(ficha.factura.total_cent, -total);
    assert_eq!(
        emitir_factura(&estado, &actor, r, true).await.unwrap(),
        "R-2026-0001"
    );
}

#[tokio::test]
async fn un_cobro_se_anula_con_motivo_y_el_albaran_entrega_el_trabajo() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    registrar_cobro(
        &estado,
        &actor,
        DestinoDelCobro::Trabajo(1),
        cobro("50", "efectivo"),
    )
    .await
    .unwrap();
    let mal = registrar_cobro(
        &estado,
        &actor,
        DestinoDelCobro::Trabajo(1),
        cobro("0", "efectivo"),
    )
    .await;
    assert!(mal.is_err(), "importe mayor que cero");
    assert!(
        anular_cobro(&estado, &actor, 1, String::new())
            .await
            .is_err()
    );
    assert_eq!(
        anular_cobro(&estado, &actor, 1, "Me equivoqué".into())
            .await
            .unwrap(),
        (Some(1), None)
    );
    let albaran = crear_albaran(&estado, &actor, 1).await.unwrap();
    assert_eq!(albaran, 1);
    let trabajo = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    assert_eq!(trabajo.trabajo.estado, "entregado");
}

/// Fallo corregido: al facturar un trabajo, el precio unitario de una ventana con descuento se
/// deducía del importe ya descontado y la hoja decía «Precio 900, Dto. 10 %, Importe 900».
#[tokio::test]
async fn la_factura_del_trabajo_cuadra_precio_descuento_e_importe() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    let mut con_descuento = ventana(150, 120);
    con_descuento.cantidad = 2;
    con_descuento.descuento = Porcentaje::desde_puntos_basicos(1_000);
    let iva = Porcentaje::desde_puntos_basicos(2_100);
    guardar_ventana(&estado, &actor, (id, 0), (con_descuento, iva, false))
        .await
        .unwrap();
    aceptar_presupuesto(&estado, &actor, id).await.unwrap();
    let factura = factura_de_trabajo(&estado, &actor, 1).await.unwrap();
    let ficha = cargar_factura(&estado, &actor, factura).await.unwrap();
    let l = &ficha.lineas[0];
    let precio = PrecioUnitario::desde_diezmilesimas(l.precio_unitario_pu);
    let descuento = Porcentaje::desde_puntos_basicos(l.descuento_bp);
    assert_eq!(l.descuento_bp, 1_000);
    assert_eq!(
        importe_de_linea(precio, l.cantidad_mil, descuento),
        Some(Dinero::desde_centimos(l.base_cent)),
        "precio × cantidad − descuento = importe"
    );
}
