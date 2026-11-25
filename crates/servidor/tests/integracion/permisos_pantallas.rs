//! El operario por la web, con y sin permisos (ADR-0024): ve lo que el dueño le deja y nunca
//! costes, márgenes ni acciones de gestión, aunque escriba la dirección a mano.

use crate::comun;

use axum::http::StatusCode;
use comun::presupuestos::{SERIE_DEMO, elegir_reglas, preparar_demo, ventana};
use comun::{Contexto, PIN, formulario, get, post, texto};
use dominio::acceso::{Permiso, Permisos};
use dominio::unidades::Porcentaje;
use servidor::aplicacion::Actor;
use servidor::aplicacion::entrar::{ResultadoDeEntrada, SolicitudDeEntrada, entrar};
use servidor::aplicacion::lineas_de_presupuesto::guardar_ventana;
use servidor::aplicacion::presupuestos::crear_presupuesto;
use servidor::aplicacion::sesion::DuracionDeSesion;
use servidor::aplicacion::usuarios::guardar_permisos;

/// Un taller con un presupuesto de una ventana y un operario con los permisos indicados.
async fn taller_con_operario(permisos: &[Permiso]) -> (Contexto, String, String) {
    let contexto = Contexto::nuevo().await;
    let (_, dueno) = contexto.con_propietario().await;
    let estado = contexto.estado.clone().con_datos_demo(true);
    preparar_demo(&estado, &dueno, SERIE_DEMO).await;
    elegir_reglas(&estado, &dueno).await;
    let id = crear_presupuesto(&estado, &dueno, 1, None).await.unwrap();
    let linea = (
        ventana(150, 120),
        Porcentaje::desde_puntos_basicos(2_100),
        false,
    );
    guardar_ventana(&estado, &dueno, (id, 0), linea)
        .await
        .unwrap();
    let operario = contexto.con_operario(&dueno).await;
    let elegidos = Permisos::con(permisos.iter().copied());
    guardar_permisos(&estado, &dueno, operario, elegidos)
        .await
        .unwrap();
    let (token, csrf) = entrar_como(&contexto, operario).await;
    (contexto, token, csrf)
}

async fn entrar_como(contexto: &Contexto, usuario_id: i64) -> (String, String) {
    let solicitud = SolicitudDeEntrada {
        usuario_id,
        clave: PIN.into(),
        ip: "10.0.0.7".into(),
        duracion: DuracionDeSesion::Normal,
    };
    match entrar(&contexto.estado, solicitud).await.unwrap() {
        ResultadoDeEntrada::Correcta { token, .. } => {
            let actor: Actor = servidor::aplicacion::sesion::identificar(&contexto.estado, &token)
                .await
                .unwrap()
                .unwrap();
            (token, actor.token_csrf)
        }
        otro => panic!("el operario no pudo entrar: {otro:?}"),
    }
}

async fn estado_de(contexto: &Contexto, token: &str, ruta: &str) -> StatusCode {
    contexto.pedir(get(ruta, Some(token))).await.status()
}

async fn pagina(contexto: &Contexto, token: &str, ruta: &str) -> String {
    let respuesta = contexto.pedir(get(ruta, Some(token))).await;
    assert_eq!(respuesta.status(), StatusCode::OK, "{ruta}");
    texto(respuesta).await
}

/// Lo del dueño: ni con todos los permisos responde a un operario.
const SOLO_DEL_DUENO: [&str; 7] = [
    "/trabajos",
    "/catalogo/articulos",
    "/stock",
    "/ajustes/usuarios",
    "/ajustes/presupuestos",
    "/clientes/importar",
    "/presupuestos/1/lineas/nueva",
];

#[tokio::test]
async fn sin_permisos_ve_clientes_y_su_dia_y_nada_mas() {
    let (contexto, token, _) = taller_con_operario(&[]).await;
    let ficha = pagina(&contexto, &token, "/clientes/1").await;
    for oculto in [
        "Editar datos",
        "Nuevo presupuesto",
        "Borrar cliente",
        "Exportar datos",
    ] {
        assert!(!ficha.contains(oculto), "{oculto}");
    }
    assert!(!ficha.contains("/presupuestos/1"));
    pagina(&contexto, &token, "/mi-dia").await;
    for ruta in [
        "/presupuestos",
        "/presupuestos/1",
        "/facturas",
        "/clientes/nuevo",
    ] {
        assert_eq!(
            estado_de(&contexto, &token, ruta).await,
            StatusCode::NOT_FOUND,
            "{ruta}"
        );
    }
    for ruta in SOLO_DEL_DUENO {
        assert_eq!(
            estado_de(&contexto, &token, ruta).await,
            StatusCode::NOT_FOUND,
            "{ruta}"
        );
    }
}

#[tokio::test]
async fn con_todos_los_permisos_ve_el_presupuesto_sin_costes_ni_acciones() {
    let (contexto, token, csrf) = taller_con_operario(&Permiso::TODOS).await;
    let presupuesto = pagina(&contexto, &token, "/presupuestos/1").await;
    assert!(presupuesto.contains("Total con IVA"));
    assert!(presupuesto.contains("Imprimir o guardar PDF"));
    for oculto in [
        "<dt>Perfiles</dt>",
        "<dt>Cristal</dt>",
        "<dt>Herrajes</dt>",
        "<dt>Mano de obra</dt>",
        "Aceptar presupuesto",
        "Marcar como enviado",
        "Nueva versión",
        "+ Ventana",
        "Guardar datos",
    ] {
        assert!(!presupuesto.contains(oculto), "{oculto}");
    }
    pagina(&contexto, &token, "/presupuestos/1/imprimir").await;
    pagina(&contexto, &token, "/facturas").await;
    pagina(&contexto, &token, "/clientes/nuevo").await;
    for ruta in SOLO_DEL_DUENO {
        assert_eq!(
            estado_de(&contexto, &token, ruta).await,
            StatusCode::NOT_FOUND,
            "{ruta}"
        );
    }
    let enviar = formulario(&[("_csrf", &csrf)]);
    let respuesta = contexto
        .pedir(post(
            "/presupuestos/1/estado/enviado",
            Some(&token),
            &enviar,
        ))
        .await;
    assert_eq!(respuesta.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn el_menu_ensena_solo_lo_permitido() {
    let (contexto, token, _) = taller_con_operario(&[Permiso::VerFacturas]).await;
    let clientes = pagina(&contexto, &token, "/clientes").await;
    assert!(clientes.contains("href=\"/facturas\""));
    assert!(clientes.contains("href=\"/mi-dia\""));
    for oculto in [
        "href=\"/presupuestos\"",
        "href=\"/catalogo\"",
        "href=\"/ajustes/usuarios\"",
    ] {
        assert!(!clientes.contains(oculto), "{oculto}");
    }
}
