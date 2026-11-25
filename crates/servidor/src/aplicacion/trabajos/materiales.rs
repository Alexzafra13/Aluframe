//! Material a pedir de cada ventana del trabajo (ADR-0035): nace al aceptar con lo que el
//! presupuesto tenía en «Para pedir», se rehace al cambiar las medidas y lo marcan como pedido
//! el dueño o el operario que puede ver la ventana.

use dominio::acceso::Accion;
use dominio::taller::{EstadoDeMaterial, MedidaDeMaterial, estado_tras_cambiar_la_medida};
use rusqlite::Connection;

use super::acciones::cambiar;
use super::congelado::VentanaCongelada;
use super::consultar::ventana_para;
use crate::aplicacion::Actor;
use crate::aplicacion::resumen_de_linea::{APedirResumido, VidrioResumido};
use crate::bd::consultas::articulos;
use crate::bd::consultas::materiales::{self, FilaMaterial, NuevoMaterial};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Un cristal o un artículo que pide esta ventana, sacado de su copia congelada.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MaterialDeVentana {
    clase: String,
    codigo: String,
    nombre: Option<String>,
    modelo: Option<String>,
    cantidad: i64,
    ancho_dmm: Option<i64>,
    alto_dmm: Option<i64>,
}

impl From<&VidrioResumido> for MaterialDeVentana {
    fn from(v: &VidrioResumido) -> Self {
        MaterialDeVentana {
            clase: "cristal".into(),
            codigo: v.tipo.clone(),
            nombre: v.nombre.clone(),
            modelo: None,
            cantidad: i64::from(v.cantidad),
            ancho_dmm: Some(v.ancho_dmm),
            alto_dmm: Some(v.alto_dmm),
        }
    }
}

impl From<&APedirResumido> for MaterialDeVentana {
    fn from(m: &APedirResumido) -> Self {
        MaterialDeVentana {
            clase: m.clase.clone(),
            codigo: m.codigo.clone(),
            nombre: m.nombre.clone(),
            modelo: m.modelo.clone(),
            cantidad: i64::from(m.cantidad),
            ancho_dmm: m.ancho_dmm,
            alto_dmm: m.alto_dmm,
        }
    }
}

/// Lo que guardó el presupuesto en «Para pedir». El cristal lleva el modelo elegido en la línea,
/// si lo hay.
fn de_la_ventana(
    con: &Connection,
    c: &VentanaCongelada,
) -> Result<Vec<MaterialDeVentana>, ErrorApp> {
    let vidrio_elegido = match c.vidrio_articulo_id {
        Some(id) => articulos::buscar(con, id)?.map(|a| a.descripcion),
        None => None,
    };
    let r = &c.resumen;
    // Los resúmenes anteriores a la 0.25.0 no tienen «a_pedir»: entonces, su cristal.
    let lista: Vec<MaterialDeVentana> = if r.a_pedir.is_empty() {
        r.vidrios.iter().map(MaterialDeVentana::from).collect()
    } else {
        r.a_pedir.iter().map(MaterialDeVentana::from).collect()
    };
    Ok(lista
        .into_iter()
        .map(|mut m| {
            if m.clase == "cristal" && vidrio_elegido.is_some() {
                m.modelo.clone_from(&vidrio_elegido);
            }
            m
        })
        .collect())
}

fn guardar(
    con: &Connection,
    ventana: i64,
    lista: &[(MaterialDeVentana, EstadoDeMaterial, Option<&str>)],
) -> Result<(), ErrorApp> {
    for (orden, (m, estado, nota)) in (1..).zip(lista) {
        let nuevo = NuevoMaterial {
            clase: &m.clase,
            codigo: &m.codigo,
            nombre: m.nombre.as_deref(),
            modelo: m.modelo.as_deref(),
            cantidad: m.cantidad.max(1),
            ancho_dmm: m.ancho_dmm,
            alto_dmm: m.alto_dmm,
            estado: estado.como_texto(),
            nota: *nota,
        };
        materiales::insertar(con, (ventana, orden), &nuevo)?;
    }
    Ok(())
}

/// Al aceptar el presupuesto: todo pendiente de pedir.
pub fn crear_materiales(
    con: &Connection,
    ventana: i64,
    c: &VentanaCongelada,
) -> Result<(), ErrorApp> {
    let lista: Vec<_> = de_la_ventana(con, c)?
        .into_iter()
        .map(|m| (m, EstadoDeMaterial::Pendiente, None))
        .collect();
    guardar(con, ventana, &lista)
}

fn medida(cantidad: i64, (ancho_dmm, alto_dmm): (Option<i64>, Option<i64>)) -> MedidaDeMaterial {
    MedidaDeMaterial {
        cantidad: u32::try_from(cantidad).unwrap_or_default(),
        ancho_dmm,
        alto_dmm,
    }
}

/// Al cambiar las medidas: cada cosa conserva su estado si su medida no cambió; lo ya pedido con
/// otra medida vuelve a pendiente y lo dice (dominio::taller::estado_tras_cambiar_la_medida).
pub fn rehacer_materiales(
    con: &Connection,
    ventana: i64,
    c: &VentanaCongelada,
) -> Result<(), ErrorApp> {
    let antes = materiales::de_ventana(con, ventana)?;
    let nuevos = de_la_ventana(con, c)?;
    let lista: Vec<_> = nuevos
        .into_iter()
        .map(|m| {
            let anterior = antes
                .iter()
                .find(|a| a.codigo == m.codigo && a.nombre == m.nombre);
            let (estado, nota) = anterior.map_or((EstadoDeMaterial::Pendiente, None), |a| {
                let estado =
                    EstadoDeMaterial::desde_texto(&a.estado).unwrap_or(EstadoDeMaterial::Pendiente);
                let ahora = medida(m.cantidad, (m.ancho_dmm, m.alto_dmm));
                let previa = medida(a.cantidad, (a.ancho_dmm, a.alto_dmm));
                estado_tras_cambiar_la_medida(estado, previa, ahora)
            });
            (m, estado, nota)
        })
        .collect();
    materiales::borrar_de_ventana(con, ventana)?;
    guardar(con, ventana, &lista)
}

/// Pendiente, pedido o recibido, con una nota opcional («Cristalería X, llega el martes»).
/// Devuelve (trabajo, ventana), para volver a la ventana.
pub async fn marcar_material(
    estado: &EstadoApp,
    actor: &Actor,
    (id, version): (i64, i64),
    (nuevo, nota): (EstadoDeMaterial, Option<String>),
) -> Result<(i64, i64), ErrorApp> {
    let nota = nota
        .map(|n| n.trim().chars().take(200).collect::<String>())
        .filter(|n| !n.is_empty());
    let (quien, usuario) = (actor.clone(), actor.id);
    let permiso = (Accion::MarcarPasos, "cambiar_material", "material_a_pedir");
    cambiar(estado, actor, permiso, move |tx, ahora| {
        let fila: FilaMaterial = materiales::buscar(tx, id)?.ok_or(ErrorApp::NoEncontrado)?;
        let (_, trabajo) = ventana_para(tx, &quien, fila.ventana_id)?;
        let datos = (nuevo.como_texto(), nota.as_deref());
        materiales::cambiar_estado(tx, (id, version), datos, (usuario, ahora))?;
        Ok(((trabajo.id, fila.ventana_id), id))
    })
    .await
}
