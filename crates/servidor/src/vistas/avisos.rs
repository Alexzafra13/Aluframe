//! La campana de la barra superior: lo importante, por orden de urgencia.

use askama::Template;

use super::AvisoDelTaller;

/// Cuántos de cada grupo caben en la campana; el resto se ve en el Inicio.
const POR_GRUPO: usize = 5;

pub struct AvisosReunidos {
    pub copias: Option<String>,
    pub apuntes: Vec<AvisoDelTaller>,
    pub falta_algo: Vec<AvisoDelTaller>,
    pub cobros_vencidos: Vec<AvisoDelTaller>,
    pub stock_bajo: Vec<AvisoDelTaller>,
    pub sin_respuesta: Vec<AvisoDelTaller>,
}

pub struct GrupoDeAvisos {
    pub titulo: &'static str,
    pub avisos: Vec<AvisoDelTaller>,
    pub mas: usize,
}

#[derive(Template)]
#[template(path = "campana.html")]
pub struct Campana {
    pub total: usize,
    pub grupos: Vec<GrupoDeAvisos>,
}

impl Campana {
    pub fn desde(avisos: AvisosReunidos) -> Campana {
        let copias = avisos.copias.map(|texto| AvisoDelTaller {
            texto,
            enlace: "/ajustes/copias".to_owned(),
        });
        let grupos: Vec<GrupoDeAvisos> = [
            ("Copias de seguridad", copias.into_iter().collect()),
            ("Apuntes", avisos.apuntes),
            ("Falta algo en el taller", avisos.falta_algo),
            ("Cobros vencidos", avisos.cobros_vencidos),
            ("Stock bajo mínimo", avisos.stock_bajo),
            ("Presupuestos sin respuesta", avisos.sin_respuesta),
        ]
        .into_iter()
        .filter(|(_, lista)| !lista.is_empty())
        .map(|(titulo, mut lista)| {
            let mas = lista.len().saturating_sub(POR_GRUPO);
            lista.truncate(POR_GRUPO);
            GrupoDeAvisos {
                titulo,
                avisos: lista,
                mas,
            }
        })
        .collect();
        Campana {
            total: grupos.iter().map(|g| g.avisos.len() + g.mas).sum(),
            grupos,
        }
    }
}
