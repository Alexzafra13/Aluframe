//! Caducidad de los presupuestos (ADR-0027): 30 días por defecto, renovar con los precios de hoy
//! y archivar medio año después sin borrar nada.

use crate::comun;

use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use comun::{Contexto, destino, formulario, get, post, texto};
use dominio::unidades::Porcentaje;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::{cambiar_estado, cargar_presupuesto, crear_presupuesto};
use time::Duration;

/// Un presupuesto enviado el 21/11/2026, completo y listo para aceptar.
async fn enviado(contexto: &Contexto) -> i64 {
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    let iva = Porcentaje::desde_puntos_basicos(2_100);
    guardar_ventana(&estado, &actor, (id, 0), (ventana(150, 120), iva, false))
        .await
        .unwrap();
    cambiar_estado(&estado, &actor, id, "enviado")
        .await
        .unwrap();
    id
}

#[tokio::test]
async fn caduca_a_los_30_dias_y_no_se_acepta_hasta_renovarlo() {
    let contexto = Contexto::nuevo().await;
    let id = enviado(&contexto).await;
    let (token, actor) = contexto.con_propietario_existente().await;
    let ficha = cargar_presupuesto(&contexto.estado, &actor, id)
        .await
        .unwrap();
    assert_eq!(
        ficha.presupuesto.validez_hasta.as_deref(),
        Some("2026-12-21")
    );
    let campana = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(campana.contains("P-2026-0001"), "enviado sin respuesta");

    contexto.reloj.avanzar(Duration::days(31));
    let (token, actor) = contexto.con_propietario_existente().await;
    let pagina = texto(contexto.pedir(get("/presupuestos/1", Some(&token))).await).await;
    assert!(pagina.contains("Caducado") && pagina.contains("Renovar"));
    assert!(pagina.contains("su validez terminó el 21/12/2026"));
    let estado = contexto.estado.clone().con_datos_demo(true);
    assert!(aceptar_presupuesto(&estado, &actor, id).await.is_err());
    let caducados = texto(
        contexto
            .pedir(get("/presupuestos?estado=caducado", Some(&token)))
            .await,
    )
    .await;
    assert!(caducados.contains("P-2026-0001"));
    let enviados = get("/presupuestos?estado=enviado", Some(&token));
    assert!(
        !texto(contexto.pedir(enviados).await)
            .await
            .contains("P-2026-0001")
    );
    let campana = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(
        !campana.contains("P-2026-0001"),
        "caducado, ya no espera respuesta"
    );
}

#[tokio::test]
async fn renovar_pone_precios_y_validez_de_hoy_en_una_version_nueva() {
    let contexto = Contexto::nuevo().await;
    let id = enviado(&contexto).await;
    contexto.reloj.avanzar(Duration::days(31));
    let (token, actor) = contexto.con_propietario_existente().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    let csrf = formulario(&[("_csrf", &actor.token_csrf)]);
    let renovar = contexto
        .pedir(post("/presupuestos/1/renovar", Some(&token), &csrf))
        .await;
    assert_eq!(destino(&renovar), Some("/presupuestos/1?aviso=renovado"));
    let renovado = cargar_presupuesto(&estado, &actor, id).await.unwrap();
    assert_eq!(renovado.presupuesto.estado, "borrador");
    assert_eq!(
        renovado.presupuesto.validez_hasta.as_deref(),
        Some("2027-01-21"),
        "30 días desde hoy"
    );
    assert_eq!(
        renovado.versiones.len(),
        2,
        "lo enviado queda como versión 1"
    );
    aceptar_presupuesto(&estado, &actor, id).await.unwrap();
}

#[tokio::test]
async fn medio_anio_despues_se_archiva_sin_borrarse() {
    let contexto = Contexto::nuevo().await;
    enviado(&contexto).await;
    contexto.reloj.avanzar(Duration::days(30 + 183));
    let (token, _) = contexto.con_propietario_existente().await;
    let lista = |ruta: &'static str| {
        let peticion = get(ruta, Some(&token));
        async { texto(contexto.pedir(peticion).await).await }
    };
    assert!(!lista("/presupuestos").await.contains("P-2026-0001"));
    assert!(
        !lista("/presupuestos?estado=caducado")
            .await
            .contains("P-2026-0001")
    );
    let archivados = lista("/presupuestos?estado=archivado").await;
    assert!(archivados.contains("P-2026-0001") && archivados.contains("Archivado"));
    let cliente = lista("/clientes/1").await;
    assert!(
        cliente.contains("P-2026-0001"),
        "sigue en la ficha del cliente"
    );
    let ficha = lista("/presupuestos/1").await;
    assert!(ficha.contains("Archivado") && ficha.contains("Renovar"));
}

#[tokio::test]
async fn un_borrador_caducado_no_se_envia_sin_renovarlo() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    contexto.reloj.avanzar(Duration::days(31));
    let error = cambiar_estado(&estado, &actor, id, "enviado").await;
    assert!(error.is_err(), "hay que renovarlo antes");
    let (token, _) = contexto.con_propietario_existente().await;
    let ficha = texto(contexto.pedir(get("/presupuestos/1", Some(&token))).await).await;
    assert!(!ficha.contains("Marcar como enviado"));
}
