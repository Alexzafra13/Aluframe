//! Pantallas de apuntes (ADR-0025).

use askama::Template;
use dominio::acceso::Accion;
use dominio::apuntes::EntradaDeApunte;

use super::{AvisoDelTaller, GrupoMenu, Opcion, UsuarioEnPantalla};
use crate::aplicacion::Actor;
use crate::bd::consultas::apuntes::FilaApunte;
use crate::fecha::{formatear_para_pantalla, leer_utc};

/// Un apunte en la lista, con lo que puede hacer con él quien la mira.
pub struct TarjetaApunte {
    pub id: i64,
    pub version: i64,
    pub texto: String,
    pub medidas: Option<String>,
    /// «Juan → tú», «Tú → Luis» o «Juan»: quién lo escribió y para quién es.
    pub de_para: String,
    pub cliente: Option<(i64, String)>,
    pub contacto: Option<String>,
    pub direccion: Option<String>,
    pub fecha: String,
    pub estado: String,
    pub presupuesto_id: Option<i64>,
    pub puede_marcar: bool,
    pub puede_presupuestar: bool,
}

impl TarjetaApunte {
    pub fn de(fila: FilaApunte, actor: &Actor) -> TarjetaApunte {
        let gestiona = actor.puede(Accion::GestionarApuntes);
        let pendiente = fila.estado == "pendiente";
        TarjetaApunte {
            de_para: de_para(&fila, actor.id),
            puede_marcar: gestiona || fila.para_id == Some(actor.id),
            puede_presupuestar: gestiona && pendiente && fila.cliente_id.is_some(),
            fecha: leer_utc(&fila.creado_en)
                .map(formatear_para_pantalla)
                .unwrap_or_default(),
            cliente: fila.cliente_id.zip(fila.cliente),
            id: fila.id,
            version: fila.version,
            texto: fila.texto,
            medidas: fila.medidas,
            contacto: fila.contacto,
            direccion: fila.direccion,
            estado: fila.estado,
            presupuesto_id: fila.presupuesto_id,
        }
    }
}

fn de_para(fila: &FilaApunte, yo: i64) -> String {
    let quien = |id: i64, nombre: &str| {
        if id == yo {
            "Tú".to_owned()
        } else {
            nombre.to_owned()
        }
    };
    let autor = quien(fila.autor_id, &fila.autor);
    match (fila.para_id, fila.para.as_deref()) {
        (Some(id), _) if id == yo => format!("{autor} → tarea para ti"),
        (Some(_), Some(nombre)) => format!("{autor} → tarea para {nombre}"),
        _ => autor,
    }
}

/// Los pendientes para el dueño en la campana y el Inicio: quién, qué y para quién.
pub fn avisos_de_apuntes(filas: &[FilaApunte]) -> Vec<AvisoDelTaller> {
    filas
        .iter()
        .map(|a| {
            let mut texto: String = a.texto.chars().take(80).collect();
            if a.texto.chars().count() > 80 {
                texto.push('…');
            }
            let sobre = a.cliente.as_deref().or(a.contacto.as_deref());
            AvisoDelTaller {
                texto: match sobre {
                    Some(s) => format!("{}: {texto} ({s})", a.autor),
                    None => format!("{}: {texto}", a.autor),
                },
                enlace: format!("/apuntes#apunte-{}", a.id),
            }
        })
        .collect()
}

#[derive(Template)]
#[template(path = "apuntes.html")]
pub struct PaginaApuntes {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub abiertos: bool,
    pub apuntes: Vec<TarjetaApunte>,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

#[derive(Template)]
#[template(path = "apunte_nuevo.html")]
pub struct PaginaNuevoApunte {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    /// Cuando se abre desde la ficha de un cliente.
    pub cliente: Option<(i64, String)>,
    pub direcciones: Vec<Opcion>,
    /// Solo para el dueño: a quién mandar la tarea («» = para mí).
    pub operarios: Vec<Opcion>,
    pub valores: EntradaDeApunte,
    pub aviso: Option<String>,
}
