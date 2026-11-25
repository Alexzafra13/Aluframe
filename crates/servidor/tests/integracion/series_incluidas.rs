use crate::comun;

use comun::Contexto;
use servidor::aplicacion::importar_serie::importar_serie;
use servidor::aplicacion::series::listar_series;
use servidor::aplicacion::series_incluidas::{
    CargaDeSerie, cargar_series_incluidas, listar_series_incluidas,
};

fn codigos_cargados(cargas: &[CargaDeSerie]) -> Vec<String> {
    cargas
        .iter()
        .map(|carga| match carga {
            CargaDeSerie::Cargada { codigo, numero } => format!("{codigo} v{numero}"),
            otra => panic!("se esperaba cargada: {otra:?}"),
        })
        .collect()
}

#[test]
fn incluye_todas_las_series_de_data_series() {
    let carpeta = format!("{}/../../data/series", env!("CARGO_MANIFEST_DIR"));
    let mut en_disco: Vec<String> = std::fs::read_dir(carpeta)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".yaml"))
        .collect();
    en_disco.sort();
    let incluidas: Vec<String> = listar_series_incluidas()
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(incluidas, en_disco);
}

#[tokio::test]
async fn una_instalacion_nueva_trae_las_series_sin_importarlas_a_mano() {
    let contexto = Contexto::nuevo().await;
    let cargas = cargar_series_incluidas(&contexto.estado).await.unwrap();
    assert_eq!(
        codigos_cargados(&cargas),
        vec![
            "ALUPROM_21 v1",
            "MOSQUITERA_ENROLLABLE v1",
            "PERSIANA_PARED v1",
            "SOPENA_V94 v1"
        ]
    );
    let (_, actor) = contexto.con_propietario().await;
    let series = listar_series(&contexto.estado, &actor).await.unwrap();
    let codigos: Vec<&str> = series.iter().map(|s| s.serie.codigo.as_str()).collect();
    assert_eq!(
        codigos,
        vec![
            "ALUPROM_21",
            "MOSQUITERA_ENROLLABLE",
            "PERSIANA_PARED",
            "SOPENA_V94"
        ]
    );
}

#[tokio::test]
async fn arrancar_otra_vez_no_duplica_versiones() {
    let contexto = Contexto::nuevo().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let otra_vez = cargar_series_incluidas(&contexto.estado).await.unwrap();
    assert!(
        otra_vez
            .iter()
            .all(|c| matches!(c, CargaDeSerie::SinCambios { .. })),
        "{otra_vez:?}"
    );
}

#[tokio::test]
async fn no_pisa_una_version_importada_a_mano_si_la_incluida_no_cambia() {
    let contexto = Contexto::nuevo().await;
    let (_, actor) = contexto.con_propietario().await;
    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let (_, aluprom) = listar_series_incluidas().remove(0);
    let retocada = format!("{aluprom}\n# retocada a mano\n");
    let (_, numero) = importar_serie(&contexto.estado, &actor, retocada)
        .await
        .unwrap();
    assert_eq!(numero, 2);

    cargar_series_incluidas(&contexto.estado).await.unwrap();
    let series = listar_series(&contexto.estado, &actor).await.unwrap();
    let vigente = series
        .iter()
        .find(|s| s.serie.codigo == "ALUPROM_21")
        .unwrap();
    assert_eq!(vigente.numero_version, 2);
}
