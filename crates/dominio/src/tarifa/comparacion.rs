use std::collections::{HashMap, HashSet};

use super::{FilaDeTarifa, SALTO_DE_PRECIO_AVISADO, UnidadDeVenta};
use crate::informe::Informe;
use crate::unidades::{Porcentaje, PrecioUnitario};

/// Lo que ya hay en el catálogo de ese proveedor (artículos no archivados).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticuloVigente {
    pub referencia: String,
    pub unidad: UnidadDeVenta,
    pub precio: Option<PrecioUnitario>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CambioDePrecio {
    pub referencia: String,
    pub descripcion: String,
    pub anterior: Option<PrecioUnitario>,
    pub nuevo: PrecioUnitario,
    pub variacion: Option<Porcentaje>,
}

/// La vista previa: nada de esto se ha guardado todavía.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comparacion {
    pub nuevos: Vec<FilaDeTarifa>,
    pub cambios: Vec<CambioDePrecio>,
    pub iguales: usize,
    /// Artículos con precio que no vienen en el archivo: conservan el anterior (ADR-0012).
    pub ausentes: Vec<String>,
}

impl Comparacion {
    pub fn hay_cambios(&self) -> bool {
        !self.nuevos.is_empty() || !self.cambios.is_empty()
    }
}

pub fn comparar_con_vigente(
    filas: &[FilaDeTarifa],
    vigentes: &[ArticuloVigente],
    informe: &mut Informe,
) -> Comparacion {
    let por_referencia: HashMap<&str, &ArticuloVigente> = vigentes
        .iter()
        .map(|a| (a.referencia.as_str(), a))
        .collect();
    let mut comparacion = Comparacion::default();
    for fila in filas {
        match por_referencia.get(fila.referencia.as_str()) {
            None => comparacion.nuevos.push(fila.clone()),
            Some(vigente) => comparar_fila(fila, vigente, &mut comparacion, informe),
        }
    }
    let en_archivo: HashSet<&str> = filas.iter().map(|f| f.referencia.as_str()).collect();
    comparacion.ausentes = vigentes
        .iter()
        .filter(|a| a.precio.is_some() && !en_archivo.contains(a.referencia.as_str()))
        .map(|a| a.referencia.clone())
        .collect();
    comparacion
}

fn comparar_fila(
    fila: &FilaDeTarifa,
    vigente: &ArticuloVigente,
    comparacion: &mut Comparacion,
    informe: &mut Informe,
) {
    let donde = format!("línea {}", fila.linea);
    if vigente.unidad != fila.unidad {
        let mensaje = format!(
            "{}: la unidad cambia de {} a {}",
            fila.referencia,
            vigente.unidad.nombre(),
            fila.unidad.nombre()
        );
        informe.aviso(&donde, mensaje);
    }
    if vigente.precio == Some(fila.precio) {
        comparacion.iguales += 1;
        return;
    }
    let variacion = vigente.precio.and_then(|a| fila.precio.variacion_desde(a));
    if let Some(v) = variacion.filter(|v| v.absoluto() > SALTO_DE_PRECIO_AVISADO) {
        let mensaje = format!(
            "{}: el precio cambia un {}, más del 30 %; comprueba que es correcto",
            fila.referencia,
            v.formatear(true)
        );
        informe.aviso(&donde, mensaje);
    }
    comparacion.cambios.push(CambioDePrecio {
        referencia: fila.referencia.clone(),
        descripcion: fila.descripcion.clone(),
        anterior: vigente.precio,
        nuevo: fila.precio,
        variacion,
    });
}
