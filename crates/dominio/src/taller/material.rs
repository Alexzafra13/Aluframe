//! Material que se encarga a medida para cada ventana del trabajo (ADR-0035): el cristal y lo que
//! la serie marca `se_pide` (la persiana). Lo normal es pendiente → pedido; «recibido» existe por
//! si se quiere apuntar, pero no hace falta (propietario, 2026-11-25).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDeMaterial {
    Pendiente,
    Pedido,
    Recibido,
}

impl EstadoDeMaterial {
    pub fn desde_texto(texto: &str) -> Option<EstadoDeMaterial> {
        match texto {
            "pendiente" => Some(EstadoDeMaterial::Pendiente),
            "pedido" => Some(EstadoDeMaterial::Pedido),
            "recibido" => Some(EstadoDeMaterial::Recibido),
            _ => None,
        }
    }

    pub fn como_texto(self) -> &'static str {
        match self {
            EstadoDeMaterial::Pendiente => "pendiente",
            EstadoDeMaterial::Pedido => "pedido",
            EstadoDeMaterial::Recibido => "recibido",
        }
    }

    /// Para la pantalla: «Pendiente de pedir», «Pedido», «Recibido».
    pub fn etiqueta(self) -> &'static str {
        match self {
            EstadoDeMaterial::Pendiente => "Pendiente de pedir",
            EstadoDeMaterial::Pedido => "Pedido",
            EstadoDeMaterial::Recibido => "Recibido",
        }
    }

    pub fn falta_pedir(self) -> bool {
        self == EstadoDeMaterial::Pendiente
    }
}

/// Medida y cantidad de un material, para saber si cambió al modificar las medidas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MedidaDeMaterial {
    pub cantidad: u32,
    pub ancho_dmm: Option<i64>,
    pub alto_dmm: Option<i64>,
}

/// Al modificar las medidas de una ventana con el trabajo en marcha: lo pendiente toma la medida
/// nueva sin más; lo que ya se pidió (o llegó) con otra medida vuelve a pendiente y lo dice, para
/// que no se monte un cristal o una persiana de la medida vieja.
pub fn estado_tras_cambiar_la_medida(
    estado: EstadoDeMaterial,
    antes: MedidaDeMaterial,
    ahora: MedidaDeMaterial,
) -> (EstadoDeMaterial, Option<&'static str>) {
    if estado.falta_pedir() || antes == ahora {
        return (estado, None);
    }
    (
        EstadoDeMaterial::Pendiente,
        Some("La medida cambió después de pedirlo: hay que volver a pedirlo."),
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const MEDIDA: MedidaDeMaterial = MedidaDeMaterial {
        cantidad: 2,
        ancho_dmm: Some(5_020),
        alto_dmm: Some(10_690),
    };

    #[test]
    fn los_estados_se_leen_y_se_escriben_igual() {
        for estado in [
            EstadoDeMaterial::Pendiente,
            EstadoDeMaterial::Pedido,
            EstadoDeMaterial::Recibido,
        ] {
            assert_eq!(
                EstadoDeMaterial::desde_texto(estado.como_texto()),
                Some(estado)
            );
        }
        assert_eq!(EstadoDeMaterial::desde_texto("perdido"), None);
    }

    #[test]
    fn lo_pendiente_toma_la_medida_nueva_sin_avisar() {
        let nueva = MedidaDeMaterial {
            ancho_dmm: Some(6_000),
            ..MEDIDA
        };
        let resultado = estado_tras_cambiar_la_medida(EstadoDeMaterial::Pendiente, MEDIDA, nueva);
        assert_eq!(resultado, (EstadoDeMaterial::Pendiente, None));
    }

    #[test]
    fn lo_pedido_con_otra_medida_vuelve_a_pendiente_y_lo_dice() {
        let nueva = MedidaDeMaterial {
            alto_dmm: Some(9_000),
            ..MEDIDA
        };
        for estado in [EstadoDeMaterial::Pedido, EstadoDeMaterial::Recibido] {
            let (nuevo, nota) = estado_tras_cambiar_la_medida(estado, MEDIDA, nueva);
            assert_eq!(nuevo, EstadoDeMaterial::Pendiente);
            assert!(nota.is_some_and(|n| n.contains("volver a pedirlo")));
        }
    }

    #[test]
    fn lo_pedido_con_la_misma_medida_se_queda_como_estaba() {
        let resultado = estado_tras_cambiar_la_medida(EstadoDeMaterial::Pedido, MEDIDA, MEDIDA);
        assert_eq!(resultado, (EstadoDeMaterial::Pedido, None));
    }
}
