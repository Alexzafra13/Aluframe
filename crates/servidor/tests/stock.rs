#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use comun::Contexto;
use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use dominio::stock::TipoDeMovimiento;
use dominio::unidades::Porcentaje;
use servidor::aplicacion::Actor;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;
use servidor::aplicacion::stock::terminar::{RetalConfirmado, terminar_orden};
use servidor::aplicacion::stock::{MovimientoManual, registrar, saldos};
use servidor::aplicacion::trabajos::acciones::repetir_pieza;
use servidor::aplicacion::trabajos::orden::cargar_orden;
use servidor::estado::EstadoApp;

/// Trabajo 1 con una ventana de 150 × 120 de la serie ficticia «validada».
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

async fn articulo(estado: &EstadoApp, referencia: &'static str) -> i64 {
    estado
        .bd
        .leer(move |con| {
            Ok(con.query_row(
                "SELECT id FROM articulo WHERE referencia = ?1",
                [referencia],
                |f| f.get(0),
            )?)
        })
        .await
        .unwrap()
}

async fn entrada(
    estado: &EstadoApp,
    actor: &Actor,
    referencia: &'static str,
    (cantidad, largo): (&str, &str),
) {
    let m = MovimientoManual {
        articulo: articulo(estado, referencia).await,
        largo_cm: largo.into(),
        cantidad: cantidad.into(),
        ..MovimientoManual::default()
    };
    registrar(estado, actor, TipoDeMovimiento::Entrada, m)
        .await
        .unwrap();
}

async fn saldo(estado: &EstadoApp, actor: &Actor, referencia: &str, largo: Option<i64>) -> i64 {
    saldos(estado, actor)
        .await
        .unwrap()
        .iter()
        .filter(|s| s.referencia == referencia && s.longitud_dmm == largo)
        .map(|s| s.cantidad_mil)
        .sum()
}

async fn barras_nuevas(estado: &EstadoApp, actor: &Actor, perfil: &str) -> i64 {
    let orden = cargar_orden(estado, actor, 1).await.unwrap();
    let grupo = orden.plan.iter().find(|g| g.perfil == perfil).unwrap();
    i64::try_from(grupo.plan.as_ref().unwrap().barras_nuevas()).unwrap()
}

/// Criterio de aceptación de H5: terminar una orden descuenta exactamente el material optimizado.
#[tokio::test]
async fn terminar_la_orden_descuenta_el_plan_y_los_accesorios_una_sola_vez() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    for referencia in [
        "DEMO_MARCO_LATERAL",
        "DEMO_HOJA_VERTICAL",
        "DEMO_HOJA_HORIZONTAL",
    ] {
        entrada(&estado, &actor, referencia, ("10", "")).await;
    }
    entrada(&estado, &actor, "DEMO_RUEDA", ("10", "")).await;
    let lateral = barras_nuevas(&estado, &actor, "DEMO_MARCO_LATERAL").await;
    terminar_orden(&estado, &actor, 1, vec![]).await.unwrap();
    assert_eq!(
        saldo(&estado, &actor, "DEMO_MARCO_LATERAL", None).await,
        10_000 - lateral * 1_000
    );
    assert_eq!(
        saldo(&estado, &actor, "DEMO_RUEDA", None).await,
        6_000,
        "4 ruedas por ventana"
    );
    assert_eq!(
        saldo(&estado, &actor, "DEMO_KIT", None).await,
        -1_000,
        "sin inventario queda en negativo"
    );
    let otra_vez = terminar_orden(&estado, &actor, 1, vec![]).await;
    assert!(
        otra_vez.is_err(),
        "una orden terminada no vuelve a descontar"
    );
    assert_eq!(saldo(&estado, &actor, "DEMO_RUEDA", None).await, 6_000);
}

/// Criterio de aceptación de H5: con retales del mismo perfil y color, el plan los usa antes de
/// abrir barras nuevas; al terminar sale ese retal y entra el que se confirma.
#[tokio::test]
async fn el_plan_usa_el_retal_del_stock_y_al_terminar_entra_el_retal_confirmado() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    entrada(&estado, &actor, "DEMO_HOJA_HORIZONTAL", ("1", "300")).await;
    let orden = cargar_orden(&estado, &actor, 1).await.unwrap();
    let grupo = orden
        .plan
        .iter()
        .find(|g| g.perfil == "DEMO_HOJA_HORIZONTAL")
        .unwrap();
    let plan = grupo.plan.as_ref().unwrap();
    assert_eq!(
        plan.barras_nuevas(),
        0,
        "4 × 73 cm caben en el retal de 300"
    );
    assert!(plan.barras[0].retal.is_some());
    let grupo_indice = orden
        .enlace
        .perfiles
        .keys()
        .position(|(p, _)| p == "DEMO_HOJA_HORIZONTAL")
        .unwrap();
    let confirmado = RetalConfirmado {
        grupo: grupo_indice,
        largo_dmm: 6_000,
    };
    terminar_orden(&estado, &actor, 1, vec![confirmado])
        .await
        .unwrap();
    assert_eq!(
        saldo(&estado, &actor, "DEMO_HOJA_HORIZONTAL", Some(30_000)).await,
        0
    );
    assert_eq!(
        saldo(&estado, &actor, "DEMO_HOJA_HORIZONTAL", Some(6_000)).await,
        1_000
    );
}

#[tokio::test]
async fn una_pieza_repetida_entra_en_el_plan() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let piezas = |o: &servidor::aplicacion::trabajos::orden::OrdenDeTaller| {
        let g = o
            .plan
            .iter()
            .find(|g| g.perfil == "DEMO_HOJA_VERTICAL")
            .unwrap();
        g.plan.as_ref().unwrap().piezas().count()
    };
    let antes = piezas(&cargar_orden(&estado, &actor, 1).await.unwrap());
    let ventana = estado
        .bd
        .leer(
            |con| Ok(con.query_row("SELECT id FROM ventana_trabajo", [], |f| f.get::<_, i64>(0))?),
        )
        .await
        .unwrap();
    repetir_pieza(&estado, &actor, (ventana, 4, 1), "Rayón".into())
        .await
        .unwrap();
    assert_eq!(
        piezas(&cargar_orden(&estado, &actor, 1).await.unwrap()),
        antes + 1
    );
    let sin_motivo = repetir_pieza(&estado, &actor, (ventana, 4, 1), " ".into()).await;
    assert!(sin_motivo.is_err());
}

/// Criterio de aceptación de H5: el stock nunca se edita a mano sin crear un ajuste.
#[tokio::test]
async fn los_ajustes_llevan_motivo_no_dejan_negativo_y_el_libro_no_se_edita() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    entrada(&estado, &actor, "DEMO_RUEDA", ("2", "")).await;
    let id = articulo(&estado, "DEMO_RUEDA").await;
    let ajuste = |cantidad: &str, motivo: &str| MovimientoManual {
        articulo: id,
        cantidad: cantidad.into(),
        motivo: motivo.into(),
        ..MovimientoManual::default()
    };
    assert!(
        registrar(&estado, &actor, TipoDeMovimiento::Ajuste, ajuste("-1", ""))
            .await
            .is_err()
    );
    assert!(
        registrar(
            &estado,
            &actor,
            TipoDeMovimiento::Ajuste,
            ajuste("-3", "Recuento")
        )
        .await
        .is_err()
    );
    registrar(
        &estado,
        &actor,
        TipoDeMovimiento::Ajuste,
        ajuste("-1", "Recuento"),
    )
    .await
    .unwrap();
    assert_eq!(saldo(&estado, &actor, "DEMO_RUEDA", None).await, 1_000);
    let editar = estado
        .bd
        .escribir(|tx| Ok(tx.execute("UPDATE movimiento_stock SET cantidad_mil = 99", [])?))
        .await;
    assert!(editar.is_err(), "el libro de stock solo admite añadir");
}
