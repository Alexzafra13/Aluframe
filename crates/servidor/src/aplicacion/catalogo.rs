use dominio::acceso::Accion;
use dominio::catalogo::{DatosDeAcabado, DatosDeArticulo, DatosDeProveedor};
use rusqlite::Transaction;

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::acabados::{self, FilaAcabado};
use crate::bd::consultas::articulos::{self, FilaArticulo, FiltroArticulos};
use crate::bd::consultas::proveedores::{self, FilaProveedor};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

async fn leer<T: Send + 'static>(
    estado: &EstadoApp,
    actor: &Actor,
    consulta: impl FnOnce(&rusqlite::Connection) -> Result<T, ErrorApp> + Send + 'static,
) -> Result<T, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado.bd.leer(consulta).await
}

/// Ejecuta un cambio del catálogo en una transacción y deja su apunte de auditoría.
async fn cambiar<T: Send + 'static>(
    estado: &EstadoApp,
    actor: &Actor,
    (accion, entidad): (&'static str, &'static str),
    operacion: impl FnOnce(&Transaction, &str) -> Result<(T, i64), ErrorApp> + Send + 'static,
) -> Result<T, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let (resultado, entidad_id) = operacion(tx, &formatear_utc(ahora))?;
            anotar(
                tx,
                &Cambio {
                    ahora,
                    autor,
                    accion,
                    entidad,
                    entidad_id,
                },
            )?;
            Ok(resultado)
        })
        .await
}

pub async fn listar_proveedores(
    estado: &EstadoApp,
    actor: &Actor,
    con_archivados: bool,
) -> Result<Vec<FilaProveedor>, ErrorApp> {
    leer(estado, actor, move |con| {
        proveedores::listar(con, con_archivados)
    })
    .await
}

pub async fn cargar_proveedor(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FilaProveedor, ErrorApp> {
    leer(estado, actor, move |con| {
        proveedores::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)
    })
    .await
}

fn nombre_libre(tx: &Transaction, nombre: &str, excepto: Option<i64>) -> Result<(), ErrorApp> {
    if proveedores::existe_nombre(tx, nombre, excepto)? {
        return Err(ErrorApp::Validacion(format!(
            "Ya existe el proveedor «{nombre}»."
        )));
    }
    Ok(())
}

pub async fn crear_proveedor(
    estado: &EstadoApp,
    actor: &Actor,
    datos: DatosDeProveedor,
) -> Result<i64, ErrorApp> {
    cambiar(
        estado,
        actor,
        ("crear_proveedor", "proveedor"),
        move |tx, _| {
            nombre_libre(tx, &datos.nombre, None)?;
            let id = proveedores::insertar(tx, &datos)?;
            Ok((id, id))
        },
    )
    .await
}

pub async fn actualizar_proveedor(
    estado: &EstadoApp,
    actor: &Actor,
    ids: (i64, i64),
    datos: DatosDeProveedor,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        actor,
        ("editar_proveedor", "proveedor"),
        move |tx, _| {
            nombre_libre(tx, &datos.nombre, Some(ids.0))?;
            proveedores::actualizar(tx, ids, &datos)?;
            Ok(((), ids.0))
        },
    )
    .await
}

/// Archivar oculta la ficha sin borrarla: lo usado no se borra (DATOS, regla 2).
pub async fn archivar(
    estado: &EstadoApp,
    actor: &Actor,
    (entidad, id): (&'static str, i64),
    archivado: bool,
) -> Result<(), ErrorApp> {
    let accion = if archivado { "archivar" } else { "recuperar" };
    cambiar(estado, actor, (accion, entidad), move |tx, _| {
        proveedores::cambiar_archivado(tx, entidad, id, archivado)?;
        Ok(((), id))
    })
    .await
}

pub async fn listar_acabados(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<FilaAcabado>, ErrorApp> {
    leer(estado, actor, acabados::listar).await
}

pub async fn cargar_acabado(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FilaAcabado, ErrorApp> {
    leer(estado, actor, move |con| {
        acabados::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)
    })
    .await
}

pub async fn crear_acabado(
    estado: &EstadoApp,
    actor: &Actor,
    datos: DatosDeAcabado,
) -> Result<i64, ErrorApp> {
    cambiar(estado, actor, ("crear_acabado", "acabado"), move |tx, _| {
        acabados::comprobar_unicos(tx, &datos, None)?;
        let id = acabados::insertar(tx, &datos)?;
        Ok((id, id))
    })
    .await
}

pub async fn actualizar_acabado(
    estado: &EstadoApp,
    actor: &Actor,
    ids: (i64, i64),
    datos: DatosDeAcabado,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        actor,
        ("editar_acabado", "acabado"),
        move |tx, _| {
            acabados::comprobar_unicos(tx, &datos, Some(ids.0))?;
            acabados::actualizar(tx, ids, &datos)?;
            Ok(((), ids.0))
        },
    )
    .await
}

pub async fn listar_articulos(
    estado: &EstadoApp,
    actor: &Actor,
    filtro: FiltroArticulos,
) -> Result<Vec<FilaArticulo>, ErrorApp> {
    leer(estado, actor, move |con| articulos::listar(con, &filtro)).await
}

pub async fn cargar_articulo(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FilaArticulo, ErrorApp> {
    leer(estado, actor, move |con| {
        articulos::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)
    })
    .await
}

fn referencia_libre(
    tx: &Transaction,
    proveedor_id: i64,
    referencia: &str,
    excepto: Option<i64>,
) -> Result<(), ErrorApp> {
    proveedores::buscar(tx, proveedor_id)?
        .ok_or_else(|| ErrorApp::Validacion("Elige un proveedor.".into()))?;
    if articulos::existe_referencia(tx, (proveedor_id, referencia), excepto)? {
        return Err(ErrorApp::Validacion(format!(
            "Ese proveedor ya tiene la referencia «{referencia}»."
        )));
    }
    Ok(())
}

pub async fn crear_articulo(
    estado: &EstadoApp,
    actor: &Actor,
    proveedor_id: i64,
    datos: DatosDeArticulo,
) -> Result<i64, ErrorApp> {
    cambiar(
        estado,
        actor,
        ("crear_articulo", "articulo"),
        move |tx, ahora| {
            referencia_libre(tx, proveedor_id, &datos.referencia, None)?;
            let id = articulos::insertar(tx, proveedor_id, &datos, (ahora, None))?;
            Ok((id, id))
        },
    )
    .await
}

pub async fn actualizar_articulo(
    estado: &EstadoApp,
    actor: &Actor,
    ids: (i64, i64),
    datos: DatosDeArticulo,
) -> Result<(), ErrorApp> {
    cambiar(
        estado,
        actor,
        ("editar_articulo", "articulo"),
        move |tx, _| {
            let actual = articulos::buscar(tx, ids.0)?.ok_or(ErrorApp::NoEncontrado)?;
            referencia_libre(tx, actual.proveedor_id, &datos.referencia, Some(ids.0))?;
            articulos::actualizar(tx, ids, &datos)?;
            Ok(((), ids.0))
        },
    )
    .await
}
