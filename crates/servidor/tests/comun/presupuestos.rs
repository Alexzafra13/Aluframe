//! Preparación compartida: serie y tarifa ficticias de `fixtures/demo/` (nunca datos reales).

use std::collections::BTreeMap;

use dominio::catalogo::{EntradaDeProveedor, validar_proveedor};
use dominio::clientes::{EntradaDeCliente, validar_cliente};
use dominio::unidades::{Decimal, Dinero, Porcentaje};
use servidor::aplicacion::Actor;
use servidor::aplicacion::ajustes_presupuesto::{TextosDeAjustes, guardar_ajustes_de_presupuesto};
use servidor::aplicacion::calculo_de_ventana::EntradaDeVentana;
use servidor::aplicacion::catalogo::crear_proveedor;
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::importar_serie::importar_serie;
use servidor::aplicacion::importar_tarifa::{
    CabeceraDeTarifa, importar_tarifa, previsualizar_tarifa,
};
use servidor::estado::EstadoApp;

pub fn leer(ruta: &str) -> String {
    std::fs::read_to_string(format!("{}/../../{ruta}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

pub const SERIE_DEMO: &str = "fixtures/demo/serie-demo.yaml";
pub const SERIE_COMPLETA: &str = "crates/servidor/tests/datos/serie-prueba-completa.yaml";

/// Deja la serie indicada (versión 1), la tarifa demo vigente y un cliente (n.º 1).
pub async fn preparar_demo(estado: &EstadoApp, actor: &Actor, serie: &str) {
    importar_serie(estado, actor, leer(serie)).await.unwrap();
    let proveedor = validar_proveedor(&EntradaDeProveedor {
        nombre: "DEMO".into(),
        ..EntradaDeProveedor::default()
    })
    .unwrap();
    let proveedor_id = crear_proveedor(estado, actor, proveedor).await.unwrap();
    let tarifa = leer("fixtures/demo/tarifa-demo.csv");
    let previa = previsualizar_tarifa(estado, actor, proveedor_id, tarifa.clone())
        .await
        .unwrap();
    let cabecera = CabeceraDeTarifa {
        proveedor_id,
        nombre: "Demo".into(),
        ..CabeceraDeTarifa::default()
    };
    importar_tarifa(estado, actor, (cabecera, tarifa, previa.sha256))
        .await
        .unwrap();
    let cliente = validar_cliente(&EntradaDeCliente {
        nombre: "Ana Ruiz".into(),
        ..EntradaDeCliente::default()
    })
    .unwrap();
    crear_cliente(estado, actor, cliente, vec![]).await.unwrap();
}

pub async fn elegir_reglas(estado: &EstadoApp, actor: &Actor) {
    let textos = [
        ("precio.margen_modo", "coste"),
        ("precio.margen", "30"),
        ("precio.perfiles", "barras"),
        ("iva.general", "21"),
        ("iva.reducido", "10"),
        ("empresa.nombre", "Taller de prueba"),
    ];
    let textos = TextosDeAjustes(
        textos
            .iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect(),
    );
    guardar_ajustes_de_presupuesto(estado, actor, textos)
        .await
        .unwrap();
}

pub fn ventana(ancho: i64, alto: i64) -> EntradaDeVentana {
    EntradaDeVentana {
        serie_version_id: 1,
        tipologia: "corredera_2_hojas_demo".into(),
        ancho: Decimal::desde_entero(ancho),
        alto: Decimal::desde_entero(alto),
        opciones: BTreeMap::new(),
        acabado_id: None,
        vidrio_articulo_id: None,
        cantidad: 1,
        mano_de_obra: Dinero::CERO,
        descuento: Porcentaje::CERO,
    }
}
