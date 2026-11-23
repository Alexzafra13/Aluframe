//! Presupuestos en cada situación (borrador, enviado, caducado, rechazado, archivado y
//! aceptados) con ventanas de Aluprom 21. Los aceptados pasan al taller (`taller.rs`).

use std::collections::BTreeMap;

use dominio::unidades::{Decimal, Dinero, Porcentaje, PrecioUnitario};
use time::Duration;

use super::taller;
use crate::aplicacion::Actor;
use crate::aplicacion::aceptar_presupuesto::aceptar_presupuesto;
use crate::aplicacion::calculo_de_ventana::EntradaDeVentana;
use crate::aplicacion::lineas_de_presupuesto::{LineaLibre, guardar_libre, guardar_ventana};
use crate::aplicacion::presupuestos::{cambiar_estado, crear_presupuesto};
use crate::bd::consultas::{acabados, demostracion, series};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{fecha_de_madrid, formatear_utc};

/// Una ventana corredera de 2 hojas de Aluprom 21: ancho y alto en cm, unidades, color
/// (sin él, blanco) y si lleva Climalit.
struct Ventana {
    medidas: (i64, i64),
    cantidad: u32,
    acabado: Option<i64>,
    climalit: bool,
}

const fn blanca(ancho: i64, alto: i64) -> Ventana {
    Ventana {
        medidas: (ancho, alto),
        cantidad: 1,
        acabado: None,
        climalit: false,
    }
}

fn iva() -> Porcentaje {
    Porcentaje::desde_puntos_basicos(2_100)
}

pub(super) fn cliente(clientes: &[i64], indice: usize) -> Result<i64, ErrorApp> {
    clientes
        .get(indice)
        .copied()
        .ok_or_else(|| ErrorApp::Interno("faltan clientes en la demostración".into()))
}

/// Lo que todos los presupuestos de la demostración comparten.
struct Contexto<'a> {
    estado: &'a EstadoApp,
    dueno: &'a Actor,
    serie: i64,
    clientes: &'a [i64],
}

impl Contexto<'_> {
    /// Un presupuesto nuevo del cliente `indice` con esas ventanas.
    async fn nuevo(&self, indice: usize, ventanas: Vec<Ventana>) -> Result<i64, ErrorApp> {
        let cliente = cliente(self.clientes, indice)?;
        let id = crear_presupuesto(self.estado, self.dueno, cliente, None).await?;
        for v in ventanas {
            anadir_ventana(self.estado, self.dueno, (id, self.serie), v).await?;
        }
        Ok(id)
    }
}

pub(super) async fn crear(
    estado: &EstadoApp,
    (dueno, operario): (&Actor, &Actor),
    clientes: &[i64],
) -> Result<(), ErrorApp> {
    let serie = estado
        .bd
        .leer(|con| series::buscar_ultima(con, "ALUPROM_21"))
        .await?
        .ok_or_else(|| ErrorApp::Interno("la demostración necesita Aluprom 21".into()))?
        .id;
    let c = Contexto {
        estado,
        dueno,
        serie,
        clientes,
    };
    borrador_y_enviado(&c).await?;
    sin_respuesta(&c).await?;
    let trabajos = aceptados(&c).await?;
    taller::crear(estado, (dueno, operario), (&trabajos, clientes)).await
}

/// Un borrador con una ventana en color y Climalit, y uno enviado que espera respuesta.
async fn borrador_y_enviado(c: &Contexto<'_>) -> Result<(), ErrorApp> {
    let con_color = Ventana {
        medidas: (150, 120),
        cantidad: 1,
        acabado: acabado(c.estado, "Gris antracita").await?,
        climalit: true,
    };
    let borrador = c.nuevo(0, vec![blanca(120, 120), con_color]).await?;
    anadir_libre(c, borrador, ("Mosquitera enrollable", "45")).await?;
    let enviado = c.nuevo(1, vec![blanca(100, 100)]).await?;
    let instalacion = ("Instalación y retirada de la ventana vieja", "60");
    anadir_libre(c, enviado, instalacion).await?;
    cambiar_estado(c.estado, c.dueno, enviado, "enviado").await
}

/// Uno caducado, uno rechazado y uno archivado.
async fn sin_respuesta(c: &Contexto<'_>) -> Result<(), ErrorApp> {
    let caducado = c.nuevo(2, vec![blanca(140, 110)]).await?;
    enviar_hace(c, caducado, (40, 10)).await?;
    let mut tres = blanca(180, 120);
    tres.cantidad = 3;
    let rechazado = c.nuevo(3, vec![tres]).await?;
    cambiar_estado(c.estado, c.dueno, rechazado, "rechazado").await?;
    let archivado = c.nuevo(4, vec![blanca(90, 90)]).await?;
    enviar_hace(c, archivado, (230, 200)).await
}

/// Cuatro aceptados: sus trabajos, en orden.
async fn aceptados(c: &Contexto<'_>) -> Result<Vec<i64>, ErrorApp> {
    let mut trabajos = Vec::new();
    for (indice, ventanas) in [
        (5, vec![blanca(110, 100)]),
        (6, vec![blanca(120, 120), blanca(80, 60)]),
        (7, vec![blanca(160, 120)]),
        (8, vec![blanca(120, 100), blanca(120, 100)]),
    ] {
        let id = c.nuevo(indice, ventanas).await?;
        aceptar_presupuesto(c.estado, c.dueno, id).await?;
        let trabajo = c
            .estado
            .bd
            .leer(move |con| demostracion::trabajo_del_presupuesto(con, id))
            .await?;
        trabajos.push(trabajo);
    }
    Ok(trabajos)
}

async fn anadir_ventana(
    estado: &EstadoApp,
    dueno: &Actor,
    (presupuesto, serie): (i64, i64),
    v: Ventana,
) -> Result<(), ErrorApp> {
    let mut opciones = BTreeMap::new();
    if v.climalit {
        opciones.insert("acristalamiento".to_owned(), "climalit".to_owned());
    }
    let entrada = EntradaDeVentana {
        serie_version_id: serie,
        tipologia: "corredera_2_hojas".into(),
        ancho: Decimal::desde_entero(v.medidas.0),
        alto: Decimal::desde_entero(v.medidas.1),
        opciones,
        acabado_id: v.acabado,
        vidrio_articulo_id: None,
        cantidad: v.cantidad,
        mano_de_obra: Dinero::desde_centimos(4_000),
        descuento: Porcentaje::CERO,
    };
    guardar_ventana(estado, dueno, (presupuesto, 0), (entrada, iva(), false)).await?;
    Ok(())
}

async fn anadir_libre(
    c: &Contexto<'_>,
    presupuesto: i64,
    (descripcion, euros): (&str, &str),
) -> Result<(), ErrorApp> {
    let linea = LineaLibre {
        descripcion: descripcion.into(),
        cantidad_mil: 1_000,
        precio_unitario: PrecioUnitario::desde_texto_es(euros)
            .map_err(|e| ErrorApp::Interno(e.to_string()))?,
        descuento: Porcentaje::CERO,
        iva: iva(),
    };
    guardar_libre(c.estado, c.dueno, (presupuesto, 0), linea).await?;
    Ok(())
}

/// Enviado como si se hubiera hecho hace `dias` días, con la validez acabada hace `vencio`.
async fn enviar_hace(
    c: &Contexto<'_>,
    id: i64,
    (dias, vencio): (i64, i64),
) -> Result<(), ErrorApp> {
    let estado = c.estado;
    cambiar_estado(estado, c.dueno, id, "enviado").await?;
    let ahora = estado.ahora();
    let creado = formatear_utc(ahora - Duration::days(dias));
    let validez = fecha_de_madrid(ahora, -vencio);
    estado
        .bd
        .escribir(move |tx| demostracion::fechar_presupuesto(tx, id, (&creado, &validez)))
        .await
}

async fn acabado(estado: &EstadoApp, nombre: &'static str) -> Result<Option<i64>, ErrorApp> {
    let lista = estado.bd.leer(acabados::listar).await?;
    Ok(lista.into_iter().find(|a| a.nombre == nombre).map(|a| a.id))
}
