//! Plan de corte de un trabajo entero (decisión 2 del plan de H4): todas sus ventanas juntas,
//! por serie, perfil y color, para aprovechar las barras. Cada pieza recuerda su ventana y su
//! número. El código del perfil es propio de cada serie («MARCO_LATERAL» es el 21210 en
//! Aluprom 21 y otro perfil en la V-94): sin la serie se mezclarían perfiles distintos.

use std::collections::BTreeMap;

use crate::corte::{
    ErrorCorte, MetodoDeCorte, ParametrosDeCorte, PiezaACortar, PlanDeCorte, RetalDisponible,
    ValoracionDeSobrantes, planificar_cortes,
};
use crate::texto::nombre_legible;
use crate::unidades::Longitud;

/// Qué barra es: el perfil de una serie en un color.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClaveDePerfil {
    pub serie: String,
    pub perfil: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiezaDeTrabajo {
    pub serie: String,
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
    pub serie: String,
    pub perfil: String,
    pub color: String,
    pub longitud_barra: Longitud,
    pub plan: Result<PlanDeCorte, ErrorCorte>,
    /// Para cada pieza del plan (por su `id`): ventana y número.
    pub origen: BTreeMap<u32, (u32, u32)>,
}

impl ClaveDePerfil {
    /// «Marco lateral (Aluprom 21) · Blanco», para la hoja y la revisión de retales.
    pub fn nombre(&self) -> String {
        format!(
            "{} ({}) · {}",
            nombre_legible(&self.perfil),
            nombre_legible(&self.serie),
            self.color
        )
    }
}

impl BarrasDePerfil {
    pub fn clave(&self) -> ClaveDePerfil {
        ClaveDePerfil {
            serie: self.serie.clone(),
            perfil: self.perfil.clone(),
            color: self.color.clone(),
        }
    }
}

impl PiezaDeTrabajo {
    pub fn clave(&self) -> ClaveDePerfil {
        ClaveDePerfil {
            serie: self.serie.clone(),
            perfil: self.perfil.clone(),
            color: self.color.clone(),
        }
    }
}

/// Retales del stock de cada barra, para usarlos antes de abrir barras nuevas.
pub type RetalesPorPerfil = BTreeMap<ClaveDePerfil, Vec<RetalDisponible>>;

/// Cómo se valoran los sobrantes de cada perfil, por (serie, perfil): lo que se suele cortar no
/// depende del color.
pub type ValoracionesPorPerfil = BTreeMap<(String, String), ValoracionDeSobrantes>;

/// `parametros` lleva sierra, recorte y retal; la longitud de barra es la de cada serie.
/// Una serie sin longitud de barra conocida no se planifica (queda fuera del resultado).
/// Un sobrante por debajo del retal mínimo de Ajustes nunca cuenta como útil.
pub fn planificar_trabajo(
    piezas: &[PiezaDeTrabajo],
    (longitudes_barra, retales): (&BTreeMap<String, Longitud>, &RetalesPorPerfil),
    parametros: ParametrosDeCorte,
    (metodo, valoraciones): (MetodoDeCorte, &ValoracionesPorPerfil),
) -> Vec<BarrasDePerfil> {
    let mut grupos: BTreeMap<ClaveDePerfil, Vec<&PiezaDeTrabajo>> = BTreeMap::new();
    for pieza in piezas {
        grupos.entry(pieza.clave()).or_default().push(pieza);
    }
    grupos
        .into_iter()
        .filter_map(|(clave, del_grupo)| {
            let longitud_barra = *longitudes_barra.get(&clave.serie)?;
            let (a_cortar, origen) = una_por_unidad(&del_grupo);
            let parametros = ParametrosDeCorte {
                longitud_barra,
                ..parametros
            };
            let disponibles = retales.get(&clave).map_or(&[][..], Vec::as_slice);
            let mut valoracion = valoraciones
                .get(&(clave.serie.clone(), clave.perfil.clone()))
                .cloned()
                .unwrap_or_default();
            valoracion.minimo_util = valoracion.minimo_util.max(parametros.retal_minimo);
            Some(BarrasDePerfil {
                plan: planificar_cortes(
                    (&a_cortar, disponibles),
                    &parametros,
                    (metodo, &valoracion),
                ),
                serie: clave.serie,
                perfil: clave.perfil,
                color: clave.color,
                longitud_barra,
                origen,
            })
        })
        .collect()
}

/// Cada unidad es una pieza a cortar, numerada; `origen` recuerda su ventana y su número.
fn una_por_unidad(piezas: &[&PiezaDeTrabajo]) -> (Vec<PiezaACortar>, BTreeMap<u32, (u32, u32)>) {
    let mut origen = BTreeMap::new();
    let mut a_cortar = Vec::new();
    for pieza in piezas {
        for _ in 0..pieza.cantidad {
            let id = u32::try_from(a_cortar.len()).unwrap_or(u32::MAX);
            origen.insert(id, (pieza.ventana, pieza.numero));
            a_cortar.push(PiezaACortar {
                id,
                longitud: pieza.longitud,
            });
        }
    }
    (a_cortar, origen)
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
            serie: "SERIE_A".into(),
            perfil: perfil.into(),
            color: color.into(),
            longitud: cm(largo),
            cantidad,
            ventana,
            numero: 5,
        }
    }

    fn sin_sierra() -> ParametrosDeCorte {
        ParametrosDeCorte {
            longitud_barra: Longitud::CERO,
            grosor_sierra: Longitud::CERO,
            recorte_extremos: Longitud::CERO,
            retal_minimo: Longitud::CERO,
        }
    }

    #[test]
    fn junta_las_ventanas_por_perfil_y_color_sin_perder_piezas() {
        let mut sin_barra = pieza(("MARCO", "Blanco"), 50, 1, 1);
        sin_barra.serie = "SIN_BARRA".into();
        let piezas = [
            pieza(("HOJA", "Blanco"), 73, 4, 1),
            pieza(("HOJA", "Blanco"), 73, 4, 2),
            pieza(("HOJA", "Negro"), 73, 4, 3),
            pieza(("MARCO", "Blanco"), 120, 2, 1),
            pieza(("SIN_PERFIL", "Blanco"), 50, 1, 1),
            sin_barra,
        ];
        let barras = BTreeMap::from([("SERIE_A".to_owned(), cm(640))]);
        let plan = planificar_trabajo(
            &piezas,
            (&barras, &BTreeMap::new()),
            sin_sierra(),
            (MetodoDeCorte::MejorAprovechamiento, &BTreeMap::new()),
        );
        assert_eq!(plan.len(), 4);
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

    /// Fallo corregido: el mismo código de perfil en dos series son perfiles distintos y no se
    /// pueden cortar de la misma barra ni compartir sus retales.
    #[test]
    fn el_mismo_codigo_de_perfil_en_dos_series_no_se_mezcla() {
        let mut de_otra_serie = pieza(("MARCO", "Blanco"), 120, 2, 2);
        de_otra_serie.serie = "SERIE_B".into();
        let piezas = [pieza(("MARCO", "Blanco"), 120, 2, 1), de_otra_serie];
        let barras = BTreeMap::from([
            ("SERIE_A".to_owned(), cm(640)),
            ("SERIE_B".to_owned(), cm(500)),
        ]);
        let retal = RetalDisponible {
            id: 1,
            longitud: cm(300),
        };
        let clave_b = ClaveDePerfil {
            serie: "SERIE_B".into(),
            perfil: "MARCO".into(),
            color: "Blanco".into(),
        };
        let retales = BTreeMap::from([(clave_b.clone(), vec![retal])]);
        let plan = planificar_trabajo(
            &piezas,
            (&barras, &retales),
            sin_sierra(),
            (MetodoDeCorte::Rapido, &BTreeMap::new()),
        );
        assert_eq!(plan.len(), 2, "un grupo por serie");
        let a = plan.iter().find(|g| g.serie == "SERIE_A").unwrap();
        let b = plan.iter().find(|g| g.clave() == clave_b).unwrap();
        assert_eq!(a.longitud_barra, cm(640));
        assert_eq!(b.longitud_barra, cm(500));
        let corte_a = a.plan.as_ref().unwrap();
        assert!(
            corte_a.barras.iter().all(|barra| barra.retal.is_none()),
            "el retal es de B"
        );
        assert_eq!(corte_a.piezas().count(), 2);
        assert_eq!(b.plan.as_ref().unwrap().barras[0].retal, Some(1));
    }
}
