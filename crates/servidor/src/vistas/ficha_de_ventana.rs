//! Lo que identifica la ventana en el editor: qué tipo es y de qué serie y marca (con su
//! logotipo), y los herrajes con su dibujo y el modelo que se compra (ADR-0029).

use crate::aplicacion::calculo_de_ventana::CalculoDeVentana;
use crate::series::marcas::url_de_logo;
use crate::series::siluetas::url_de_silueta;
use dominio::texto::nombre_legible;

pub struct FichaDeSerie {
    /// «Ventana corredera 4 hojas».
    pub tipo: String,
    /// «Aluprom 21».
    pub modelo: String,
    /// «Aluval»: el texto se ve si no hay logotipo y siempre lo leen los lectores de pantalla.
    pub fabricante: String,
    pub logo: Option<String>,
}

impl FichaDeSerie {
    pub fn desde(c: &CalculoDeVentana) -> FichaDeSerie {
        let tipo = c.serie.tipologia(&c.resultado.tipologia).map_or_else(
            || nombre_legible(&c.resultado.tipologia),
            |t| t.nombre.clone(),
        );
        FichaDeSerie {
            tipo,
            modelo: nombre_legible(&c.serie.codigo),
            fabricante: nombre_legible(&c.serie.fabricante),
            logo: url_de_logo(&c.serie.fabricante),
        }
    }
}

/// Un herraje (cierre, kit, rueda, tornillo): lo que no se corta.
pub struct HerrajeEnPantalla {
    /// El nombre genérico de la serie: «Cierre embutido».
    pub nombre: String,
    /// La descripción del artículo del catálogo, si lo hay: el modelo concreto que se compra.
    pub modelo: Option<String>,
    pub cantidad: String,
    pub silueta: Option<String>,
}

pub fn herrajes_en_pantalla(c: &CalculoDeVentana) -> Vec<HerrajeEnPantalla> {
    c.resultado
        .accesorios_sumados()
        .into_iter()
        .map(|(codigo, cantidad)| HerrajeEnPantalla {
            nombre: nombre_legible(&codigo),
            modelo: c.articulos.get(&codigo).cloned(),
            cantidad: cantidad.to_string().replace('.', ","),
            silueta: url_de_silueta(&c.serie, &codigo),
        })
        .collect()
}
