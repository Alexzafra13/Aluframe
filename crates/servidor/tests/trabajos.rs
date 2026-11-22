#![allow(clippy::unwrap_used, clippy::panic)]

mod comun;

use comun::Contexto;
use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, leer, preparar_demo, ventana};
use dominio::acceso::Rol;
use dominio::taller::{AccionDePaso, EstadoDeTrabajo, PASOS_POR_DEFECTO};
use dominio::unidades::Porcentaje;
use servidor::aplicacion::Actor;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::importar_serie::importar_serie;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;
use servidor::aplicacion::trabajos::acciones::{
    anotar_falta, asignar_trabajo, cambiar_paso, cerrar_trabajo,
};
use servidor::aplicacion::trabajos::consultar::{
    cargar_trabajo, listar_trabajos, ventanas_de_operario,
};
use servidor::aplicacion::trabajos::crear::{guardar_pasos_del_taller, leer_pasos_del_taller};
use servidor::estado::EstadoApp;

/// Presupuesto aceptado con una línea de 2 ventanas de 150 × 120: nace el trabajo 1.
async fn con_trabajo(contexto: &Contexto) -> (EstadoApp, Actor) {
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_COMPLETA).await;
    elegir_reglas(&estado, &actor).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    let mut dos = ventana(150, 120);
    dos.cantidad = 2;
    let linea = (dos, Porcentaje::desde_puntos_basicos(2_100), false);
    guardar_ventana(&estado, &actor, (id, 0), linea)
        .await
        .unwrap();
    aceptar_presupuesto(&estado, &actor, id).await.unwrap();
    (estado, actor)
}

fn operario(id: i64) -> Actor {
    Actor {
        id,
        nombre: "Juan".into(),
        rol: Rol::Operario,
        token_csrf: String::new(),
        huella_sesion: String::new(),
    }
}

#[tokio::test]
async fn aceptar_crea_el_trabajo_con_una_ventana_por_unidad_y_sus_pasos() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let ficha = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    assert_eq!(ficha.trabajo.codigo(), "T-2026-0001");
    assert_eq!(ficha.trabajo.cliente_nombre, "Ana Ruiz");
    assert_eq!(ficha.ventanas.len(), 2, "cantidad 2 son 2 ventanas");
    assert_eq!(ficha.ventanas[0].pasos.len(), PASOS_POR_DEFECTO.len());
    assert_eq!(ficha.progreso(), (0, 2 * PASOS_POR_DEFECTO.len()));
    let congelada = &ficha.ventanas[0].congelado;
    assert_eq!((congelada.ancho_dmm, congelada.alto_dmm), (15_000, 12_000));
    assert_eq!(congelada.longitud_barra_dmm, Some(60_000));
    assert!(!congelada.resumen.piezas.is_empty());
    assert_eq!(
        listar_trabajos(&estado, &actor, false).await.unwrap().len(),
        1
    );
}

/// Criterio de aceptación de H4: cambiar una fórmula después de aceptar no altera el trabajo.
#[tokio::test]
async fn cambiar_la_serie_despues_de_aceptar_no_altera_el_trabajo() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let antes = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    let otra =
        leer(SERIE_COMPLETA).replace("descuento_hoja_vertical: 5", "descuento_hoja_vertical: 9");
    importar_serie(&estado, &actor, otra).await.unwrap();
    let despues = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    assert_eq!(
        antes.ventanas[0].fila.congelado_json,
        despues.ventanas[0].fila.congelado_json
    );
}

#[tokio::test]
async fn el_operario_no_toca_lo_que_no_tiene_asignado() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let juan = operario(contexto.con_operario(&actor).await);
    assert!(
        ventanas_de_operario(&estado, &juan)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        cargar_trabajo(&estado, &juan, 1).await.is_err(),
        "no ve la ficha ni importes"
    );
    let ficha = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    let paso = &ficha.ventanas[0].pasos[0];
    let ajeno = cambiar_paso(
        &estado,
        &juan,
        (paso.id, paso.version),
        AccionDePaso::Terminar,
    )
    .await;
    assert!(ajeno.is_err(), "sin asignar no puede tocarla");
}

#[tokio::test]
async fn el_operario_marca_sus_pasos_y_avisa_de_lo_que_falta() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let juan = operario(contexto.con_operario(&actor).await);
    let ficha = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    let asignacion = (Some(juan.id), None);
    asignar_trabajo(&estado, &actor, (1, ficha.trabajo.version), asignacion)
        .await
        .unwrap();
    assert_eq!(ventanas_de_operario(&estado, &juan).await.unwrap().len(), 2);
    let paso = &ficha.ventanas[0].pasos[0];
    let version = (paso.id, paso.version);
    cambiar_paso(&estado, &juan, version, AccionDePaso::Terminar)
        .await
        .unwrap();
    let tarde = cambiar_paso(&estado, &juan, version, AccionDePaso::Deshacer).await;
    assert!(
        tarde.is_err(),
        "con la versión vieja se avisa en lugar de pisar"
    );
    let nota = Some("Falta cristal".into());
    anotar_falta(&estado, &juan, ficha.ventanas[0].fila.id, nota)
        .await
        .unwrap();
    let ficha = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    assert_eq!(ficha.estado(), EstadoDeTrabajo::EnCurso);
    assert_eq!(ficha.progreso().0, 1);
    assert_eq!(ficha.falta_algo(), Some("Falta cristal"));
}

#[tokio::test]
async fn con_todos_los_pasos_hechos_queda_terminado_y_se_entrega_a_mano() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let ficha = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    for ventana in &ficha.ventanas {
        for paso in &ventana.pasos {
            cambiar_paso(
                &estado,
                &actor,
                (paso.id, paso.version),
                AccionDePaso::Terminar,
            )
            .await
            .unwrap();
        }
    }
    let ficha = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    assert_eq!(ficha.estado(), EstadoDeTrabajo::Terminado);
    cerrar_trabajo(&estado, &actor, 1, EstadoDeTrabajo::Entregado)
        .await
        .unwrap();
    assert!(
        listar_trabajos(&estado, &actor, false)
            .await
            .unwrap()
            .is_empty()
    );
    let otra_vez = cerrar_trabajo(&estado, &actor, 1, EstadoDeTrabajo::Anulado).await;
    assert!(otra_vez.is_err());
}

#[tokio::test]
async fn los_pasos_de_ajustes_valen_para_los_trabajos_nuevos() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let texto = "Cortar\nSoldar\n\ncortar\n".to_owned();
    guardar_pasos_del_taller(&contexto.estado, &actor, texto)
        .await
        .unwrap();
    let pasos = leer_pasos_del_taller(&contexto.estado).await.unwrap();
    assert_eq!(pasos, ["Cortar", "Soldar"]);
}
