//! Lo que queda por cobrar: las rectificativas emitidas cuentan y las facturas antiguas no se
//! pierden de los avisos aunque haya muchas.

use crate::comun;

use comun::facturas::{con_cliente, emitida_libre};
use comun::{Contexto, get, texto};
use servidor::aplicacion::Actor;
use servidor::aplicacion::facturas::crear::rectificativa;
use servidor::aplicacion::facturas::emitir::emitir_factura;
use servidor::aplicacion::facturas::listar_por_cobrar;
use servidor::estado::EstadoApp;

/// Una factura libre de 80 € + IVA que venció antes de hoy (el reloj de las pruebas está en
/// noviembre de 2026), ya emitida.
async fn emitida_y_vencida(estado: &EstadoApp, actor: &Actor) -> i64 {
    emitida_libre(estado, actor, (None, Some("2026-10-31"))).await
}

/// Fallo corregido: una factura anulada con una rectificativa emitida seguía «Vencida», con su
/// importe pendiente, en la lista de cobros pendientes y en la campana del Inicio.
#[tokio::test]
async fn una_factura_anulada_con_rectificativa_ya_no_se_reclama() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = con_cliente(&contexto).await;
    let estado = &contexto.estado;
    let original = emitida_y_vencida(estado, &actor).await;
    let campana = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(campana.contains("F-2026-0001"), "vencida, se avisa");

    let r = rectificativa(estado, &actor, original, "Factura duplicada".into())
        .await
        .unwrap();
    let borrador = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(
        borrador.contains("F-2026-0001"),
        "un borrador de rectificativa aún no anula nada"
    );
    emitir_factura(estado, &actor, r, true).await.unwrap();

    assert!(listar_por_cobrar(estado, &actor).await.unwrap().is_empty());
    let campana = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(!campana.contains("F-2026-0001"), "{campana}");
    let pendientes = texto(
        contexto
            .pedir(get("/facturas?pendientes=si", Some(&token)))
            .await,
    )
    .await;
    assert!(!pendientes.contains("F-2026-0001"));
    let ficha = texto(contexto.pedir(get("/facturas/1", Some(&token))).await).await;
    assert!(
        ficha.contains("Anulada") && ficha.contains("R-2026-0001"),
        "{ficha}"
    );
}

/// Fallo corregido: los avisos salían de la lista de las 500 últimas facturas; una antigua sin
/// cobrar desaparecía de ellos.
#[tokio::test]
async fn una_factura_antigua_sin_cobrar_sigue_en_los_avisos_aunque_haya_muchas() {
    let contexto = Contexto::nuevo().await;
    let (token, actor) = con_cliente(&contexto).await;
    emitida_y_vencida(&contexto.estado, &actor).await;
    let muchas = |tx: &rusqlite::Transaction| {
        for numero in 2..=600 {
            tx.execute(
                "INSERT INTO factura (serie, anio, numero, cliente_id, estado, fecha_emision, \
                 total_cent, snapshot_json, creado_en) VALUES ('F', 2026, ?1, 1, 'emitida', \
                 '2026-11-01', 0, '{}', '2026-11-01T10:00:00Z')",
                [numero],
            )?;
        }
        Ok(())
    };
    contexto.estado.bd.escribir(muchas).await.unwrap();
    let por_cobrar = listar_por_cobrar(&contexto.estado, &actor).await.unwrap();
    assert_eq!(por_cobrar.len(), 1, "las de 0 € no deben nada");
    let campana = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(campana.contains("F-2026-0001"), "{campana}");
}
