//! Lo que no se corta ni sale del stock, sino que se encarga a medida para cada trabajo: el
//! cristal siempre y los artículos marcados `se_pide` en la serie (la persiana). Qué se pide es
//! un dato de la serie, no del programa: lo que se añada más adelante se marca allí.

use super::{ResultadoVentana, Serie};
use crate::unidades::Longitud;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaseDePedido {
    Cristal,
    Articulo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialAPedir {
    pub clase: ClaseDePedido,
    /// El código del cristal o del artículo en la serie (`CRISTAL`, `PERSIANA`).
    pub codigo: String,
    /// «Arriba», «Abajo» en un cristal partido.
    pub nombre: Option<String>,
    pub cantidad: u32,
    /// El cristal, su medida; un artículo, la medida escrita de la ventana como referencia.
    pub ancho: Option<Longitud>,
    pub alto: Option<Longitud>,
}

pub fn material_a_pedir(serie: &Serie, r: &ResultadoVentana) -> Vec<MaterialAPedir> {
    let cristales = r.vidrio.iter().flatten().map(|v| MaterialAPedir {
        clase: ClaseDePedido::Cristal,
        codigo: v.tipo.clone(),
        nombre: v.nombre.clone(),
        cantidad: v.cantidad,
        ancho: Some(v.ancho),
        alto: Some(v.alto),
    });
    let medida = |valor| Longitud::desde_decimal(valor, serie.unidad).ok();
    let articulos = r
        .accesorios_sumados()
        .into_iter()
        .filter(|(codigo, _)| se_pide(serie, codigo))
        .map(|(codigo, cantidad)| MaterialAPedir {
            clase: ClaseDePedido::Articulo,
            codigo,
            nombre: None,
            cantidad: cantidad
                .a_entero()
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(1),
            ancho: medida(r.ancho),
            alto: medida(r.alto),
        });
    cristales.chain(articulos).collect()
}

pub fn se_pide(serie: &Serie, codigo: &str) -> bool {
    serie.referencias.get(codigo).is_some_and(|r| r.se_pide)
}
