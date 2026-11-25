use crate::comun;

use comun::Contexto;
use comun::presupuestos::{SERIE_COMPLETA, SERIE_DEMO, elegir_reglas, preparar_demo, ventana};
use dominio::precio::{LineaParaTotales, calcular_totales};
use dominio::unidades::{Dinero, Porcentaje, PrecioUnitario};
use servidor::aplicacion::aceptar_presupuesto::{aceptar_presupuesto, motivos_para_no_aceptar};
use servidor::aplicacion::clientes::borrar_cliente;
use servidor::aplicacion::lineas_de_presupuesto::*;
use servidor::aplicacion::presupuestos::*;
use servidor::aplicacion::tarifas::{deshacer_tarifa, listar_tarifas};

fn iva(bp: i64) -> Porcentaje {
    Porcentaje::desde_puntos_basicos(bp)
}

fn libre() -> LineaLibre {
    LineaLibre {
        descripcion: "Instalación".into(),
        cantidad_mil: 1_000,
        precio_unitario: PrecioUnitario::desde_texto_es("80").unwrap(),
        descuento: Porcentaje::CERO,
        iva: iva(2_100),
    }
}

#[tokio::test]
async fn sin_reglas_elegidas_la_linea_sale_incompleta_y_dice_por_que() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_DEMO).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    guardar_ventana(
        &estado,
        &actor,
        (id, 0),
        (ventana(150, 120), iva(2_100), false),
    )
    .await
    .unwrap();
    let ficha = cargar_presupuesto(&estado, &actor, id).await.unwrap();
    assert_eq!(ficha.presupuesto.codigo(), "P-2026-0001");
    assert!(ficha.lineas[0].incompleta);
    let faltan = &ficha.lineas[0].faltan_json;
    assert!(
        faltan.contains("elige cómo se cobran los perfiles"),
        "{faltan}"
    );
    assert!(faltan.contains("elige el margen"), "{faltan}");
    assert!(
        faltan.contains("Serie de demostración"),
        "una serie demo nunca da precio final"
    );
    let hoy = ("2026-11-21".to_owned(), "2026-05-23".to_owned());
    assert!(!motivos_para_no_aceptar(&ficha, &hoy).is_empty());
    assert!(aceptar_presupuesto(&estado, &actor, id).await.is_err());
}

/// Presupuesto del cliente 1 con una ventana por medida y las reglas de precio elegidas.
async fn presupuesto_con(
    estado: &servidor::estado::EstadoApp,
    actor: &servidor::aplicacion::Actor,
    (serie, medidas): (&str, &[(i64, i64)]),
) -> i64 {
    preparar_demo(estado, actor, serie).await;
    elegir_reglas(estado, actor).await;
    let id = crear_presupuesto(estado, actor, 1, None).await.unwrap();
    for (ancho, alto) in medidas {
        let linea = (ventana(*ancho, *alto), iva(2_100), false);
        guardar_ventana(estado, actor, (id, 0), linea)
            .await
            .unwrap();
    }
    id
}

/// Criterio de aceptación de H3 (parte del servidor): 3 ventanas y una línea libre, con el
/// total igual al que da el dominio con los importes de las líneas.
#[tokio::test]
async fn tres_ventanas_y_una_linea_libre_suman_el_total_del_dominio() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    let medidas = [(150, 120), (120, 100), (200, 140)];
    let id = presupuesto_con(&estado, &actor, (SERIE_DEMO, &medidas)).await;
    guardar_libre(&estado, &actor, (id, 0), libre())
        .await
        .unwrap();
    let ficha = cargar_presupuesto(&estado, &actor, id).await.unwrap();
    assert_eq!(ficha.lineas.len(), 4);
    let demo = r#"["Serie de demostración con datos ficticios."]"#;
    let ventanas: Vec<_> = ficha
        .lineas
        .iter()
        .filter(|l| l.tipo == "ventana")
        .collect();
    assert!(
        ventanas.iter().all(|l| l.faltan_json == demo),
        "solo falta lo propio de la demo"
    );
    assert!(ficha.lineas.iter().all(|l| l.precio_cent > 0));
    let lineas: Vec<_> = ficha
        .lineas
        .iter()
        .map(|l| LineaParaTotales {
            importe: Dinero::desde_centimos(l.precio_cent),
            tipo_iva: iva(l.tipo_iva_bp),
        })
        .collect();
    let totales = calcular_totales(&lineas, Porcentaje::CERO).unwrap();
    assert_eq!(ficha.presupuesto.total_cent, totales.total.centimos());
    assert!(
        !ficha.presupuesto.completo,
        "con datos ficticios no se puede aceptar"
    );
}

#[tokio::test]
async fn aceptar_congela_y_bloquea_cambios_tarifa_y_borrado() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    let id = presupuesto_con(&estado, &actor, (SERIE_COMPLETA, &[(150, 120)])).await;
    cambiar_estado(&estado, &actor, id, "enviado")
        .await
        .unwrap();
    assert!(
        guardar_libre(&estado, &actor, (id, 0), libre())
            .await
            .is_err(),
        "lo enviado no se edita"
    );
    aceptar_presupuesto(&estado, &actor, id).await.unwrap();
    let ficha = cargar_presupuesto(&estado, &actor, id).await.unwrap();
    assert_eq!(ficha.presupuesto.estado, "aceptado");
    let copia = ficha.presupuesto.copia_json.unwrap();
    assert!(copia.contains("\"codigo\":\"P-2026-0001\"") && copia.contains("Ana Ruiz"));
    assert!(nueva_version(&estado, &actor, id, None).await.is_err());
    let tarifa = listar_tarifas(&estado, &actor).await.unwrap()[0].id;
    let error = deshacer_tarifa(&estado, &actor, tarifa).await.unwrap_err();
    assert!(error.to_string().contains("presupuesto aceptado"));
    let error = borrar_cliente(&estado, &actor, 1).await.unwrap_err();
    assert!(error.to_string().contains("tiene presupuestos"));
}

#[tokio::test]
async fn una_version_nueva_copia_las_lineas_y_vuelve_a_borrador() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &actor, SERIE_DEMO).await;
    let id = crear_presupuesto(&estado, &actor, 1, None).await.unwrap();
    guardar_libre(&estado, &actor, (id, 0), libre())
        .await
        .unwrap();
    guardar_libre(&estado, &actor, (id, 0), libre())
        .await
        .unwrap();
    cambiar_estado(&estado, &actor, id, "enviado")
        .await
        .unwrap();
    nueva_version(&estado, &actor, id, Some("Cambia el color".into()))
        .await
        .unwrap();
    let ficha = cargar_presupuesto(&estado, &actor, id).await.unwrap();
    assert_eq!(
        (
            ficha.presupuesto.numero_version,
            ficha.presupuesto.estado.as_str()
        ),
        (2, "borrador")
    );
    assert_eq!(ficha.lineas.len(), 2);
    assert_eq!(ficha.versiones.len(), 2);
    borrar_linea(&estado, &actor, (id, 1)).await.unwrap();
    let ficha = cargar_presupuesto(&estado, &actor, id).await.unwrap();
    assert_eq!(
        ficha.lineas.iter().map(|l| l.orden).collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(ficha.presupuesto.total_cent, 9_680, "80 € + 21 %");
}
