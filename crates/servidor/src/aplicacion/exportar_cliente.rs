use dominio::acceso::Accion;

use super::clientes::cargar_cliente;
use super::{Actor, autorizar};
use crate::bd::consultas::apuntes::{self, FilaApunte, FiltroDeApuntes};
use crate::bd::consultas::clientes::FilaCliente;
use crate::bd::consultas::direcciones::FilaDireccion;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

/// Derecho de acceso (RGPD): todo lo guardado del cliente en un CSV de dos columnas.
pub async fn exportar_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<String, ErrorApp> {
    autorizar(actor, Accion::GestionarClientes)?;
    let ficha = cargar_cliente(estado, actor, id).await?;
    let notas = estado
        .bd
        .leer(move |con| apuntes_del_cliente(con, id))
        .await?;
    let mut filas = filas_del_cliente(&ficha.cliente);
    filas.extend(filas_de_direcciones(&ficha.direcciones));
    filas.extend(filas_de_apuntes(&notas));
    let mut csv = String::from("campo;valor\r\n");
    for (campo, valor) in filas {
        csv.push_str(&format!(
            "{campo};{}\r\n",
            citar(&valor.unwrap_or_default())
        ));
    }
    Ok(csv)
}

fn citar(valor: &str) -> String {
    if valor.contains([';', '"', '\n', '\r']) {
        format!("\"{}\"", valor.replace('"', "\"\""))
    } else {
        valor.to_owned()
    }
}

type Fila = (String, Option<String>);

fn filas_del_cliente(c: &FilaCliente) -> Vec<Fila> {
    vec![
        ("numero".into(), Some(c.id.to_string())),
        ("tipo".into(), Some(c.tipo.clone())),
        ("nombre".into(), c.nombre.clone()),
        ("nif".into(), c.nif.clone()),
        ("email".into(), c.email.clone()),
        ("telefono".into(), c.telefono.clone()),
        ("calle".into(), c.fiscal_calle.clone()),
        ("cp".into(), c.fiscal_cp.clone()),
        ("localidad".into(), c.fiscal_localidad.clone()),
        ("provincia".into(), c.fiscal_provincia.clone()),
        ("pais".into(), c.fiscal_pais.clone()),
        ("forma_de_pago".into(), c.forma_de_pago.clone()),
        ("notas".into(), c.notas.clone()),
        ("alta".into(), Some(c.creado_en.clone())),
    ]
}

fn filas_de_direcciones(direcciones: &[FilaDireccion]) -> Vec<Fila> {
    let mut filas = Vec::new();
    for (n, d) in direcciones.iter().enumerate() {
        let datos = &d.datos;
        for (campo, valor) in [
            ("etiqueta", &datos.etiqueta),
            ("calle", &datos.calle),
            ("cp", &datos.cp),
            ("localidad", &datos.localidad),
            ("provincia", &datos.provincia),
            ("contacto", &datos.contacto_nombre),
            ("telefono_contacto", &datos.contacto_telefono),
            ("notas", &datos.notas),
        ] {
            filas.push((format!("direccion_{}_{campo}", n + 1), valor.clone()));
        }
    }
    filas
}

/// Los apuntes también son datos del cliente (ADR-0025): abiertos y cerrados.
fn apuntes_del_cliente(
    con: &rusqlite::Connection,
    cliente: i64,
) -> Result<Vec<FilaApunte>, ErrorApp> {
    let mut todos = Vec::new();
    for abiertos in [true, false] {
        let filtro = FiltroDeApuntes {
            abiertos,
            de_usuario: None,
            cliente: Some(cliente),
        };
        todos.extend(apuntes::listar(con, &filtro)?);
    }
    Ok(todos)
}

fn filas_de_apuntes(notas: &[FilaApunte]) -> Vec<Fila> {
    let mut filas = Vec::new();
    for (n, a) in notas.iter().enumerate() {
        for (campo, valor) in [
            ("fecha", Some(a.creado_en.clone())),
            ("texto", Some(a.texto.clone())),
            ("medidas", a.medidas.clone()),
            ("contacto", a.contacto.clone()),
        ] {
            filas.push((format!("apunte_{}_{campo}", n + 1), valor));
        }
    }
    filas
}
