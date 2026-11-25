//! El material a pedir de una ventana del trabajo, como se ve en la ficha y en «Mi día».

use dominio::taller::EstadoDeMaterial;
use dominio::texto::nombre_legible;

use super::AvisoDelTaller;
use super::texto_de_medida;
use crate::aplicacion::trabajos::consultar::FichaDeTrabajo;
use crate::bd::consultas::materiales::FilaMaterial;
use crate::fecha::{formatear_para_pantalla, leer_utc};

pub struct MaterialEnPantalla {
    pub id: i64,
    pub version: i64,
    /// «Cristal», «Cristal arriba», «Persiana».
    pub nombre: String,
    pub modelo: Option<String>,
    pub cantidad: i64,
    pub medida: String,
    /// `pendiente`, `pedido` o `recibido`, para la clase y los botones.
    pub estado: String,
    pub etiqueta: &'static str,
    pub nota: Option<String>,
    /// «Luis · 25/11/2026 10:30»: quién lo marcó y cuándo.
    pub quien: String,
}

impl From<&FilaMaterial> for MaterialEnPantalla {
    fn from(m: &FilaMaterial) -> Self {
        let estado =
            EstadoDeMaterial::desde_texto(&m.estado).unwrap_or(EstadoDeMaterial::Pendiente);
        let cuando = m
            .cambiado_en
            .as_deref()
            .and_then(leer_utc)
            .map(formatear_para_pantalla);
        let quien = [m.usuario_nombre.clone(), cuando]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");
        MaterialEnPantalla {
            id: m.id,
            version: m.version,
            nombre: match &m.nombre {
                Some(parte) => format!("{} {}", nombre_legible(&m.codigo), parte.to_lowercase()),
                None => nombre_legible(&m.codigo),
            },
            modelo: m.modelo.clone(),
            cantidad: m.cantidad,
            medida: texto_de_medida(m.ancho_dmm, m.alto_dmm),
            estado: estado.como_texto().to_owned(),
            etiqueta: estado.etiqueta(),
            nota: m.nota.clone(),
            quien,
        }
    }
}

/// Los «falta algo» de los operarios y, si queda material sin pedir, qué (ADR-0035).
pub fn avisos_del_trabajo(ficha: &FichaDeTrabajo) -> Vec<AvisoDelTaller> {
    let enlace = format!("/trabajos/{}", ficha.trabajo.id);
    let codigo = ficha.trabajo.codigo();
    let mut avisos: Vec<AvisoDelTaller> = ficha
        .ventanas
        .iter()
        .filter_map(|v| {
            let nota = v.fila.falta_algo.as_deref()?;
            Some(AvisoDelTaller {
                texto: format!("{codigo} · ventana {}: {nota}", v.fila.orden),
                enlace: enlace.clone(),
            })
        })
        .collect();
    let mut sin_pedir: Vec<String> = ficha
        .ventanas
        .iter()
        .flat_map(|v| &v.materiales)
        .filter(|m| m.estado == "pendiente")
        .map(|m| match m.clase.as_str() {
            "cristal" => "cristal".to_owned(),
            _ => nombre_legible(&m.codigo).to_lowercase(),
        })
        .collect();
    sin_pedir.sort();
    sin_pedir.dedup();
    if !sin_pedir.is_empty() {
        let texto = format!("{codigo}: falta pedir {}", unir_con_y(&sin_pedir));
        avisos.push(AvisoDelTaller { texto, enlace });
    }
    avisos
}

/// «cristal», «cristal y persiana», «cristal, persiana y mosquitera».
fn unir_con_y(partes: &[String]) -> String {
    match partes {
        [] => String::new(),
        [una] => una.clone(),
        [resto @ .., ultima] => format!("{} y {ultima}", resto.join(", ")),
    }
}
