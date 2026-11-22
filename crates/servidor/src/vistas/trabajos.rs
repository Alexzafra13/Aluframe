//! Pantallas de los trabajos del propietario: tablero del Inicio, lista y ficha (PANTALLAS 3 y 8).

use askama::Template;
use dominio::taller::{ColumnaDelTablero, EstadoDeTrabajo};

use super::{GrupoMenu, Opcion, UsuarioEnPantalla, euros, fecha_corta};
use crate::aplicacion::trabajos::consultar::{FichaDeTrabajo, VentanaDeTrabajo};

/// «Ventana corredera 2 hojas» y «120 × 150 cm» de la descripción de una ventana.
pub fn titulo_y_medidas(descripcion: &str) -> (String, String) {
    let mut tramos = descripcion.split(" · ");
    let titulo = tramos.next().unwrap_or_default().to_owned();
    let medidas = descripcion
        .split(" · ")
        .find(|t| t.contains(" × "))
        .unwrap_or_default()
        .to_owned();
    (titulo, medidas)
}

pub struct TarjetaTrabajo {
    pub id: i64,
    pub codigo: String,
    pub cliente: String,
    pub titulo: String,
    pub medidas: String,
    pub mas: usize,
    pub dibujo: String,
    pub progreso: String,
    pub operario: Option<String>,
    pub fecha: String,
    pub estado: &'static str,
    pub falta_algo: Option<String>,
}

impl From<&FichaDeTrabajo> for TarjetaTrabajo {
    fn from(f: &FichaDeTrabajo) -> Self {
        let primera = f.ventanas.first();
        let (titulo, medidas) = primera.map_or_else(
            || ("Sin ventanas".to_owned(), String::new()),
            |v| titulo_y_medidas(&v.fila.descripcion),
        );
        let (hechos, total) = f.progreso();
        TarjetaTrabajo {
            id: f.trabajo.id,
            codigo: f.trabajo.codigo(),
            cliente: f.trabajo.cliente_nombre.clone(),
            titulo,
            medidas,
            mas: f.ventanas.len().saturating_sub(1),
            dibujo: primera
                .map(|v| v.congelado.resumen.dibujo.clone())
                .unwrap_or_default(),
            progreso: format!("{hechos} de {total} pasos"),
            operario: f.trabajo.asignado_nombre.clone(),
            fecha: f
                .trabajo
                .fecha_prevista
                .as_deref()
                .map(fecha_corta)
                .unwrap_or_default(),
            estado: f.estado().nombre(),
            falta_algo: f.falta_algo().map(str::to_owned),
        }
    }
}

pub struct AvisoDelTaller {
    pub texto: String,
    pub enlace: String,
}

#[derive(Template)]
#[template(path = "inicio.html")]
pub struct PaginaInicio {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub columnas: Vec<(&'static str, Vec<TarjetaTrabajo>)>,
    pub avisos: Vec<AvisoDelTaller>,
}

/// Las tres columnas del tablero y los avisos «falta algo» de los operarios.
pub fn tablero(
    fichas: &[FichaDeTrabajo],
) -> (
    Vec<(&'static str, Vec<TarjetaTrabajo>)>,
    Vec<AvisoDelTaller>,
) {
    let mut columnas = vec![
        ("Por cortar", Vec::new()),
        ("En montaje", Vec::new()),
        ("Listos para entregar", Vec::new()),
    ];
    let mut avisos = Vec::new();
    for ficha in fichas {
        let indice = match ficha.columna() {
            Some(ColumnaDelTablero::PorCortar) => 0,
            Some(ColumnaDelTablero::EnMontaje) => 1,
            Some(ColumnaDelTablero::ParaEntregar) => 2,
            None => continue,
        };
        for v in ficha
            .ventanas
            .iter()
            .filter(|v| v.fila.falta_algo.is_some())
        {
            avisos.push(AvisoDelTaller {
                texto: format!(
                    "{} · ventana {}: {}",
                    ficha.trabajo.codigo(),
                    v.fila.orden,
                    v.fila.falta_algo.as_deref().unwrap_or_default()
                ),
                enlace: format!("/trabajos/{}", ficha.trabajo.id),
            });
        }
        if let Some((_, tarjetas)) = columnas.get_mut(indice) {
            tarjetas.push(TarjetaTrabajo::from(ficha));
        }
    }
    (columnas, avisos)
}

#[derive(Template)]
#[template(path = "trabajos.html")]
pub struct PaginaTrabajos {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub tarjetas: Vec<TarjetaTrabajo>,
    pub cerrados: bool,
}

pub struct PasoEnPantalla {
    pub id: i64,
    pub version: i64,
    pub nombre: String,
    pub estado: String,
    pub quien: String,
}

pub struct VentanaEnPantalla {
    pub id: i64,
    pub orden: i64,
    pub titulo: String,
    pub descripcion: String,
    pub dibujo: String,
    pub pasos: Vec<PasoEnPantalla>,
    pub hechos: usize,
    /// El id del operario como texto, para marcarlo en el desplegable; vacío sin asignar.
    pub asignado: String,
    pub falta_algo: Option<String>,
    pub cristal_recibido: bool,
    pub revision: i64,
    pub medida_cambiada: bool,
}

impl From<&VentanaDeTrabajo> for VentanaEnPantalla {
    fn from(v: &VentanaDeTrabajo) -> Self {
        VentanaEnPantalla {
            id: v.fila.id,
            orden: v.fila.orden,
            titulo: titulo_y_medidas(&v.fila.descripcion).0,
            descripcion: v.fila.descripcion.clone(),
            dibujo: v.congelado.resumen.dibujo.clone(),
            pasos: v
                .pasos
                .iter()
                .map(|p| PasoEnPantalla {
                    id: p.id,
                    version: p.version,
                    nombre: p.nombre.clone(),
                    estado: p.estado.clone(),
                    quien: p.usuario_nombre.clone().unwrap_or_default(),
                })
                .collect(),
            hechos: v.hechos(),
            asignado: v.fila.asignado_a.map(|a| a.to_string()).unwrap_or_default(),
            falta_algo: v.fila.falta_algo.clone(),
            cristal_recibido: v.fila.cristal_recibido,
            revision: v.fila.revision,
            medida_cambiada: v.fila.medida_cambiada,
        }
    }
}

#[derive(Template)]
#[template(path = "trabajo.html")]
pub struct PaginaTrabajo {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub id: i64,
    pub version: i64,
    pub codigo: String,
    pub cliente: String,
    pub cliente_id: i64,
    pub presupuesto_id: i64,
    pub direccion: String,
    pub estado: &'static str,
    pub abierto: bool,
    pub importe: String,
    pub progreso: (usize, usize),
    pub asignado_a: Option<i64>,
    pub fecha_prevista: String,
    pub operarios: Vec<Opcion>,
    pub ventanas: Vec<VentanaEnPantalla>,
    pub aviso: Option<String>,
    pub confirmacion: Option<String>,
}

impl PaginaTrabajo {
    pub fn nueva(
        usuario: UsuarioEnPantalla,
        (ficha, operarios): (&FichaDeTrabajo, Vec<Opcion>),
        (aviso, confirmacion): (Option<String>, Option<String>),
    ) -> PaginaTrabajo {
        let t = &ficha.trabajo;
        let abierto = !matches!(
            ficha.estado(),
            EstadoDeTrabajo::Entregado | EstadoDeTrabajo::Anulado
        );
        PaginaTrabajo {
            usuario,
            menu: super::MENU,
            seccion: "trabajos",
            id: t.id,
            version: t.version,
            codigo: t.codigo(),
            cliente: t.cliente_nombre.clone(),
            cliente_id: t.cliente_id,
            presupuesto_id: t.presupuesto_id,
            direccion: t.direccion_montaje.clone().unwrap_or_default(),
            estado: ficha.estado().nombre(),
            abierto,
            importe: euros(t.importe_cent),
            progreso: ficha.progreso(),
            asignado_a: t.asignado_a,
            fecha_prevista: t.fecha_prevista.clone().unwrap_or_default(),
            operarios,
            ventanas: ficha.ventanas.iter().map(VentanaEnPantalla::from).collect(),
            aviso,
            confirmacion,
        }
    }

    pub fn es_operario(&self, valor: &str) -> bool {
        self.asignado_a.map(|a| a.to_string()).as_deref() == Some(valor)
    }
}
