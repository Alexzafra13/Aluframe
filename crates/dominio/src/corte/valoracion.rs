//! Cuánto vale lo que sobra de un plan, para elegir entre repartos que gastan las mismas barras
//! nuevas (ADR-0026). Primero cuenta cuántas piezas de las que se suelen cortar saldrían de los
//! sobrantes; a igualdad, prefiere el sobrante junto (un retal largo mejor que dos cortos).

use crate::unidades::Longitud;

/// Dos retales a menos de esto se consideran casi iguales: el segundo vale la mitad, para no
/// llenar el almacén de trozos que sirven para lo mismo.
const CASI_IGUALES_DMM: i64 = 1_000;
const ESCALA: i128 = 1_000;
const BITS_DE_CONCENTRACION: u32 = 48;

/// Lo que se sabe de un perfil para valorar sus sobrantes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValoracionDeSobrantes {
    /// Por debajo de este largo un sobrante es desecho: no cuenta como retal útil.
    pub minimo_util: Longitud,
    /// Lo que se suele cortar de este perfil: largo de la pieza y su peso (cuántas veces, con
    /// más peso lo reciente). Vacío si no hay medidas habituales ni historial.
    pub habituales: Vec<(Longitud, u32)>,
}

impl ValoracionDeSobrantes {
    /// Mayor es mejor. Uso esperado de los sobrantes (en milésimas de pieza habitual) y, a
    /// igualdad, la suma de sus cuadrados, que premia juntar lo que sobra.
    pub fn valorar(&self, sobrantes: &[Longitud], sierra: Longitud) -> i128 {
        let concentracion: i128 = sobrantes
            .iter()
            .map(|s| i128::from(s.decimas_de_mm().max(0)).pow(2))
            .sum();
        (self.uso_esperado(sobrantes, sierra) << BITS_DE_CONCENTRACION) + concentracion
    }

    /// Para cada sobrante útil, cuántas piezas habituales saldrían de él de media; uno casi
    /// igual a otro ya contado vale la mitad (y el tercero, la cuarta parte…).
    fn uso_esperado(&self, sobrantes: &[Longitud], sierra: Longitud) -> i128 {
        let peso_total: i128 = self.habituales.iter().map(|(_, p)| i128::from(*p)).sum();
        if peso_total == 0 {
            return 0;
        }
        let mut utiles: Vec<i64> = sobrantes
            .iter()
            .map(|s| s.decimas_de_mm())
            .filter(|s| *s > 0 && *s >= self.minimo_util.decimas_de_mm())
            .collect();
        utiles.sort_unstable_by(|a, b| b.cmp(a));
        let mut total = 0;
        for (indice, largo) in utiles.iter().enumerate() {
            let parecidos = utiles[..indice]
                .iter()
                .filter(|anterior| *anterior - largo < CASI_IGUALES_DMM)
                .count();
            let piezas = self.piezas_que_salen(*largo, sierra) * ESCALA / peso_total;
            total += piezas >> parecidos.min(64);
        }
        total
    }

    /// Suma, para cada medida habitual, su peso por cuántas veces cabe en el sobrante.
    fn piezas_que_salen(&self, largo: i64, sierra: Longitud) -> i128 {
        self.habituales
            .iter()
            .filter_map(|(pieza, peso)| {
                let consumo = pieza.decimas_de_mm() + sierra.decimas_de_mm();
                (consumo > 0).then(|| i128::from(largo / consumo) * i128::from(*peso))
            })
            .sum()
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn cm(valor: i64) -> Longitud {
        Longitud::desde_decimas_de_mm(valor * 100)
    }

    fn valorar(v: &ValoracionDeSobrantes, sobrantes: &[i64]) -> i128 {
        let lista: Vec<Longitud> = sobrantes.iter().map(|s| cm(*s)).collect();
        v.valorar(&lista, Longitud::CERO)
    }

    #[test]
    fn sin_historial_prefiere_un_retal_largo_a_dos_cortos() {
        let v = ValoracionDeSobrantes::default();
        assert!(valorar(&v, &[190, 10]) > valorar(&v, &[100, 100]));
        assert!(valorar(&v, &[300, 40]) > valorar(&v, &[200, 140]));
    }

    #[test]
    fn con_historial_prefiere_el_sobrante_en_que_caben_las_piezas_habituales() {
        let v = ValoracionDeSobrantes {
            minimo_util: Longitud::CERO,
            habituales: vec![(cm(110), 3)],
        };
        assert!(
            valorar(&v, &[115, 85]) > valorar(&v, &[100, 100]),
            "en 115 cabe una de 110; en 100 ninguna"
        );
        assert!(valorar(&v, &[230]) > valorar(&v, &[115, 115]));
    }

    #[test]
    fn un_retal_casi_igual_a_otro_vale_la_mitad() {
        let v = ValoracionDeSobrantes {
            minimo_util: Longitud::CERO,
            habituales: vec![(cm(50), 1)],
        };
        let parecidos = v.uso_esperado(&[cm(120), cm(115)], Longitud::CERO);
        let distintos = v.uso_esperado(&[cm(120), cm(100)], Longitud::CERO);
        assert_eq!(
            parecidos,
            2_000 + 1_000,
            "de cada uno salen 2, pero 115 vale la mitad"
        );
        assert_eq!(distintos, 2_000 + 2_000);
        let medias = ValoracionDeSobrantes {
            minimo_util: Longitud::CERO,
            habituales: vec![(cm(100), 1), (cm(60), 1)],
        };
        assert_eq!(
            medias.uso_esperado(&[cm(70)], Longitud::CERO),
            500,
            "la mitad de lo que se corta cabe en 70"
        );
    }

    #[test]
    fn lo_que_no_llega_al_minimo_util_es_desecho_y_la_sierra_cuenta() {
        let v = ValoracionDeSobrantes {
            minimo_util: cm(50),
            habituales: vec![(cm(40), 1)],
        };
        assert_eq!(v.uso_esperado(&[cm(45)], Longitud::CERO), 0);
        assert_eq!(v.uso_esperado(&[cm(80)], Longitud::CERO), 2_000);
        let sierra = Longitud::desde_decimas_de_mm(20);
        assert_eq!(v.uso_esperado(&[cm(80)], sierra), 1_000);
    }
}
