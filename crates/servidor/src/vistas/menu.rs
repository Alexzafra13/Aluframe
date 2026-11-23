pub struct EntradaMenu {
    pub nombre: &'static str,
    pub ruta: &'static str,
    pub seccion: &'static str,
}

pub struct GrupoMenu {
    pub titulo: &'static str,
    pub entradas: &'static [EntradaMenu],
}

const fn entrada(nombre: &'static str, ruta: &'static str, seccion: &'static str) -> EntradaMenu {
    EntradaMenu {
        nombre,
        ruta,
        seccion,
    }
}

/// Menú de 8 entradas de PANTALLAS.md (apartado 1); Ajustes y Salir van al pie.
pub const MENU: &[GrupoMenu] = &[
    GrupoMenu {
        titulo: "Vender",
        entradas: &[
            entrada("Inicio", "/", "inicio"),
            entrada("Mi día", "/mi-dia", "mi-dia"),
            entrada("Clientes", "/clientes", "clientes"),
            entrada("Presupuestos", "/presupuestos", "presupuestos"),
            entrada("Trabajos", "/trabajos", "trabajos"),
        ],
    },
    GrupoMenu {
        titulo: "Taller",
        entradas: &[
            entrada("Stock", "/stock", "stock"),
            entrada("Catálogo", "/catalogo", "catalogo"),
        ],
    },
    GrupoMenu {
        titulo: "Cobrar",
        entradas: &[entrada("Facturas", "/facturas", "facturas")],
    },
];
