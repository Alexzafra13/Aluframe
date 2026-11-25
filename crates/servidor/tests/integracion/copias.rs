use crate::comun;

use std::path::{Path, PathBuf};

use comun::Contexto;
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use servidor::aplicacion::Actor;
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::copias::discos::{CARPETA_DE_COPIAS, DiscosDeCopia};
use servidor::aplicacion::copias::hacer::hacer_copia;
use servidor::aplicacion::copias::pantalla::{aviso_de_copias, consultar_copias, restaurar_copia};
use servidor::aplicacion::datos::confirmar_importacion;
use servidor::errores::ErrorApp;
use servidor::estado::EstadoApp;
use time::Duration;

fn carpeta_temporal() -> PathBuf {
    let mut aleatorio = [0u8; 8];
    getrandom::fill(&mut aleatorio).unwrap();
    let sufijo: String = aleatorio.iter().map(|b| format!("{b:02x}")).collect();
    let carpeta = std::env::temp_dir().join(format!("alumframe-copias-{sufijo}"));
    std::fs::create_dir_all(&carpeta).unwrap();
    carpeta
}

/// Un taller con su carpeta de datos y un «pendrive» (otra carpeta) que se indica como disco.
async fn taller(pendrive: &Path) -> (Contexto, Actor) {
    let mut contexto = Contexto::nuevo().await;
    contexto.estado = contexto
        .estado
        .clone()
        .con_carpeta_copias(carpeta_temporal())
        .con_discos_de_copia(DiscosDeCopia {
            indicados: vec![pendrive.to_path_buf()],
            buscar_en_el_sistema: false,
        });
    let (_, actor) = contexto.con_propietario().await;
    (contexto, actor)
}

async fn nuevo_cliente(estado: &EstadoApp, actor: &Actor, nombre: &str) {
    let datos = validar_cliente(&EntradaDeCliente {
        nombre: nombre.into(),
        telefono: "612345678".into(),
        ..EntradaDeCliente::default()
    })
    .unwrap();
    crear_cliente(estado, actor, datos, vec![]).await.unwrap();
}

async fn clientes(estado: &EstadoApp) -> Vec<String> {
    estado
        .bd
        .leer(|con| {
            let mut consulta =
                con.prepare("SELECT nombre_o_razon_social FROM cliente ORDER BY id")?;
            let filas = consulta.query_map([], |f| f.get(0))?;
            Ok(filas.collect::<Result<_, _>>()?)
        })
        .await
        .unwrap()
}

fn copias_en(carpeta: &Path) -> Vec<String> {
    let mut nombres: Vec<String> = std::fs::read_dir(carpeta)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".alumframe"))
        .collect();
    nombres.sort();
    nombres
}

#[tokio::test]
async fn sin_disco_copia_en_el_equipo_y_avisa() {
    let pendrive = carpeta_temporal();
    let (contexto, actor) = taller(&pendrive).await;
    let registro = hacer_copia(&contexto.estado).await.unwrap();
    assert!(registro.ultima_completa.is_none());
    assert!(
        registro
            .aviso
            .as_deref()
            .unwrap()
            .contains("AlumFrame-copias")
    );
    assert!(aviso_de_copias(&contexto.estado).is_some());
    let estado = consultar_copias(&contexto.estado, &actor).unwrap();
    assert_eq!(estado.lugares.len(), 1, "solo el equipo");
    assert_eq!(estado.lugares[0].copias.len(), 1);
    assert_eq!(estado.sin_preparar, [pendrive.display().to_string()]);
}

#[tokio::test]
async fn con_el_disco_preparado_la_copia_queda_comprobada_en_el() {
    let pendrive = carpeta_temporal();
    let (contexto, _) = taller(&pendrive).await;
    contexto
        .estado
        .discos_de_copia()
        .preparar(&pendrive)
        .unwrap();
    let registro = hacer_copia(&contexto.estado).await.unwrap();
    assert_eq!(registro.aviso, None);
    assert!(registro.ultima_completa.is_some());
    // 08:00 UTC del 21-11-2026 son las 09:00 en Madrid.
    let en_disco = copias_en(&pendrive.join(CARPETA_DE_COPIAS));
    assert_eq!(en_disco, ["alumframe-2026-11-21-0900.alumframe"]);
    assert!(pendrive.join(CARPETA_DE_COPIAS).join("LEEME.txt").is_file());
    assert_eq!(aviso_de_copias(&contexto.estado), None);
}

#[tokio::test]
async fn el_disco_conserva_siete_diarias_y_las_semanales() {
    let pendrive = carpeta_temporal();
    let (contexto, _) = taller(&pendrive).await;
    contexto
        .estado
        .discos_de_copia()
        .preparar(&pendrive)
        .unwrap();
    for _ in 0..30 {
        hacer_copia(&contexto.estado).await.unwrap();
        contexto.reloj.avanzar(Duration::days(1));
    }
    let en_disco = copias_en(&pendrive.join(CARPETA_DE_COPIAS));
    assert!(en_disco.len() <= 11, "{en_disco:?}");
    assert!(en_disco.len() >= 10, "{en_disco:?}");
    assert_eq!(
        en_disco.last().unwrap(),
        "alumframe-2026-12-20-0900.alumframe"
    );
    assert!(en_disco.contains(&"alumframe-2026-12-14-0900.alumframe".to_owned()));
}

#[tokio::test]
async fn una_copia_del_disco_se_restaura_en_un_equipo_limpio() {
    let pendrive = carpeta_temporal();
    let (taller_viejo, actor) = taller(&pendrive).await;
    taller_viejo
        .estado
        .discos_de_copia()
        .preparar(&pendrive)
        .unwrap();
    nuevo_cliente(&taller_viejo.estado, &actor, "Ana Ruiz").await;
    nuevo_cliente(&taller_viejo.estado, &actor, "Construcciones Peña").await;
    hacer_copia(&taller_viejo.estado).await.unwrap();

    let (limpio, actor_nuevo) = taller(&pendrive).await;
    let disco = pendrive.join(CARPETA_DE_COPIAS).display().to_string();
    let nombre = "alumframe-2026-11-21-0900.alumframe";
    let clave = restaurar_copia(&limpio.estado, &actor_nuevo, (&disco, nombre))
        .await
        .unwrap();
    confirmar_importacion(&limpio.estado, &actor_nuevo, &clave)
        .await
        .unwrap();
    assert_eq!(
        clientes(&limpio.estado).await,
        ["Ana Ruiz", "Construcciones Peña"]
    );
}

#[tokio::test]
async fn solo_se_restaura_una_copia_de_la_lista() {
    let pendrive = carpeta_temporal();
    let (contexto, actor) = taller(&pendrive).await;
    hacer_copia(&contexto.estado).await.unwrap();
    for (disco, nombre) in [
        ("", "../alumframe.db"),
        ("/etc", "alumframe-2026-11-21-0900.alumframe"),
        ("", "alumframe-2026-11-22-0900.alumframe"),
    ] {
        let resultado = restaurar_copia(&contexto.estado, &actor, (disco, nombre)).await;
        assert!(
            matches!(resultado, Err(ErrorApp::NoEncontrado)),
            "{disco} {nombre}"
        );
    }
}
