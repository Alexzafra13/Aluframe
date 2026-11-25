//! Lo que identifica la ventana en el editor: qué tipo es y de qué serie y marca (con su
//! logotipo), y los herrajes con su dibujo y el modelo que se compra (ADR-0029).

use crate::aplicacion::calculo_de_ventana::CalculoDeVentana;
use crate::series::marcas::url_de_logo;
use crate::series::siluetas::url_de_silueta;
use dominio::dibujo::dibujar_chapa;
use dominio::texto::nombre_legible;
use dominio::unidades::Longitud;

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

/// Lo que se pide no va con los herrajes: va en «Para pedir».
pub fn herrajes_en_pantalla(c: &CalculoDeVentana) -> Vec<HerrajeEnPantalla> {
    c.resultado
        .accesorios_sumados()
        .into_iter()
        .filter(|(codigo, _)| !dominio::series::se_pide(&c.serie, codigo))
        .map(|(codigo, cantidad)| HerrajeEnPantalla {
            nombre: nombre_legible(&codigo),
            modelo: c.articulos.get(&codigo).cloned(),
            cantidad: cantidad.to_string().replace('.', ","),
            silueta: url_de_silueta(&c.serie, &codigo),
        })
        .collect()
}

/// Una chapa con su dibujo técnico a escala y sus medidas («?» lo que falta en la serie).
pub struct ChapaEnPantalla {
    pub nombre: String,
    pub modelo: Option<String>,
    pub cantidad: u32,
    pub medida: String,
    pub dibujo: String,
}

pub fn texto_de_medida(ancho: Option<i64>, alto: Option<i64>) -> String {
    let cm = |dmm: Option<i64>| {
        dmm.map_or_else(
            || "?".to_owned(),
            |d| Longitud::desde_decimas_de_mm(d).formatear_cm(),
        )
    };
    format!("{} × {}", cm(ancho), cm(alto))
}

pub fn chapas_en_pantalla(c: &CalculoDeVentana) -> Vec<ChapaEnPantalla> {
    c.resultado
        .chapas
        .iter()
        .map(|chapa| ChapaEnPantalla {
            nombre: chapa
                .nombre
                .clone()
                .unwrap_or_else(|| nombre_legible(&chapa.articulo)),
            modelo: c
                .articulos
                .get(&chapa.articulo)
                .cloned()
                .or_else(|| Some(nombre_legible(&chapa.articulo))),
            cantidad: chapa.cantidad,
            medida: texto_de_medida(
                chapa.ancho.map(Longitud::decimas_de_mm),
                chapa.alto.map(Longitud::decimas_de_mm),
            ),
            dibujo: dibujar_chapa(chapa.ancho, chapa.alto),
        })
        .collect()
}

/// Cristal o artículo que se encarga a medida (la persiana), con su medida.
pub struct APedirEnPantalla {
    pub nombre: String,
    pub modelo: Option<String>,
    pub cantidad: u32,
    pub medida: String,
}

pub fn a_pedir_en_pantalla(c: &CalculoDeVentana) -> Vec<APedirEnPantalla> {
    crate::aplicacion::resumen_de_linea::a_pedir(c)
        .into_iter()
        .map(|m| APedirEnPantalla {
            nombre: match &m.nombre {
                Some(parte) => format!("{} {}", nombre_legible(&m.codigo), parte.to_lowercase()),
                None => nombre_legible(&m.codigo),
            },
            modelo: m.modelo,
            cantidad: m.cantidad,
            medida: texto_de_medida(m.ancho_dmm, m.alto_dmm),
        })
        .collect()
}
