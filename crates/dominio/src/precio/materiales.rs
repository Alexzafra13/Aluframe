use std::collections::BTreeMap;

use super::importe::{aumentar, sumar};
use super::{ArticuloConPrecio, ModoPerfiles};
use crate::series::{ResultadoVentana, UnidadCantidad};
use crate::tarifa::UnidadDeVenta;
use crate::unidades::{Decimal, Dinero, Longitud, Porcentaje, SentidoRedondeo};

/// Lo que gasta una ventana de un perfil: barras del plan de corte y metros cortados.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfilUsado {
    pub barras: u32,
    pub longitud_barra: Longitud,
    pub cortado: Longitud,
}

/// Coste de los materiales de una ventana. Los perfiles se calculan de las dos maneras para
/// poder compararlas; `perfiles` es la del modo elegido.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CosteDeMateriales {
    pub perfiles: Dinero,
    pub perfiles_por_barras: Option<Dinero>,
    pub perfiles_por_metros: Option<Dinero>,
    pub cristal: Dinero,
    pub accesorios: Dinero,
    pub faltan: Vec<String>,
}

type Catalogo = BTreeMap<String, ArticuloConPrecio>;

pub fn calcular_coste_de_materiales(
    resultado: &ResultadoVentana,
    perfiles: &BTreeMap<String, PerfilUsado>,
    catalogo: &Catalogo,
    (modo, recargo): (Option<ModoPerfiles>, Porcentaje),
) -> CosteDeMateriales {
    let mut coste = CosteDeMateriales::default();
    coste.faltan.extend(resultado.pendientes.iter().cloned());
    let merma = match modo {
        Some(ModoPerfiles::MetrosConMerma(m)) => m,
        _ => Porcentaje::CERO,
    };
    let barras = sumar_perfiles(perfiles, catalogo, recargo, None);
    let metros = sumar_perfiles(perfiles, catalogo, recargo, Some(merma));
    if let Err(faltas) = &barras {
        coste.faltan.extend(faltas.iter().cloned());
    }
    let (barras, metros) = (barras.ok(), metros.ok());
    coste.perfiles_por_barras = barras;
    coste.perfiles_por_metros = metros;
    coste.perfiles = match modo {
        Some(ModoPerfiles::BarrasEnteras) => barras.unwrap_or_default(),
        Some(ModoPerfiles::MetrosConMerma(_)) => metros.unwrap_or_default(),
        None => {
            coste
                .faltan
                .push("elige cómo se cobran los perfiles (Ajustes > Presupuestos)".into());
            Dinero::CERO
        }
    };
    coste.cristal = coste_de_cristal(resultado, catalogo, recargo, &mut coste.faltan);
    coste.accesorios = coste_de_accesorios(resultado, catalogo, recargo, &mut coste.faltan);
    coste.faltan.dedup();
    coste
}

/// Precio de compra: tarifa − descuento de compra, más el recargo del color si lo lleva.
fn coste(articulo: &ArticuloConPrecio, cantidad_mil: i64, recargo: Porcentaje) -> Option<Dinero> {
    let bruto = articulo.precio.importe(cantidad_mil)?;
    let neto = aumentar(
        bruto,
        Porcentaje::desde_puntos_basicos(-articulo.descuento_compra.puntos_basicos()),
    )?;
    if articulo.con_acabado {
        aumentar(neto, recargo)
    } else {
        Some(neto)
    }
}

fn buscar<'a>(
    catalogo: &'a Catalogo,
    codigo: &str,
    faltan: &mut Vec<String>,
) -> Option<&'a ArticuloConPrecio> {
    let articulo = catalogo.get(codigo);
    if articulo.is_none() {
        faltan.push(format!("falta el precio de {codigo}"));
    }
    articulo
}

/// Con `merma = None` cuenta barras enteras; con merma, metros cortados más la merma.
fn sumar_perfiles(
    perfiles: &BTreeMap<String, PerfilUsado>,
    catalogo: &Catalogo,
    recargo: Porcentaje,
    merma: Option<Porcentaje>,
) -> Result<Dinero, Vec<String>> {
    let mut faltan = Vec::new();
    let mut importes = Vec::with_capacity(perfiles.len());
    for (codigo, usado) in perfiles {
        let Some(articulo) = buscar(catalogo, codigo, &mut faltan) else {
            continue;
        };
        match cantidad_de_perfil(articulo, usado, merma).and_then(|c| coste(articulo, c, recargo)) {
            Some(importe) => importes.push(importe),
            None => faltan.push(format!("{codigo} no se vende por barra ni por metro")),
        }
    }
    if !faltan.is_empty() {
        return Err(faltan);
    }
    sumar(importes).ok_or_else(|| vec!["el importe de los perfiles es demasiado grande".into()])
}

/// Cantidad en milésimas de la unidad de venta (barras o metros).
fn cantidad_de_perfil(
    articulo: &ArticuloConPrecio,
    usado: &PerfilUsado,
    merma: Option<Porcentaje>,
) -> Option<i64> {
    let dmm = match merma {
        None => i64::from(usado.barras).checked_mul(usado.longitud_barra.decimas_de_mm())?,
        Some(m) => {
            let cortado = i128::from(usado.cortado.decimas_de_mm());
            let con_merma = SentidoRedondeo::Arriba
                .dividir(cortado * (10_000 + i128::from(m.puntos_basicos())), 10_000)?;
            i64::try_from(con_merma).ok()?
        }
    };
    match articulo.unidad {
        UnidadDeVenta::Metro => Some(dmm / 10),
        UnidadDeVenta::Barra if merma.is_none() => Some(i64::from(usado.barras) * 1_000),
        UnidadDeVenta::Barra => {
            let barra = articulo
                .longitud_barra
                .unwrap_or(usado.longitud_barra)
                .decimas_de_mm();
            i64::try_from(
                SentidoRedondeo::Cercano.dividir(i128::from(dmm) * 1_000, i128::from(barra))?,
            )
            .ok()
        }
        _ => None,
    }
}

/// Superficie de cada vidrio (m²); la superficie mínima facturable está pendiente del proveedor.
fn coste_de_cristal(
    resultado: &ResultadoVentana,
    catalogo: &Catalogo,
    recargo: Porcentaje,
    faltan: &mut Vec<String>,
) -> Dinero {
    let Some(vidrios) = &resultado.vidrio else {
        faltan.push("falta el cristal".into());
        return Dinero::CERO;
    };
    let mut total = Dinero::CERO;
    for vidrio in vidrios {
        let Some(articulo) = buscar(catalogo, &vidrio.tipo, faltan) else {
            continue;
        };
        if articulo.unidad != UnidadDeVenta::MetroCuadrado {
            faltan.push(format!("el vidrio {} debe venderse por m²", vidrio.tipo));
            continue;
        }
        let dmm2 = i128::from(vidrio.ancho.decimas_de_mm())
            * i128::from(vidrio.alto.decimas_de_mm())
            * i128::from(vidrio.cantidad);
        let mil = SentidoRedondeo::Cercano
            .dividir(dmm2, 100_000)
            .and_then(|m| i64::try_from(m).ok());
        match mil
            .and_then(|m| coste(articulo, m, recargo))
            .and_then(|c| total.sumar(c))
        {
            Some(suma) => total = suma,
            None => faltan.push(format!("no se pudo calcular el cristal {}", vidrio.tipo)),
        }
    }
    total
}

fn coste_de_accesorios(
    resultado: &ResultadoVentana,
    catalogo: &Catalogo,
    recargo: Porcentaje,
    faltan: &mut Vec<String>,
) -> Dinero {
    let mut total = Dinero::CERO;
    for accesorio in &resultado.accesorios {
        let Some(articulo) = buscar(catalogo, &accesorio.articulo, faltan) else {
            continue;
        };
        let cantidad =
            cantidad_de_accesorio(articulo, accesorio.cantidad, accesorio.unidad_cantidad);
        match cantidad
            .and_then(|m| coste(articulo, m, recargo))
            .and_then(|c| total.sumar(c))
        {
            Some(suma) => total = suma,
            None => faltan.push(format!(
                "{}: la unidad de venta no encaja con la cantidad de la serie",
                accesorio.articulo
            )),
        }
    }
    total
}

/// Tornillos por caja se compran por caja y se gastan por unidad: caja = unidades ÷ unidades por caja.
fn cantidad_de_accesorio(
    articulo: &ArticuloConPrecio,
    cantidad: Decimal,
    unidad: UnidadCantidad,
) -> Option<i64> {
    let mil = cantidad
        .multiplicar(Decimal::desde_entero(1_000))
        .ok()?
        .a_entero()?;
    match (articulo.unidad, unidad) {
        (UnidadDeVenta::Unidad, UnidadCantidad::Unidad)
        | (UnidadDeVenta::Metro, UnidadCantidad::Metro) => Some(mil),
        (UnidadDeVenta::Caja, UnidadCantidad::Unidad) => {
            let por_caja = articulo.unidades_por_caja.filter(|n| *n > 0)?;
            i64::try_from(SentidoRedondeo::Cercano.dividir(i128::from(mil), i128::from(por_caja))?)
                .ok()
        }
        _ => None,
    }
}
