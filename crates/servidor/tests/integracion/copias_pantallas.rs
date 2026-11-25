use crate::comun;

use std::path::{Path, PathBuf};

use comun::{Contexto, destino, formulario, get, post, texto};
use servidor::aplicacion::Actor;
use servidor::aplicacion::copias::discos::DiscosDeCopia;
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};

fn carpeta_temporal() -> PathBuf {
    let carpeta = std::env::temp_dir().join(format!(
        "alumframe-copias-pantallas-{}",
        servidor::autenticacion::generar_token().unwrap()
    ));
    std::fs::create_dir_all(&carpeta).unwrap();
    carpeta
}

async fn preparado(pendrive: &Path) -> (Contexto, String, Actor) {
    let mut contexto = Contexto::nuevo().await;
    contexto.estado = contexto
        .estado
        .clone()
        .con_carpeta_copias(carpeta_temporal())
        .con_discos_de_copia(DiscosDeCopia {
            indicados: vec![pendrive.to_path_buf()],
            buscar_en_el_sistema: false,
        });
    let (token, actor) = contexto.con_propietario().await;
    (contexto, token, actor)
}

#[tokio::test]
async fn sin_disco_avisa_en_el_inicio_y_se_prepara_desde_ajustes() {
    let pendrive = carpeta_temporal();
    let (contexto, token, actor) = preparado(&pendrive).await;
    let csrf = formulario(&[("_csrf", &actor.token_csrf)]);
    let hecha = contexto
        .pedir(post("/ajustes/copias/ahora", Some(&token), &csrf))
        .await;
    assert_eq!(destino(&hecha), Some("/ajustes/copias?aviso=hecha"));
    let inicio = texto(contexto.pedir(get("/", Some(&token))).await).await;
    assert!(inicio.contains("No hay ningún disco de copias conectado"));

    let raiz = pendrive.display().to_string();
    let pagina = texto(contexto.pedir(get("/ajustes/copias", Some(&token))).await).await;
    assert!(pagina.contains("Usar este disco para las copias") && pagina.contains(&raiz));
    let preparar = formulario(&[("_csrf", &actor.token_csrf), ("raiz", &raiz)]);
    let preparado = contexto
        .pedir(post("/ajustes/copias/preparar", Some(&token), &preparar))
        .await;
    assert_eq!(destino(&preparado), Some("/ajustes/copias?aviso=preparado"));
    contexto
        .pedir(post("/ajustes/copias/ahora", Some(&token), &csrf))
        .await;
    let inicio = texto(contexto.pedir(get("/", Some(&token))).await).await;
    assert!(
        !inicio.contains("Ver las copias"),
        "sin aviso con la copia en el disco"
    );
    let pagina = texto(contexto.pedir(get("/ajustes/copias", Some(&token))).await).await;
    assert!(pagina.contains("En el disco") && pagina.contains("21/11/2026 09:00"));
}

#[tokio::test]
async fn restaurar_lleva_a_revisar_la_copia_y_no_acepta_rutas_inventadas() {
    let pendrive = carpeta_temporal();
    let (contexto, token, actor) = preparado(&pendrive).await;
    let csrf = formulario(&[("_csrf", &actor.token_csrf)]);
    contexto
        .pedir(post("/ajustes/copias/ahora", Some(&token), &csrf))
        .await;
    let elegir = |nombre: &str| {
        formulario(&[
            ("_csrf", &actor.token_csrf),
            ("disco", ""),
            ("nombre", nombre),
        ])
    };
    let restaurar = contexto
        .pedir(post(
            "/ajustes/copias/restaurar",
            Some(&token),
            &elegir("alumframe-2026-11-21-0900.alumframe"),
        ))
        .await;
    let ir_a = destino(&restaurar).unwrap();
    assert!(ir_a.starts_with("/ajustes/datos/importar/"), "{ir_a}");
    let revisar = texto(contexto.pedir(get(ir_a, Some(&token))).await).await;
    assert!(revisar.contains("Importar"));
    let inventada = contexto
        .pedir(post(
            "/ajustes/copias/restaurar",
            Some(&token),
            &elegir("../alumframe.db"),
        ))
        .await;
    assert_eq!(inventada.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn la_campana_cuenta_los_avisos_y_el_operario_no_la_ve() {
    let pendrive = carpeta_temporal();
    let (contexto, token, actor) = preparado(&pendrive).await;
    let inicio = texto(contexto.pedir(get("/", Some(&token))).await).await;
    assert!(inicio.contains("hx-get=\"/avisos/campana\""));
    let vacia = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(vacia.contains("Avisos: ninguno") && vacia.contains("Todo en orden"));

    let csrf = formulario(&[("_csrf", &actor.token_csrf)]);
    contexto
        .pedir(post("/ajustes/copias/ahora", Some(&token), &csrf))
        .await;
    let campana = texto(contexto.pedir(get("/avisos/campana", Some(&token))).await).await;
    assert!(campana.contains("Avisos: 1") && campana.contains("Copias de seguridad"));
    assert!(campana.contains("href=\"/ajustes/copias\""));

    let id = contexto.con_operario(&actor).await;
    let solicitud = SolicitudDeEntrada {
        usuario_id: id,
        clave: comun::PIN.into(),
        ip: "10.0.0.7".into(),
        duracion: servidor::aplicacion::sesion::DuracionDeSesion::Normal,
    };
    let ResultadoDeEntrada::Correcta {
        token: operario, ..
    } = entrar(&contexto.estado, solicitud).await.unwrap()
    else {
        panic!("el operario no pudo entrar");
    };
    let prohibida = contexto
        .pedir(get("/avisos/campana", Some(&operario)))
        .await;
    assert!(prohibida.status().is_client_error());
}
