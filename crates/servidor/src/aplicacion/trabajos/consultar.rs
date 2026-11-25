//! Lo que se lee de los trabajos: la ficha, la lista para el tablero y las ventanas del operario.

use dominio::acceso::Accion;
use dominio::taller::{ColumnaDelTablero, EstadoDePaso, EstadoDeTrabajo, columna_del_tablero};
use rusqlite::Connection;

use super::congelado::VentanaCongelada;
use crate::aplicacion::{Actor, autorizar};
use crate::bd::consultas::materiales::{self, FilaMaterial};
use crate::bd::consultas::pasos::{self, FilaPaso};
use crate::bd::consultas::repetidas::{self, FilaRepetida};
use crate::bd::consultas::trabajos::{self, FilaTrabajo};
use crate::bd::consultas::ventanas_trabajo::{self, FilaVentana};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;

pub struct VentanaDeTrabajo {
    pub fila: FilaVentana,
    pub congelado: VentanaCongelada,
    pub pasos: Vec<FilaPaso>,
    /// Piezas que se han vuelto a cortar (corte mal, rayón…): entran en el plan y en el stock.
    pub repetidas: Vec<FilaRepetida>,
    /// Cristal y lo que se encarga a medida, con su estado (ADR-0035).
    pub materiales: Vec<FilaMaterial>,
}

impl VentanaDeTrabajo {
    pub fn hechos(&self) -> usize {
        self.pasos.iter().filter(|p| p.estado == "hecho").count()
    }
}

pub struct FichaDeTrabajo {
    pub trabajo: FilaTrabajo,
    pub ventanas: Vec<VentanaDeTrabajo>,
}

impl FichaDeTrabajo {
    pub fn progreso(&self) -> (usize, usize) {
        let hechos = self.ventanas.iter().map(VentanaDeTrabajo::hechos).sum();
        let total = self.ventanas.iter().map(|v| v.pasos.len()).sum();
        (hechos, total)
    }

    pub fn estado(&self) -> EstadoDeTrabajo {
        EstadoDeTrabajo::desde_texto(&self.trabajo.estado).unwrap_or(EstadoDeTrabajo::Pendiente)
    }

    pub fn columna(&self) -> Option<ColumnaDelTablero> {
        let primeros: Vec<EstadoDePaso> = self
            .ventanas
            .iter()
            .filter_map(|v| v.pasos.first())
            .filter_map(|p| EstadoDePaso::desde_texto(&p.estado))
            .collect();
        columna_del_tablero(self.estado(), &primeros)
    }

    pub fn falta_algo(&self) -> Option<&str> {
        self.ventanas
            .iter()
            .find_map(|v| v.fila.falta_algo.as_deref())
    }
}

pub fn ventana_completa(con: &Connection, fila: FilaVentana) -> Result<VentanaDeTrabajo, ErrorApp> {
    Ok(VentanaDeTrabajo {
        congelado: VentanaCongelada::leer(&fila.congelado_json),
        pasos: pasos::de_ventana(con, fila.id)?,
        repetidas: repetidas::de_ventana(con, fila.id)?,
        materiales: materiales::de_ventana(con, fila.id)?,
        fila,
    })
}

pub fn cargar(con: &Connection, id: i64) -> Result<FichaDeTrabajo, ErrorApp> {
    let trabajo = trabajos::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
    let ventanas = ventanas_trabajo::de_trabajo(con, id)?
        .into_iter()
        .map(|v| ventana_completa(con, v))
        .collect::<Result<_, _>>()?;
    Ok(FichaDeTrabajo { trabajo, ventanas })
}

pub async fn cargar_trabajo(
    estado: &EstadoApp,
    actor: &Actor,
    id: i64,
) -> Result<FichaDeTrabajo, ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado.bd.leer(move |con| cargar(con, id)).await
}

pub async fn listar_trabajos(
    estado: &EstadoApp,
    actor: &Actor,
    incluir_cerrados: bool,
) -> Result<Vec<FichaDeTrabajo>, ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado
        .bd
        .leer(move |con| {
            trabajos::listar(con, incluir_cerrados)?
                .into_iter()
                .map(|t| cargar(con, t.id))
                .collect()
        })
        .await
}

pub async fn trabajos_de_cliente(
    estado: &EstadoApp,
    actor: &Actor,
    cliente: i64,
) -> Result<Vec<FilaTrabajo>, ErrorApp> {
    autorizar(actor, Accion::GestionarTrabajos)?;
    estado
        .bd
        .leer(move |con| trabajos::de_cliente(con, cliente))
        .await
}

/// Una ventana con su trabajo (sin importes). El operario solo ve las suyas; las demás, 404.
pub fn ventana_para(
    con: &Connection,
    actor: &Actor,
    id: i64,
) -> Result<(VentanaDeTrabajo, FilaTrabajo), ErrorApp> {
    let fila = ventanas_trabajo::buscar(con, id)?.ok_or(ErrorApp::NoEncontrado)?;
    let trabajo = trabajos::buscar(con, fila.trabajo_id)?.ok_or(ErrorApp::NoEncontrado)?;
    if !actor.rol.es_propietario() {
        // Con permiso (ADR-0024), un operario ve y marca las de todo el taller.
        let suya = actor.puede(Accion::VerTodosLosTrabajos)
            || fila.asignado_a == Some(actor.id)
            || (fila.asignado_a.is_none() && trabajo.asignado_a == Some(actor.id));
        let abierto = !matches!(trabajo.estado.as_str(), "entregado" | "anulado");
        if !suya || !abierto {
            return Err(ErrorApp::NoEncontrado);
        }
    }
    Ok((ventana_completa(con, fila)?, trabajo))
}

/// Con permiso de ver todo el taller: las ventanas abiertas de los demás (sin importes).
pub async fn ventanas_del_resto_del_taller(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<(VentanaDeTrabajo, FilaTrabajo)>, ErrorApp> {
    autorizar(actor, Accion::VerTodosLosTrabajos)?;
    let usuario = actor.id;
    estado
        .bd
        .leer(move |con| {
            let mut resultado = Vec::new();
            for fila in ventanas_trabajo::abiertas_de_otros(con, usuario)? {
                let trabajo =
                    trabajos::buscar(con, fila.trabajo_id)?.ok_or(ErrorApp::NoEncontrado)?;
                resultado.push((ventana_completa(con, fila)?, trabajo));
            }
            Ok(resultado)
        })
        .await
}

/// «Mi día»: las ventanas asignadas al usuario con el trabajo al que pertenecen.
pub async fn ventanas_de_operario(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<(VentanaDeTrabajo, FilaTrabajo)>, ErrorApp> {
    autorizar(actor, Accion::VerMiDia)?;
    let usuario = actor.id;
    estado
        .bd
        .leer(move |con| {
            let mut resultado = Vec::new();
            for fila in ventanas_trabajo::de_operario(con, usuario)? {
                let trabajo =
                    trabajos::buscar(con, fila.trabajo_id)?.ok_or(ErrorApp::NoEncontrado)?;
                resultado.push((ventana_completa(con, fila)?, trabajo));
            }
            Ok(resultado)
        })
        .await
}
