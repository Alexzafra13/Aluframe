//! Plan de corte de un trabajo entero (decisión 2 del plan de H4): todas sus ventanas juntas,
//! por perfil y color, para aprovechar las barras. Cada pieza recuerda su ventana y su número.

use std::collections::BTreeMap;

use crate::corte::{ErrorCorte, ParametrosDeCorte, PiezaACortar, PlanDeCorte, optimizar_cortes};
use crate::unidades::Longitud;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiezaDeTrabajo {
    pub perfil: String,
    pub color: String,
    pub longitud: Longitud,
    pub cantidad: u32,
    /// Ventana del trabajo (1, 2…) y número de la pieza en su dibujo.
    pub ventana: u32,
    pub numero: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarrasDePerfil {
    pub perfil: String,
    pub color: String,
    pub longitud_barra: Longitud,
    pub plan: Result<PlanDeCorte, ErrorCorte>,
    /// Para cada pieza del plan (por su `id`): ventana y número.
    pub origen: BTreeMap<u32, (u32, u32)>,
}

/// `parametros` lleva sierra, recorte y retal; la longitud de barra es la de cada perfil.
/// Un perfil sin longitud de barra conocida no se planifica (queda fuera del resultado).
pub fn planificar_trabajo(
    piezas: &[PiezaDeTrabajo],
    longitudes_barra: &BTreeMap<String, Longitud>,
    parametros: ParametrosDeCorte,
) -> Vec<BarrasDePerfil> {
    let mut grupos: BTreeMap<(String, String), Vec<&PiezaDeTrabajo>> = BTreeMap::new();
    for pieza in piezas {
        let clave = (pieza.perfil.clone(), pieza.color.clone());
        grupos.entry(clave).or_default().push(pieza);
    }
    grupos
        .into_iter()
        .filter_map(|((perfil, color), del_grupo)| {
            let longitud_barra = *longitudes_barra.get(&perfil)?;
            let mut origen = BTreeMap::new();
            let mut a_cortar = Vec::new();
            for pieza in del_grupo {
                for _ in 0..pieza.cantidad {
                    let id = u32::try_from(a_cortar.len()).unwrap_or(u32::MAX);
                    origen.insert(id, (pieza.ventana, pieza.numero));
                    a_cortar.push(PiezaACortar {
                        id,
                        longitud: pieza.longitud,
                    });
                }
            }
            let parametros = ParametrosDeCorte {
                longitud_barra,
                ..parametros
            };
            Some(BarrasDePerfil {
                plan: optimizar_cortes(&a_cortar, &parametros),
                perfil,
                color,
                longitud_barra,
                origen,
            })
        })
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn cm(valor: i64) -> Longitud {
        Longitud::desde_decimas_de_mm(valor * 100)
    }

    fn pieza(
        (perfil, color): (&str, &str),
        largo: i64,
        cantidad: u32,
        ventana: u32,
    ) -> PiezaDeTrabajo {
        PiezaDeTrabajo {
            perfil: perfil.into(),
            color: color.into(),
            longitud: cm(largo),
            cantidad,
            ventana,
            numero: 5,
        }
    }

    #[test]
    fn junta_las_ventanas_por_perfil_y_color_sin_perder_piezas() {
        let piezas = [
            pieza(("HOJA", "Blanco"), 73, 4, 1),
            pieza(("HOJA", "Blanco"), 73, 4, 2),
            pieza(("HOJA", "Negro"), 73, 4, 3),
            pieza(("MARCO", "Blanco"), 120, 2, 1),
            pieza(("SIN_BARRA", "Blanco"), 50, 1, 1),
        ];
        let barras = BTreeMap::from([("HOJA".to_owned(), cm(640)), ("MARCO".to_owned(), cm(640))]);
        let parametros = ParametrosDeCorte {
            longitud_barra: Longitud::CERO,
            grosor_sierra: Longitud::CERO,
            recorte_extremos: Longitud::CERO,
            retal_minimo: Longitud::CERO,
        };
        let plan = planificar_trabajo(&piezas, &barras, parametros);
        assert_eq!(plan.len(), 3);
        let blanco = plan
            .iter()
            .find(|b| b.perfil == "HOJA" && b.color == "Blanco")
            .unwrap();
        let cortado = blanco.plan.as_ref().unwrap();
        assert_eq!(
            cortado.numero_de_barras(),
            1,
            "8 × 73 cm caben en una barra de 640"
        );
        assert_eq!(cortado.piezas().count(), 8);
        assert_eq!(blanco.origen.values().filter(|(v, _)| *v == 2).count(), 4);
    }
}
