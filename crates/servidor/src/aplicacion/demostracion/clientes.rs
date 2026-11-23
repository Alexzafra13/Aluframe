//! Clientes inventados: nombres con «Demo» para que nunca se confundan con uno de verdad, calles
//! ficticias y teléfonos de relleno; localidades y códigos postales reales de la comarca.

use dominio::clientes::{EntradaDeCliente, validar_cliente};

use crate::aplicacion::Actor;
use crate::aplicacion::clientes::crear_cliente;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Nombre, tipo, número de la calle ficticia, código postal y localidad.
const CLIENTES: [(&str, &str, &str, &str, &str); 12] = [
    ("Lucía Demo Pérez", "particular", "3", "06300", "Zafra"),
    ("Antonio Demo García", "particular", "12", "06800", "Mérida"),
    ("Carmen Demo López", "particular", "7", "06001", "Badajoz"),
    (
        "Construcciones Demo S.L.",
        "empresa",
        "40",
        "06200",
        "Almendralejo",
    ),
    (
        "Francisco Demo Sánchez",
        "particular",
        "21",
        "06300",
        "Zafra",
    ),
    (
        "María Demo Fernández",
        "particular",
        "5",
        "06230",
        "Los Santos de Maimona",
    ),
    (
        "José Demo Martín",
        "particular",
        "9",
        "06220",
        "Villafranca de los Barros",
    ),
    (
        "Isabel Demo Gómez",
        "particular",
        "14",
        "06240",
        "Fuente de Cantos",
    ),
    (
        "Comunidad de vecinos Demo",
        "empresa",
        "2",
        "06300",
        "Zafra",
    ),
    ("Pedro Demo Ruiz", "particular", "33", "06001", "Badajoz"),
    (
        "Rosa Demo Díaz",
        "particular",
        "8",
        "06380",
        "Jerez de los Caballeros",
    ),
    ("Manuel Demo Moreno", "particular", "17", "06300", "Zafra"),
];

/// Los identificadores, en el orden de la lista.
pub(super) async fn crear(estado: &EstadoApp, dueno: &Actor) -> Result<Vec<i64>, ErrorApp> {
    let mut ids = Vec::with_capacity(CLIENTES.len());
    for (indice, (nombre, tipo, numero, cp, localidad)) in CLIENTES.into_iter().enumerate() {
        let entrada = EntradaDeCliente {
            tipo: tipo.into(),
            nombre: nombre.into(),
            telefono: format!("600 000 {:03}", indice + 1),
            fiscal_calle: format!("Calle Ficticia, {numero}"),
            fiscal_cp: cp.into(),
            fiscal_localidad: localidad.into(),
            fiscal_provincia: "Badajoz".into(),
            fiscal_pais: "España".into(),
            notas: "Cliente inventado para la demostración.".into(),
            ..EntradaDeCliente::default()
        };
        let datos = validar_cliente(&entrada)
            .map_err(|e| ErrorApp::Interno(format!("cliente de la demostración: {e:?}")))?;
        ids.push(crear_cliente(estado, dueno, datos, Vec::new()).await?);
    }
    Ok(ids)
}
