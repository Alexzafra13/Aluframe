#![allow(clippy::unwrap_used, clippy::panic)]

//! Material a pedir del trabajo (ADR-0035): nace pendiente al aceptar, lo marcan el dueño o el
//! operario de la ventana, el Inicio avisa de lo que falta pedir y, si cambian las medidas, lo ya
//! pedido vuelve a pendiente.

mod comun;

use std::collections::BTreeMap;

use comun::Contexto;
use comun::presupuestos::{SERIE_COMPLETA, elegir_reglas, preparar_demo, ventana};
use dominio::acceso::Rol;
use dominio::taller::EstadoDeMaterial;
use dominio::unidades::Porcentaje;
use servidor::aplicacion::Actor;
use servidor::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;
use servidor::aplicacion::trabajos::acciones::asignar_trabajo;
use servidor::aplicacion::trabajos::consultar::{cargar_trabajo, listar_trabajos};
use servidor::aplicacion::trabajos::materiales::marcar_material;
use servidor::aplicacion::trabajos::medidas::{NuevasMedidas, modificar_medidas};
use servidor::bd::consultas::materiales::FilaMaterial;
use servidor::errores::ErrorApp;
use servidor::estado::EstadoApp;
use servidor::vistas::tablero;

/// Presupuesto aceptado con una ventana de 150 × 120 de la serie de prueba: nace el trabajo 1.
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

fn operario(id: i64) -> Actor {
    Actor {
        id,
        nombre: "Juan".into(),
        rol: Rol::Operario,
        token_csrf: String::new(),
        huella_sesion: String::new(),
        permisos: Default::default(),
        en_demostracion: false,
    }
}

async fn material(estado: &EstadoApp, actor: &Actor) -> FilaMaterial {
    let ficha = cargar_trabajo(estado, actor, 1).await.unwrap();
    ficha.ventanas[0].materiales[0].clone()
}

#[tokio::test]
async fn al_aceptar_el_cristal_queda_pendiente_de_pedir_y_el_inicio_lo_avisa() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let m = material(&estado, &actor).await;
    assert_eq!(
        (m.clase.as_str(), m.codigo.as_str(), m.cantidad),
        ("cristal", "DEMO_VIDRIO_4MM", 2)
    );
    assert_eq!((m.ancho_dmm, m.alto_dmm), (Some(6_800), Some(11_100)));
    assert_eq!(m.estado, "pendiente");
    let fichas = listar_trabajos(&estado, &actor, false).await.unwrap();
    let (_, avisos) = tablero(&fichas);
    assert!(
        avisos
            .iter()
            .any(|a| a.texto.ends_with("falta pedir cristal")),
        "{:?}",
        avisos.iter().map(|a| &a.texto).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn el_operario_de_la_ventana_lo_marca_pedido_y_otro_no_puede() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let juan = operario(contexto.con_operario(&actor).await);
    let m = material(&estado, &actor).await;
    let pedido = (
        EstadoDeMaterial::Pedido,
        Some("Cristalería, llega el martes".into()),
    );
    let ajeno = marcar_material(&estado, &juan, (m.id, m.version), pedido.clone()).await;
    assert!(
        matches!(ajeno, Err(ErrorApp::NoEncontrado)),
        "no es su ventana"
    );
    let ficha = cargar_trabajo(&estado, &actor, 1).await.unwrap();
    asignar_trabajo(
        &estado,
        &actor,
        (1, ficha.trabajo.version),
        (Some(juan.id), None),
    )
    .await
    .unwrap();
    marcar_material(&estado, &juan, (m.id, m.version), pedido.clone())
        .await
        .unwrap();
    let ahora = material(&estado, &actor).await;
    assert_eq!(ahora.estado, "pedido");
    assert_eq!(ahora.nota.as_deref(), Some("Cristalería, llega el martes"));
    assert!(ahora.usuario_nombre.is_some(), "queda quién lo marcó");
    let tarde = marcar_material(&estado, &actor, (m.id, m.version), pedido).await;
    assert!(
        matches!(tarde, Err(ErrorApp::Conflicto)),
        "con la versión vieja se avisa"
    );
    let fichas = listar_trabajos(&estado, &actor, false).await.unwrap();
    let (_, avisos) = tablero(&fichas);
    assert!(!avisos.iter().any(|a| a.texto.contains("falta pedir")));
}

#[tokio::test]
async fn si_cambian_las_medidas_lo_pedido_vuelve_a_pendiente_y_lo_dice() {
    let contexto = Contexto::nuevo().await;
    let (estado, actor) = con_trabajo(&contexto).await;
    let m = material(&estado, &actor).await;
    let pedido = (EstadoDeMaterial::Pedido, None);
    marcar_material(&estado, &actor, (m.id, m.version), pedido)
        .await
        .unwrap();
    let ventana = cargar_trabajo(&estado, &actor, 1).await.unwrap().ventanas[0]
        .fila
        .clone();
    let medidas = NuevasMedidas {
        ancho: "160".into(),
        alto: "120".into(),
        opciones: BTreeMap::new(),
        motivo: "El cliente midió mal".into(),
        reiniciar_pasos: false,
    };
    modificar_medidas(&estado, &actor, (ventana.id, ventana.version), medidas)
        .await
        .unwrap();
    let nuevo = material(&estado, &actor).await;
    assert_eq!(nuevo.ancho_dmm, Some(7_300), "160 / 2 - 7");
    assert_eq!(nuevo.estado, "pendiente");
    assert!(nuevo.nota.is_some_and(|n| n.contains("volver a pedirlo")));
}
