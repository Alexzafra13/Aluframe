use askama::Template;
use dominio::unidades::{Decimal, Longitud};

use super::{GrupoMenu, UsuarioEnPantalla};
use crate::aplicacion::importar_serie::{Previsualizacion, texto_de_estado};
use crate::aplicacion::series::{CorteDePerfil, PruebaDeVentana, SerieGuardada};
use dominio::series::{PiezaCalculada, VidrioCalculado};

pub struct FilaSerie {
    pub codigo: String,
    pub fabricante: String,
    pub estado: &'static str,
    pub version: i64,
    pub avisos: i64,
    pub tipologias: String,
    pub referencias: String,
}

impl From<&SerieGuardada> for FilaSerie {
    fn from(guardada: &SerieGuardada) -> Self {
        let serie = &guardada.serie;
        FilaSerie {
            codigo: serie.codigo.clone(),
            fabricante: serie.fabricante.clone(),
            estado: texto_de_estado(serie.estado),
            version: guardada.numero_version,
            avisos: guardada.avisos,
            tipologias: serie
                .tipologias
                .iter()
                .map(|t| t.nombre.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            referencias: String::new(),
        }
    }
}

#[derive(Template)]
#[template(path = "series.html")]
pub struct PaginaSeries {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub series: Vec<FilaSerie>,
    pub confirmacion: Option<String>,
}

#[derive(Template)]
#[template(path = "importar_serie.html")]
pub struct PaginaImportarSerie {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub texto: String,
    pub previsualizacion: Option<Previsualizacion>,
}

pub struct FilaPieza {
    pub numero: u32,
    pub perfil: String,
    pub cantidad: u32,
    pub medida: String,
    pub corte: String,
    pub formula: String,
}

pub struct FilaVidrio {
    pub tipo: String,
    pub cantidad: u32,
    pub medidas: String,
}

pub struct FilaBarra {
    pub numero: usize,
    pub piezas: String,
    pub sobrante: String,
}

pub struct GrupoDeBarras {
    pub perfil: String,
    pub barras: Vec<FilaBarra>,
}

pub struct VistaPrueba {
    pub incompleta: bool,
    pub pendientes: Vec<String>,
    pub fuera_de_limites: Vec<String>,
    pub piezas: Vec<FilaPieza>,
    pub vidrio: Option<Vec<FilaVidrio>>,
    pub accesorios: Vec<(String, String)>,
    pub barras: Vec<GrupoDeBarras>,
    pub avisos_de_corte: Vec<String>,
}

#[derive(Template)]
#[template(path = "probar_ventana.html")]
pub struct PaginaProbarVentana {
    pub usuario: UsuarioEnPantalla,
    pub menu: &'static [GrupoMenu],
    pub seccion: &'static str,
    pub codigo: String,
    pub version: i64,
    pub tipologias: Vec<(String, String)>,
    pub elegida: String,
    pub ancho: String,
    pub alto: String,
    pub aviso: Option<String>,
    pub prueba: Option<VistaPrueba>,
}

fn cm(longitud: Longitud) -> String {
    longitud.formatear_cm()
}

fn numero(valor: Decimal) -> String {
    valor.to_string().replace('.', ",")
}

impl From<&PruebaDeVentana> for VistaPrueba {
    fn from(prueba: &PruebaDeVentana) -> Self {
        let resultado = &prueba.resultado;
        VistaPrueba {
            incompleta: resultado.es_incompleta(),
            pendientes: resultado.pendientes.clone(),
            fuera_de_limites: resultado.fuera_de_limites.clone(),
            piezas: resultado.piezas.iter().map(fila_de_pieza).collect(),
            vidrio: resultado
                .vidrio
                .as_ref()
                .map(|vidrios| vidrios.iter().map(fila_de_vidrio).collect()),
            accesorios: resultado
                .accesorios_sumados()
                .into_iter()
                .map(|(articulo, cantidad)| (articulo, numero(cantidad)))
                .collect(),
            barras: prueba.cortes.iter().map(grupo_de_barras).collect(),
            avisos_de_corte: prueba.avisos_de_corte.clone(),
        }
    }
}

fn fila_de_pieza(pieza: &PiezaCalculada) -> FilaPieza {
    FilaPieza {
        numero: pieza.numero,
        perfil: pieza.perfil.clone(),
        cantidad: pieza.cantidad,
        medida: cm(pieza.longitud),
        corte: pieza
            .corte
            .map_or_else(|| "falta".to_owned(), |c| format!("{c}°")),
        formula: pieza.formula.clone(),
    }
}

fn fila_de_vidrio(vidrio: &VidrioCalculado) -> FilaVidrio {
    FilaVidrio {
        tipo: vidrio.tipo.clone(),
        cantidad: vidrio.cantidad,
        medidas: format!("{} × {}", cm(vidrio.ancho), cm(vidrio.alto)),
    }
}

/// El identificador de la pieza cortada es n.º de pieza × 1000 + copia (ResultadoVentana::piezas_a_cortar).
fn grupo_de_barras(corte: &CorteDePerfil) -> GrupoDeBarras {
    let barras = corte.plan.barras.iter().enumerate().map(|(indice, barra)| {
        let piezas: Vec<String> = barra
            .piezas
            .iter()
            .map(|p| format!("n.º {} ({})", p.id / 1000, cm(p.longitud)))
            .collect();
        FilaBarra {
            numero: indice + 1,
            piezas: piezas.join(" + "),
            sobrante: cm(barra.sobrante),
        }
    });
    GrupoDeBarras {
        perfil: corte.perfil.clone(),
        barras: barras.collect(),
    }
}
