//! «Terminar orden y descontar stock»: lo que sale, los avisos y los retales a revisar.

use askama::Template;
use dominio::stock::formatear_cantidad;
use dominio::texto::nombre_legible;
use dominio::unidades::Longitud;

use super::{GrupoMenu, Opcion, UsuarioEnPantalla};
use crate::aplicacion::stock::prevision::Prevision;
use crate::aplicacion::trabajos::orden::OrdenDeTaller;

pub struct RetalAConfirmar {
    pub indice: usize,
    pub grupo: usize,
    pub texto: String,
    pub largo_cm: String,
}

#[derive(Template)]
#[template(path = "terminar_orden.html")]
pub struct PaginaTerminar {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: i64,
    pub codigo: String,
    pub cliente: String,
    pub salidas: Vec<(String, String)>,
    pub avisos: Vec<String>,
    pub retales: Vec<RetalAConfirmar>,
    pub grupos: Vec<Opcion>,
    pub aviso: Option<String>,
}

impl PaginaTerminar {
    pub fn nueva(
        usuario: UsuarioEnPantalla,
        (orden, prevision): (&OrdenDeTaller, &Prevision),
        aviso: Option<String>,
    ) -> PaginaTerminar {
        PaginaTerminar {
            usuario,
            menu: super::MENU,
            seccion: "trabajos",
            id: orden.ficha.trabajo.id,
            codigo: orden.ficha.trabajo.codigo(),
            cliente: orden.ficha.trabajo.cliente_nombre.clone(),
            salidas: prevision
                .salidas
                .iter()
                .map(|s| (s.texto.clone(), formatear_cantidad(-s.cantidad_mil)))
                .collect(),
            avisos: prevision.avisos.clone(),
            retales: prevision
                .retales
                .iter()
                .enumerate()
                .map(|(indice, r)| RetalAConfirmar {
                    indice,
                    grupo: r.grupo,
                    texto: r.texto.clone(),
                    largo_cm: Longitud::desde_decimas_de_mm(r.largo_dmm).formatear_cm(),
                })
                .collect(),
            grupos: orden
                .enlace
                .perfiles
                .keys()
                .enumerate()
                .map(|(i, (perfil, color))| Opcion {
                    valor: i.to_string(),
                    texto: format!("{} · {color}", nombre_legible(perfil)),
                })
                .collect(),
            aviso,
        }
    }
}
