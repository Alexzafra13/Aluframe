use std::collections::BTreeMap;

use axum::Form;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};

use super::editor_formulario::{
    importe_en_campo, leer_ventana, opciones_elegidas, valores_de_linea, valores_del_formulario,
};
use super::presupuestos::{SECCION, opciones_de_iva};
use crate::aplicacion::Actor;
use crate::aplicacion::calculo_de_ventana::EntradaDeVentana;
use crate::aplicacion::datos_del_editor::{
    DatosDelEditor, cargar_datos_del_editor, leer_seleccion,
};
use crate::aplicacion::lineas_de_presupuesto::{calcular_ventana_sin_guardar, guardar_ventana};
use crate::aplicacion::presupuestos::{FichaDePresupuesto, cargar_presupuesto};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::http::extractores::ActorActual;
use crate::http::formulario::Campos;
use crate::vistas::{
    FragmentoCalculo, MENU, Opcion, OpcionEnPantalla, PaginaEditorLinea, PestanaDeLinea,
    ValoresDeVentana, VistaCalculo, renderizar,
};

fn opciones(pares: Vec<(String, String)>) -> Vec<Opcion> {
    pares
        .into_iter()
        .map(|(valor, texto)| Opcion { valor, texto })
        .collect()
}

async fn calcular_vista(
    estado: &EstadoApp,
    actor: &Actor,
    (entrada, limites_revisados): (EntradaDeVentana, bool),
) -> Result<(Option<VistaCalculo>, Option<String>), ErrorApp> {
    let mano = entrada.mano_de_obra;
    match calcular_ventana_sin_guardar(estado, actor, entrada).await {
        Ok(calculo) => {
            let mut vista = VistaCalculo::from(&calculo);
            vista.mano_de_obra = mano.formatear();
            vista.limites_revisados = limites_revisados;
            Ok((Some(vista), None))
        }
        Err(ErrorApp::Validacion(aviso)) => Ok((None, Some(aviso))),
        Err(otro) => Err(otro),
    }
}

/// Cálculo en vivo: devuelve el dibujo y, fuera de banda, las piezas, el pie y las opciones.
pub async fn calcular(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    cabeceras: HeaderMap,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    let valores = valores_del_formulario(&Campos::de(pares.clone()));
    let (calculo, aviso) = match leer_ventana(&valores, opciones_elegidas(&pares)) {
        Ok((entrada, _, _)) => {
            let revisados = valores.limites_revisados;
            calcular_vista(&estado, &actor, (entrada, revisados)).await?
        }
        Err(ErrorApp::Validacion(aviso)) => (None, Some(aviso)),
        Err(otro) => return Err(otro),
    };
    let cambio_de_serie = cabeceras
        .get("hx-trigger-name")
        .is_some_and(|n| n == "seleccion");
    let opciones = if cambio_de_serie {
        let datos =
            cargar_datos_del_editor(&estado, &actor, leer_seleccion(&valores.seleccion)).await?;
        Some(
            datos
                .opciones
                .iter()
                .map(|o| OpcionEnPantalla::de(o, None))
                .collect(),
        )
    } else {
        None
    };
    Ok(renderizar(&FragmentoCalculo {
        calculo,
        aviso,
        opciones,
    })?
    .into_response())
}

struct Editor {
    orden: i64,
    valores: ValoresDeVentana,
    opciones: BTreeMap<String, String>,
    calculo: Option<VistaCalculo>,
    aviso: Option<String>,
}

fn pestanas(ficha: &FichaDePresupuesto) -> Vec<PestanaDeLinea> {
    ficha
        .lineas
        .iter()
        .filter(|l| l.tipo == "ventana")
        .map(|l| PestanaDeLinea {
            orden: l.orden,
            titulo: l
                .descripcion
                .split(" · ")
                .next()
                .unwrap_or_default()
                .to_owned(),
            completa: !l.incompleta && !l.revisar_limites,
        })
        .collect()
}

async fn pagina_del_editor(
    estado: &EstadoApp,
    actor: &Actor,
    (id, ficha): (i64, FichaDePresupuesto),
    e: Editor,
) -> Result<Response, ErrorApp> {
    let datos: DatosDelEditor =
        cargar_datos_del_editor(estado, actor, leer_seleccion(&e.valores.seleccion)).await?;
    let problemas = e.aviso.is_some();
    let mut valores = e.valores;
    if valores.seleccion.is_empty() {
        valores.seleccion = datos
            .series
            .first()
            .map(|(v, _)| v.clone())
            .unwrap_or_default();
    }
    let pagina = PaginaEditorLinea {
        usuario: actor.into(),
        menu: MENU,
        seccion: SECCION,
        id,
        codigo: ficha.presupuesto.codigo(),
        cliente: ficha.cliente.nombre_visible(),
        orden: e.orden,
        pestanas: pestanas(&ficha),
        opciones: datos
            .opciones
            .iter()
            .map(|o| OpcionEnPantalla::de(o, e.opciones.get(&o.codigo)))
            .collect(),
        series: opciones(datos.series),
        acabados: opciones(datos.acabados),
        vidrios: opciones(datos.vidrios),
        ivas: opciones_de_iva(datos.ajustes.iva_general, datos.ajustes.iva_reducido),
        valores,
        calculo: e.calculo,
        aviso: e.aviso,
    };
    Ok((codigo_http(problemas), renderizar(&pagina)?).into_response())
}

fn codigo_http(problemas: bool) -> StatusCode {
    if problemas {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    }
}

pub async fn mostrar_nueva(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
) -> Result<Response, ErrorApp> {
    let ficha = cargar_presupuesto(&estado, &actor, id).await?;
    if !ficha.es_editable() {
        return Ok(Redirect::to(&format!("/presupuestos/{id}")).into_response());
    }
    let ajustes = cargar_datos_del_editor(&estado, &actor, None)
        .await?
        .ajustes;
    let valores = ValoresDeVentana {
        cantidad: "1".into(),
        mano_de_obra: importe_en_campo(ajustes.mano_de_obra.centimos()),
        iva: ajustes.iva_general.puntos_basicos().to_string(),
        ..ValoresDeVentana::default()
    };
    let editor = Editor {
        orden: 0,
        valores,
        opciones: BTreeMap::new(),
        calculo: None,
        aviso: None,
    };
    pagina_del_editor(&estado, &actor, (id, ficha), editor).await
}

pub async fn mostrar_linea(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, orden)): Path<(i64, i64)>,
) -> Result<Response, ErrorApp> {
    let ficha = cargar_presupuesto(&estado, &actor, id).await?;
    let linea = ficha
        .lineas
        .iter()
        .find(|l| l.orden == orden && l.tipo == "ventana")
        .cloned()
        .ok_or(ErrorApp::NoEncontrado)?;
    let valores = valores_de_linea(&linea);
    let opciones: BTreeMap<String, String> =
        serde_json::from_str(&linea.opciones_json).unwrap_or_default();
    let (calculo, aviso) = match leer_ventana(&valores, opciones.clone()) {
        Ok((entrada, _, _)) => {
            let revisados = valores.limites_revisados;
            calcular_vista(&estado, &actor, (entrada, revisados)).await?
        }
        Err(_) => (None, None),
    };
    let editor = Editor {
        orden,
        valores,
        opciones,
        calculo,
        aviso,
    };
    pagina_del_editor(&estado, &actor, (id, ficha), editor).await
}

/// Guarda la línea (orden 0 = nueva) y vuelve a la ficha, o a una línea nueva si se pidió.
pub async fn guardar(
    estado: &EstadoApp,
    actor: &Actor,
    (id, orden): (i64, i64),
    pares: Vec<(String, String)>,
) -> Result<Response, ErrorApp> {
    let campos = Campos::de(pares.clone());
    let valores = valores_del_formulario(&campos);
    let opciones = opciones_elegidas(&pares);
    let resultado = match leer_ventana(&valores, opciones.clone()) {
        Ok(linea) => guardar_ventana(estado, actor, (id, orden), linea).await,
        Err(error) => Err(error),
    };
    match resultado {
        Ok(_) => Ok(Redirect::to(&format!("/presupuestos/{id}?aviso=linea")).into_response()),
        Err(ErrorApp::Validacion(aviso)) => {
            let ficha = cargar_presupuesto(estado, actor, id).await?;
            let editor = Editor {
                orden,
                valores,
                opciones,
                calculo: None,
                aviso: Some(aviso),
            };
            pagina_del_editor(estado, actor, (id, ficha), editor).await
        }
        Err(otro) => Err(otro),
    }
}

pub async fn guardar_nueva(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path(id): Path<i64>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    guardar(&estado, &actor, (id, 0), pares).await
}

pub async fn guardar_existente(
    State(estado): State<EstadoApp>,
    ActorActual(actor): ActorActual,
    Path((id, orden)): Path<(i64, i64)>,
    Form(pares): Form<Vec<(String, String)>>,
) -> Result<Response, ErrorApp> {
    guardar(&estado, &actor, (id, orden), pares).await
}
