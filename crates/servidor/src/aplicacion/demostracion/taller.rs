//! Trabajos en distintos momentos (sin empezar, a medias, terminados y entregados), sus facturas
//! y cobros, dos facturas sueltas (una anulada con su rectificativa) y apuntes.

use dominio::apuntes::DatosDeApunte;
use dominio::taller::{AccionDePaso, EstadoDeMaterial};
use dominio::unidades::{Porcentaje, PrecioUnitario};

use super::ventas::cliente;
use crate::aplicacion::Actor;
use crate::aplicacion::apuntes::{PeticionDeApunte, crear_apunte};
use crate::aplicacion::facturas::albaranes::crear_albaran;
use crate::aplicacion::facturas::cargar_factura;
use crate::aplicacion::facturas::cobros::{DatosDeCobro, DestinoDelCobro, registrar_cobro};
use crate::aplicacion::facturas::crear::{factura_de_trabajo, factura_libre, rectificativa};
use crate::aplicacion::facturas::editar::{
    DatosDeFactura, LineaNueva, anadir_linea, guardar_datos,
};
use crate::aplicacion::facturas::emitir::emitir_factura;
use crate::aplicacion::stock::terminar::{RetalConfirmado, preparar_terminar, terminar_orden};
use crate::aplicacion::trabajos::acciones::{asignar_trabajo, cambiar_paso};
use crate::aplicacion::trabajos::materiales::marcar_material;
use crate::bd::consultas::demostracion;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::fecha_de_madrid;

type Quienes<'a> = (&'a Actor, &'a Actor);

pub(super) async fn crear(
    estado: &EstadoApp,
    (dueno, operario): Quienes<'_>,
    (trabajos, clientes): (&[i64], &[i64]),
) -> Result<(), ErrorApp> {
    for trabajo in trabajos {
        asignar(estado, dueno, (*trabajo, operario.id)).await?;
    }
    // El primero se queda sin empezar, tal como nace al aceptar.
    let [_, a_medias, entregado, por_cobrar] = trabajos else {
        return Err(ErrorApp::Interno(
            "la demostración espera cuatro trabajos".into(),
        ));
    };
    marcar_pasos(estado, operario, *a_medias, Some(3)).await?;
    let nota = Some("Cristalería Demo, llega el jueves".to_owned());
    material(
        estado,
        operario,
        *a_medias,
        (EstadoDeMaterial::Pedido, nota),
    )
    .await?;
    let senal = ("100", "bizum");
    cobrar(estado, dueno, DestinoDelCobro::Trabajo(*a_medias), senal).await?;
    for trabajo in [*entregado, *por_cobrar] {
        marcar_pasos(estado, operario, trabajo, None).await?;
        material(estado, dueno, trabajo, (EstadoDeMaterial::Recibido, None)).await?;
        terminar(estado, dueno, trabajo).await?;
    }
    entregar_y_cobrar(estado, dueno, *entregado).await?;
    vencida_a_medias(estado, dueno, *por_cobrar).await?;
    sueltas(estado, dueno, clientes).await?;
    apuntes(estado, (dueno, operario), clientes).await
}

/// Albarán, factura y cobro entero por transferencia.
async fn entregar_y_cobrar(
    estado: &EstadoApp,
    dueno: &Actor,
    trabajo: i64,
) -> Result<(), ErrorApp> {
    crear_albaran(estado, dueno, trabajo).await?;
    let factura = factura_de_trabajo(estado, dueno, trabajo).await?;
    let pagada = facturar(estado, dueno, factura, 30).await?;
    let total = cargar_factura(estado, dueno, pagada)
        .await?
        .factura
        .total_cent;
    let entera = format!("{},{:02}", total / 100, total % 100);
    let cobro = (entera.as_str(), "transferencia");
    cobrar(estado, dueno, DestinoDelCobro::Factura(pagada), cobro).await
}

/// Factura que venció hace cinco días con solo una parte cobrada.
async fn vencida_a_medias(estado: &EstadoApp, dueno: &Actor, trabajo: i64) -> Result<(), ErrorApp> {
    let factura = factura_de_trabajo(estado, dueno, trabajo).await?;
    let vencida = facturar(estado, dueno, factura, -5).await?;
    let cobro = ("200", "efectivo");
    cobrar(estado, dueno, DestinoDelCobro::Factura(vencida), cobro).await
}

async fn asignar(
    estado: &EstadoApp,
    dueno: &Actor,
    (trabajo, operario): (i64, i64),
) -> Result<(), ErrorApp> {
    let version = estado
        .bd
        .leer(move |con| demostracion::version_de_trabajo(con, trabajo))
        .await?;
    let prevista = fecha_de_madrid(estado.ahora(), 7);
    asignar_trabajo(
        estado,
        dueno,
        (trabajo, version),
        (Some(operario), Some(prevista)),
    )
    .await
}

/// Deja todo el material del trabajo pedido o recibido (ADR-0035).
async fn material(
    estado: &EstadoApp,
    quien: &Actor,
    trabajo: i64,
    (nuevo, nota): (EstadoDeMaterial, Option<String>),
) -> Result<(), ErrorApp> {
    let materiales = estado
        .bd
        .leer(move |con| demostracion::materiales_del_trabajo(con, trabajo))
        .await?;
    for material in materiales {
        marcar_material(estado, quien, material, (nuevo, nota.clone())).await?;
    }
    Ok(())
}

/// Termina los pasos del trabajo (todos, o los `hechos` primeros y empieza el siguiente).
async fn marcar_pasos(
    estado: &EstadoApp,
    operario: &Actor,
    trabajo: i64,
    hechos: Option<usize>,
) -> Result<(), ErrorApp> {
    let pasos = estado
        .bd
        .leer(move |con| demostracion::pasos_del_trabajo(con, trabajo))
        .await?;
    for (indice, paso) in pasos.into_iter().enumerate() {
        let accion = match hechos {
            None => AccionDePaso::Terminar,
            Some(n) if indice < n => AccionDePaso::Terminar,
            Some(n) if indice == n => AccionDePaso::Empezar,
            Some(_) => break,
        };
        cambiar_paso(estado, operario, paso, accion).await?;
    }
    Ok(())
}

/// Termina la orden de taller aceptando los retales que propone el plan.
async fn terminar(estado: &EstadoApp, dueno: &Actor, trabajo: i64) -> Result<(), ErrorApp> {
    let (_, prevision) = preparar_terminar(estado, dueno, trabajo).await?;
    let retales = prevision
        .retales
        .iter()
        .map(|r| RetalConfirmado {
            grupo: r.grupo,
            largo_dmm: r.largo_dmm,
        })
        .collect();
    terminar_orden(estado, dueno, trabajo, retales).await
}

/// Emite la factura con vencimiento dentro de `dias` (negativo: ya vencida).
async fn facturar(estado: &EstadoApp, dueno: &Actor, id: i64, dias: i64) -> Result<i64, ErrorApp> {
    let version = cargar_factura(estado, dueno, id).await?.factura.version;
    let datos = DatosDeFactura {
        fecha_operacion: None,
        vencimiento: Some(fecha_de_madrid(estado.ahora(), dias)),
        notas: None,
        descuento: Porcentaje::CERO,
    };
    guardar_datos(estado, dueno, (id, version), datos).await?;
    emitir_factura(estado, dueno, id, true).await?;
    Ok(id)
}

async fn cobrar(
    estado: &EstadoApp,
    dueno: &Actor,
    destino: DestinoDelCobro,
    (importe, medio): (&str, &str),
) -> Result<(), ErrorApp> {
    let datos = DatosDeCobro {
        fecha: fecha_de_madrid(estado.ahora(), 0),
        importe: importe.into(),
        medio: medio.into(),
        nota: String::new(),
    };
    registrar_cobro(estado, dueno, destino, datos).await
}

/// Una factura suelta pendiente y otra anulada con su rectificativa.
async fn sueltas(estado: &EstadoApp, dueno: &Actor, clientes: &[i64]) -> Result<(), ErrorApp> {
    let pendiente = suelta(
        estado,
        dueno,
        (
            cliente(clientes, 9)?,
            "Cambio de cerradura de seguridad",
            "85",
        ),
    )
    .await?;
    facturar(estado, dueno, pendiente, 15).await?;
    let duplicada = suelta(
        estado,
        dueno,
        (cliente(clientes, 10)?, "Reparación de persiana", "60"),
    )
    .await?;
    facturar(estado, dueno, duplicada, 15).await?;
    let motivo = "Factura duplicada por error (demostración)".to_owned();
    let anula = rectificativa(estado, dueno, duplicada, motivo).await?;
    emitir_factura(estado, dueno, anula, true).await?;
    Ok(())
}

async fn suelta(
    estado: &EstadoApp,
    dueno: &Actor,
    (cliente, descripcion, euros): (i64, &str, &str),
) -> Result<i64, ErrorApp> {
    let id = factura_libre(estado, dueno, cliente).await?;
    let linea = LineaNueva {
        descripcion: descripcion.into(),
        cantidad_mil: 1_000,
        precio: PrecioUnitario::desde_texto_es(euros)
            .map_err(|e| ErrorApp::Interno(e.to_string()))?,
        descuento: Porcentaje::CERO,
        iva: Porcentaje::desde_puntos_basicos(2_100),
    };
    anadir_linea(estado, dueno, id, linea).await?;
    Ok(id)
}

async fn apuntes(
    estado: &EstadoApp,
    (dueno, operario): Quienes<'_>,
    clientes: &[i64],
) -> Result<(), ErrorApp> {
    let encargo = PeticionDeApunte {
        cliente: Some(cliente(clientes, 11)?),
        direccion: None,
        para: None,
        datos: DatosDeApunte {
            texto: "Quiere cambiar la ventana del baño por una corredera.".into(),
            medidas: Some("Hueco de 60 × 80 cm, más o menos".into()),
            contacto: None,
        },
    };
    crear_apunte(estado, operario, encargo).await?;
    let tarea = PeticionDeApunte {
        cliente: None,
        direccion: None,
        para: Some(operario.id),
        datos: DatosDeApunte {
            texto: "Recoger los cristales del pedido de esta semana.".into(),
            medidas: None,
            contacto: None,
        },
    };
    crear_apunte(estado, dueno, tarea).await?;
    Ok(())
}
