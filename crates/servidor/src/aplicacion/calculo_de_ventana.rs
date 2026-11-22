use std::collections::{BTreeMap, BTreeSet};

use dominio::corte::optimizar_cortes;
use dominio::dibujo::{ColorDeDibujo, DatosDeDibujo, Herrajes, dibujar_ventana};
use dominio::precio::{
    CosteDeMateriales, DatosDeLinea, PerfilUsado, PrecioDeLinea, calcular_coste_de_materiales,
    calcular_precio_de_linea,
};
use dominio::series::{Dato, ResultadoVentana, Serie, calcular_ventana_con_opciones};
use dominio::unidades::{Decimal, Dinero, Longitud, Porcentaje, Unidad};
use rusqlite::Connection;

use super::ajustes_presupuesto::AjustesDePresupuesto;
use super::enlace_con_tarifa::enlazar;
use super::series::reconstruir;
use super::taller::AjustesDeTaller;
use crate::bd::consultas::{acabados, series};
use crate::errores::ErrorApp;

/// Lo que se escribe en el editor de línea.
#[derive(Debug, Clone)]
pub struct EntradaDeVentana {
    pub serie_version_id: i64,
    pub tipologia: String,
    pub ancho: Decimal,
    pub alto: Decimal,
    pub opciones: BTreeMap<String, String>,
    pub acabado_id: Option<i64>,
    pub vidrio_articulo_id: Option<i64>,
    pub cantidad: u32,
    pub mano_de_obra: Dinero,
    pub descuento: Porcentaje,
}

pub struct CalculoDeVentana {
    pub serie: Serie,
    pub resultado: ResultadoVentana,
    pub dibujo: String,
    pub materiales: CosteDeMateriales,
    pub precio: PrecioDeLinea,
    pub tarifas: BTreeSet<i64>,
    pub descripcion: String,
    pub barras: BTreeMap<String, u32>,
}

pub fn calcular(
    con: &Connection,
    entrada: &EntradaDeVentana,
    (taller, ajustes): (&AjustesDeTaller, &AjustesDePresupuesto),
) -> Result<CalculoDeVentana, ErrorApp> {
    let (serie, tipologia) = cargar_serie(con, entrada)?;
    let medidas = (entrada.ancho, entrada.alto);
    let resultado =
        calcular_ventana_con_opciones(&serie, &entrada.tipologia, medidas, &entrada.opciones)
            .map_err(|e| ErrorApp::Validacion(e.to_string()))?;
    let (perfiles, faltan) = perfiles_usados(&serie, &resultado, taller);
    let enlace = enlazar(con, &serie, &resultado, entrada.vidrio_articulo_id)?;
    let acabado = entrada
        .acabado_id
        .map(|id| acabados::buscar(con, id))
        .transpose()?
        .flatten();
    let recargo = Porcentaje::desde_puntos_basicos(acabado.as_ref().map_or(0, |a| a.recargo_bp));
    let avisos = faltan
        .into_iter()
        .chain(enlace.avisos.iter().cloned())
        .collect();
    let (materiales, precio) = preciar(
        &resultado,
        (&perfiles, &enlace.catalogo, avisos),
        (entrada, recargo),
        ajustes,
    );
    let color = acabado.map(|a| (color_de_dibujo(&a), a.nombre));
    let (descripcion, dibujo) = presentar(&tipologia, &resultado, (entrada, serie.unidad), color);
    Ok(CalculoDeVentana {
        descripcion,
        dibujo,
        barras: perfiles
            .iter()
            .map(|(p, u)| (p.clone(), u.barras))
            .collect(),
        tarifas: enlace.tarifas,
        serie,
        resultado,
        materiales,
        precio,
    })
}

fn cargar_serie(
    con: &Connection,
    entrada: &EntradaDeVentana,
) -> Result<(Serie, dominio::series::Tipologia), ErrorApp> {
    let version =
        series::buscar_version(con, entrada.serie_version_id)?.ok_or(ErrorApp::NoEncontrado)?;
    let serie = reconstruir(version)?.serie;
    let tipologia = serie
        .tipologia(&entrada.tipologia)
        .ok_or_else(|| ErrorApp::Validacion("Elige una tipología de la serie.".into()))?
        .clone();
    Ok((serie, tipologia))
}

/// Descripción de la línea («Corredera 2 hojas · 150 × 120 cm · Blanco») y su dibujo.
fn presentar(
    tipologia: &dominio::series::Tipologia,
    resultado: &ResultadoVentana,
    (entrada, unidad): (&EntradaDeVentana, Unidad),
    color: Option<(ColorDeDibujo, String)>,
) -> (String, String) {
    let (ancho, alto) = (medida(entrada.ancho, unidad), medida(entrada.alto, unidad));
    let mut datos =
        DatosDeDibujo::desde_calculo(tipologia, resultado, &entrada.opciones, (ancho, alto));
    let (color_dibujo, nombre) = color.unzip();
    datos.color = color_dibujo.unwrap_or_default();
    let color = nombre.map(|c| format!(" · {c}")).unwrap_or_default();
    let descripcion = format!(
        "{} · {} × {} cm{color}",
        tipologia.nombre,
        ancho.formatear_cm(),
        alto.formatear_cm()
    );
    (descripcion, dibujar_ventana(&datos))
}

/// El color con que se dibuja el acabado; si sus datos no valen, blanco (nunca falla el cálculo).
fn color_de_dibujo(acabado: &acabados::FilaAcabado) -> ColorDeDibujo {
    let cierres = Herrajes::desde_texto(&acabado.cierres).unwrap_or(Herrajes::Negro);
    ColorDeDibujo::nuevo(&acabado.color_dibujo, acabado.madera, cierres).unwrap_or_default()
}

fn medida(valor: Decimal, unidad: Unidad) -> Longitud {
    Longitud::desde_decimal(valor, unidad).unwrap_or(Longitud::CERO)
}

/// Barras del plan de corte y longitud cortada de cada perfil, para una ventana.
fn perfiles_usados(
    serie: &Serie,
    resultado: &ResultadoVentana,
    taller: &AjustesDeTaller,
) -> (BTreeMap<String, PerfilUsado>, Vec<String>) {
    let mut usados = BTreeMap::new();
    let Dato::Valor(longitud_barra) = serie.longitud_barra else {
        return (
            usados,
            vec!["falta la longitud de barra de la serie".into()],
        );
    };
    let mut faltan = Vec::new();
    let parametros = taller.parametros_de_corte(longitud_barra);
    for (perfil, piezas) in resultado.piezas_a_cortar() {
        let cortado = piezas
            .iter()
            .map(|p| p.longitud.decimas_de_mm())
            .sum::<i64>();
        match optimizar_cortes(&piezas, &parametros) {
            Ok(plan) => {
                let barras = u32::try_from(plan.barras.len()).unwrap_or(u32::MAX);
                usados.insert(
                    perfil,
                    PerfilUsado {
                        barras,
                        longitud_barra,
                        cortado: Longitud::desde_decimas_de_mm(cortado),
                    },
                );
            }
            Err(error) => faltan.push(format!("{perfil}: {error}")),
        }
    }
    (usados, faltan)
}

type Catalogo = BTreeMap<String, dominio::precio::ArticuloConPrecio>;

fn preciar(
    resultado: &ResultadoVentana,
    (perfiles, catalogo, avisos): (&BTreeMap<String, PerfilUsado>, &Catalogo, Vec<String>),
    (entrada, recargo): (&EntradaDeVentana, Porcentaje),
    ajustes: &AjustesDePresupuesto,
) -> (CosteDeMateriales, PrecioDeLinea) {
    let mut materiales = calcular_coste_de_materiales(
        resultado,
        perfiles,
        catalogo,
        (ajustes.reglas.modo_perfiles, recargo),
    );
    materiales.faltan.splice(0..0, avisos);
    let linea = DatosDeLinea {
        cantidad: entrada.cantidad,
        recargo_acabado: recargo,
        mano_de_obra: entrada.mano_de_obra,
        descuento: entrada.descuento,
    };
    let precio = calcular_precio_de_linea(&materiales, &ajustes.reglas, &linea);
    (materiales, precio)
}
