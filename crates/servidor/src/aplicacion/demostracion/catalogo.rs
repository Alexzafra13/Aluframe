//! Empresa, precios, proveedor con tarifa inventada para Aluprom 21, stock y un operario. Los
//! precios están en `fixtures/demo/` y solo existen en la base de la demostración.

use dominio::acceso::{Permiso, Permisos, Rol};
use dominio::catalogo::{EntradaDeProveedor, validar_proveedor};
use dominio::stock::TipoDeMovimiento;

use super::{OPERARIO, PIN};
use crate::aplicacion::Actor;
use crate::aplicacion::ajustes_presupuesto::{TextosDeAjustes, guardar_ajustes_de_presupuesto};
use crate::aplicacion::catalogo::crear_proveedor;
use crate::aplicacion::importar_tarifa::{CabeceraDeTarifa, importar_tarifa, previsualizar_tarifa};
use crate::aplicacion::stock::{MovimientoManual, cambiar_minimo, registrar};
use crate::aplicacion::usuarios::{crear_operario, guardar_permisos};
use crate::bd::consultas::articulos_con_precio;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

const TARIFA: &str = include_str!("../../../../../fixtures/demo/tarifa-demo-aluprom-21.csv");

const AJUSTES: [(&str, &str); 19] = [
    ("empresa.nombre", "Carpintería Demo"),
    ("empresa.nombre_comercial", "Aluminios Demo"),
    ("empresa.direccion", "Calle Ficticia, 1"),
    ("empresa.cp", "06300"),
    ("empresa.localidad", "Zafra"),
    ("empresa.provincia", "Badajoz"),
    ("empresa.telefono", "924 000 000"),
    ("empresa.email", "taller@demo.invalid"),
    (
        "empresa.pie",
        "Documento de la demostración: todos los datos son inventados.",
    ),
    ("precio.margen_modo", "coste"),
    ("precio.margen", "35"),
    ("precio.perfiles", "barras"),
    ("precio.merma", "10"),
    ("precio.mano_de_obra", "40"),
    ("iva.general", "21"),
    ("iva.reducido", "10"),
    ("presupuesto.validez_dias", "30"),
    (
        "presupuesto.condiciones",
        "Montaje incluido. Plazo orientativo: tres semanas. (Texto de la demostración)",
    ),
    ("empresa.web", ""),
];

/// Barras enteras y retales (largo en cm) del inventario inicial, y los mínimos.
const STOCK: [(&str, &str, &str); 11] = [
    ("21210", "", "6"),
    ("21210", "180", "1"),
    ("21220", "", "4"),
    ("21230", "", "2"),
    ("21310", "", "6"),
    ("21340", "", "6"),
    ("21350", "", "8"),
    ("21350", "300", "1"),
    ("KIT-CORREDERA", "", "10"),
    ("RUEDA-CORREDERA", "", "40"),
    ("TORNILLO-4,8X25", "", "300"),
];

/// Deja la empresa y el catálogo listos y devuelve el operario de la demostración.
pub(super) async fn preparar(estado: &EstadoApp, dueno: &Actor) -> Result<Actor, ErrorApp> {
    let textos = AJUSTES
        .iter()
        .map(|(clave, valor)| ((*clave).to_owned(), (*valor).to_owned()))
        .collect();
    guardar_ajustes_de_presupuesto(estado, dueno, TextosDeAjustes(textos)).await?;
    tarifa(estado, dueno).await?;
    stock(estado, dueno).await?;
    operario(estado, dueno).await
}

async fn tarifa(estado: &EstadoApp, dueno: &Actor) -> Result<(), ErrorApp> {
    let proveedor = validar_proveedor(&EntradaDeProveedor {
        nombre: "Distribuidor Demo".into(),
        ..EntradaDeProveedor::default()
    })
    .map_err(|e| ErrorApp::Interno(format!("proveedor de la demostración: {e:?}")))?;
    let proveedor_id = crear_proveedor(estado, dueno, proveedor).await?;
    let previa = previsualizar_tarifa(estado, dueno, proveedor_id, TARIFA.into()).await?;
    let cabecera = CabeceraDeTarifa {
        proveedor_id,
        nombre: "Tarifa de la demostración (precios inventados)".into(),
        ..CabeceraDeTarifa::default()
    };
    importar_tarifa(estado, dueno, (cabecera, TARIFA.into(), previa.sha256)).await?;
    Ok(())
}

async fn articulo(estado: &EstadoApp, referencia: &'static str) -> Result<i64, ErrorApp> {
    let encontrados = estado
        .bd
        .leer(move |con| articulos_con_precio::por_referencia(con, referencia))
        .await?;
    encontrados
        .first()
        .map(|a| a.id)
        .ok_or_else(|| ErrorApp::Interno(format!("falta el artículo {referencia}")))
}

async fn stock(estado: &EstadoApp, dueno: &Actor) -> Result<(), ErrorApp> {
    for (referencia, largo_cm, cantidad) in STOCK {
        let movimiento = MovimientoManual {
            articulo: articulo(estado, referencia).await?,
            acabado: None,
            largo_cm: largo_cm.into(),
            cantidad: cantidad.into(),
            motivo: "Inventario inicial de la demostración".into(),
        };
        registrar(estado, dueno, TipoDeMovimiento::Entrada, movimiento).await?;
    }
    // Por debajo del mínimo a propósito: así se ve el aviso de stock bajo.
    let marco_inferior = articulo(estado, "21230").await?;
    cambiar_minimo(estado, dueno, marco_inferior, "4".into()).await
}

async fn operario(estado: &EstadoApp, dueno: &Actor) -> Result<Actor, ErrorApp> {
    let id = crear_operario(estado, dueno, OPERARIO, PIN.into()).await?;
    let permisos = Permisos::con([Permiso::EditarClientes, Permiso::VerPresupuestos]);
    guardar_permisos(estado, dueno, id, permisos.clone()).await?;
    Ok(Actor {
        id,
        nombre: OPERARIO.into(),
        rol: Rol::Operario,
        permisos,
        token_csrf: String::new(),
        huella_sesion: String::new(),
        en_demostracion: true,
    })
}
