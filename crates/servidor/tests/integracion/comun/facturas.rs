//! Facturas libres para las pruebas, sobre un cliente sin NIF (n.º 1).

use dominio::clientes::{EntradaDeCliente, validar_cliente};
use dominio::unidades::{Porcentaje, PrecioUnitario};
use servidor::aplicacion::Actor;
use servidor::aplicacion::clientes::crear_cliente;
use servidor::aplicacion::facturas::cargar_factura;
use servidor::aplicacion::facturas::crear::factura_libre;
use servidor::aplicacion::facturas::editar::{
    DatosDeFactura, LineaNueva, anadir_linea, guardar_datos,
};
use servidor::aplicacion::facturas::emitir::emitir_factura;
use servidor::estado::EstadoApp;

use super::Contexto;

/// El propietario y la clienta «Ana Ruiz» (n.º 1, sin NIF).
pub async fn con_cliente(contexto: &Contexto) -> (String, Actor) {
    let (token, actor) = contexto.con_propietario().await;
    let cliente = validar_cliente(&EntradaDeCliente {
        nombre: "Ana Ruiz".into(),
        ..EntradaDeCliente::default()
    })
    .unwrap();
    crear_cliente(&contexto.estado, &actor, cliente, vec![])
        .await
        .unwrap();
    (token, actor)
}

/// Factura libre de 80 € + IVA al cliente 1 con la fecha de operación y el vencimiento dados,
/// ya emitida (confirmando que falta el NIF).
pub async fn emitida_libre(
    estado: &EstadoApp,
    actor: &Actor,
    (fecha_operacion, vencimiento): (Option<&str>, Option<&str>),
) -> i64 {
    let id = factura_libre(estado, actor, 1).await.unwrap();
    let linea = LineaNueva {
        descripcion: "Instalación".into(),
        cantidad_mil: 1_000,
        precio: PrecioUnitario::desde_texto_es("80").unwrap(),
        descuento: Porcentaje::CERO,
        iva: Porcentaje::desde_puntos_basicos(2_100),
    };
    anadir_linea(estado, actor, id, linea).await.unwrap();
    let version = cargar_factura(estado, actor, id)
        .await
        .unwrap()
        .factura
        .version;
    let datos = DatosDeFactura {
        fecha_operacion: fecha_operacion.map(str::to_owned),
        vencimiento: vencimiento.map(str::to_owned),
        notas: None,
        descuento: Porcentaje::CERO,
    };
    guardar_datos(estado, actor, (id, version), datos)
        .await
        .unwrap();
    emitir_factura(estado, actor, id, true).await.unwrap();
    id
}
