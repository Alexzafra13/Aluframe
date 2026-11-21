use crate::texto::normalizar_clave;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoArticulo {
    Perfil,
    Vidrio,
    Herraje,
    Accesorio,
    Kit,
    Otro,
}

impl TipoArticulo {
    pub const TODOS: [TipoArticulo; 6] = [
        TipoArticulo::Perfil,
        TipoArticulo::Vidrio,
        TipoArticulo::Herraje,
        TipoArticulo::Accesorio,
        TipoArticulo::Kit,
        TipoArticulo::Otro,
    ];

    pub fn desde_texto(texto: &str) -> Option<TipoArticulo> {
        TipoArticulo::TODOS
            .into_iter()
            .find(|t| t.como_texto() == normalizar_clave(texto))
    }

    pub fn como_texto(self) -> &'static str {
        match self {
            TipoArticulo::Perfil => "perfil",
            TipoArticulo::Vidrio => "vidrio",
            TipoArticulo::Herraje => "herraje",
            TipoArticulo::Accesorio => "accesorio",
            TipoArticulo::Kit => "kit",
            TipoArticulo::Otro => "otro",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnidadDeVenta {
    Barra,
    Metro,
    MetroCuadrado,
    Unidad,
    Caja,
}

impl UnidadDeVenta {
    pub const TODAS: [UnidadDeVenta; 5] = [
        UnidadDeVenta::Barra,
        UnidadDeVenta::Metro,
        UnidadDeVenta::MetroCuadrado,
        UnidadDeVenta::Unidad,
        UnidadDeVenta::Caja,
    ];

    /// Admite las abreviaturas habituales de las tarifas: «ml», «m²», «ud».
    pub fn desde_texto(texto: &str) -> Option<UnidadDeVenta> {
        match normalizar_clave(texto).trim_end_matches('.') {
            "barra" | "barras" => Some(UnidadDeVenta::Barra),
            "metro" | "metros" | "m" | "ml" => Some(UnidadDeVenta::Metro),
            "m2" | "m²" | "metro_cuadrado" | "metros_cuadrados" => {
                Some(UnidadDeVenta::MetroCuadrado)
            }
            "unidad" | "unidades" | "ud" | "uds" | "u" => Some(UnidadDeVenta::Unidad),
            "caja" | "cajas" => Some(UnidadDeVenta::Caja),
            _ => None,
        }
    }

    /// El valor guardado en la base de datos (columna `articulo.unidad`).
    pub fn como_texto(self) -> &'static str {
        match self {
            UnidadDeVenta::Barra => "barra",
            UnidadDeVenta::Metro => "metro",
            UnidadDeVenta::MetroCuadrado => "m2",
            UnidadDeVenta::Unidad => "unidad",
            UnidadDeVenta::Caja => "caja",
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            UnidadDeVenta::MetroCuadrado => "m²",
            otra => otra.como_texto(),
        }
    }

    pub fn es_barra(self) -> bool {
        self == UnidadDeVenta::Barra
    }
}
