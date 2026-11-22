//! Los acabados (colores) que vienen con el programa, `data/acabados.yaml` (ADR-0019).

use std::collections::BTreeSet;

use crate::dibujo::{ColorDeDibujo, Herrajes, Superficie};
use crate::series::Valor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcabadoIncluido {
    pub nombre: String,
    pub ral: Option<String>,
    pub color: ColorDeDibujo,
    /// El precio de tarifa (sin recargo). Solo uno.
    pub base: bool,
}

/// Todo o nada: con un solo error no se carga ninguno y se dice cuál falla.
pub fn leer_acabados_incluidos(raiz: &Valor) -> Result<Vec<AcabadoIncluido>, String> {
    let lista = match campo(raiz, "acabados") {
        Some(Valor::Lista(lista)) => lista,
        _ => return Err("falta la lista «acabados»".into()),
    };
    let mut acabados = Vec::with_capacity(lista.len());
    let mut nombres = BTreeSet::new();
    for (indice, elemento) in lista.iter().enumerate() {
        let acabado = leer_uno(elemento).map_err(|e| format!("acabado {}: {e}", indice + 1))?;
        if !nombres.insert(acabado.nombre.to_lowercase()) {
            return Err(format!("«{}» está repetido", acabado.nombre));
        }
        acabados.push(acabado);
    }
    if acabados.iter().filter(|a| a.base).count() > 1 {
        return Err("solo un acabado puede ser la base (el precio de tarifa)".into());
    }
    Ok(acabados)
}

fn leer_uno(elemento: &Valor) -> Result<AcabadoIncluido, String> {
    let nombre = texto(elemento, "nombre").ok_or("falta el nombre")?;
    if nombre.trim().is_empty() || nombre.chars().count() > 60 {
        return Err("el nombre debe tener entre 1 y 60 caracteres".into());
    }
    let ral = texto(elemento, "ral");
    if ral.as_ref().is_some_and(|r| r.is_empty() || r.len() > 10) {
        return Err(format!("{nombre}: RAL no válido"));
    }
    let cierres = texto(elemento, "cierres")
        .and_then(|c| Herrajes::desde_texto(&c))
        .ok_or_else(|| format!("{nombre}: «cierres» debe ser blanco o negro"))?;
    let superficie = match texto(elemento, "superficie") {
        None => Superficie::Brillo,
        Some(s) => Superficie::desde_texto(&s).ok_or_else(|| {
            format!("{nombre}: «superficie» debe ser brillo, mate, texturado o anodizado")
        })?,
    };
    let color = texto(elemento, "color")
        .and_then(|c| ColorDeDibujo::nuevo(&c, booleano(elemento, "madera"), cierres))
        .ok_or_else(|| format!("{nombre}: «color» debe ser #rrggbb"))?
        .con_superficie(superficie);
    Ok(AcabadoIncluido {
        nombre,
        ral,
        color,
        base: booleano(elemento, "base"),
    })
}

fn campo<'a>(valor: &'a Valor, nombre: &str) -> Option<&'a Valor> {
    match valor {
        Valor::Mapa(campos) => campos.iter().find(|(k, _)| k == nombre).map(|(_, v)| v),
        _ => None,
    }
}

fn texto(valor: &Valor, nombre: &str) -> Option<String> {
    match campo(valor, nombre)? {
        Valor::Texto(t) | Valor::Numero(t) => Some(t.clone()),
        _ => None,
    }
}

fn booleano(valor: &Valor, nombre: &str) -> bool {
    matches!(campo(valor, nombre), Some(Valor::Booleano(true)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Valor {
        Valor::Texto(s.into())
    }

    fn acabado(nombre: &str, color: &str, base: bool) -> Valor {
        Valor::Mapa(vec![
            ("nombre".into(), t(nombre)),
            ("ral".into(), Valor::Numero("9016".into())),
            ("color".into(), t(color)),
            ("madera".into(), Valor::Booleano(false)),
            ("cierres".into(), t("blanco")),
            ("base".into(), Valor::Booleano(base)),
        ])
    }

    fn lista(acabados: Vec<Valor>) -> Valor {
        Valor::Mapa(vec![("acabados".into(), Valor::Lista(acabados))])
    }

    #[test]
    fn lee_los_acabados_con_su_ral_color_y_cierres() {
        let leidos =
            leer_acabados_incluidos(&lista(vec![acabado("Blanco", "#F1F0EA", true)])).unwrap();
        assert_eq!(leidos[0].ral.as_deref(), Some("9016"));
        assert_eq!(leidos[0].color.perfil(), "#f1f0ea");
        assert_eq!(leidos[0].color.herrajes, Herrajes::Blanco);
        assert!(leidos[0].base);
    }

    #[test]
    fn rechaza_colores_mal_escritos_repetidos_o_dos_bases() {
        let malo = lista(vec![acabado("Verde", "verde", false)]);
        assert!(
            leer_acabados_incluidos(&malo)
                .unwrap_err()
                .contains("#rrggbb")
        );
        let repetido = lista(vec![
            acabado("Negro", "#000000", false),
            acabado("negro", "#000000", false),
        ]);
        assert!(
            leer_acabados_incluidos(&repetido)
                .unwrap_err()
                .contains("repetido")
        );
        let dos_bases = lista(vec![
            acabado("Blanco", "#ffffff", true),
            acabado("Crema", "#eeeeee", true),
        ]);
        assert!(
            leer_acabados_incluidos(&dos_bases)
                .unwrap_err()
                .contains("base")
        );
    }
}
