use super::DatosDeDibujo;
use super::persiana::PersianaEnPlano;
use crate::series::TipoDibujo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lado {
    Izquierda,
    Derecha,
}

impl Lado {
    pub fn desde_texto(texto: &str) -> Option<Lado> {
        match texto {
            "izquierda" => Some(Lado::Izquierda),
            "derecha" => Some(Lado::Derecha),
            _ => None,
        }
    }
}

/// Rectángulo en décimas de mm (coordenadas enteras: el dibujo no usa coma flotante).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub ancho: i64,
    pub alto: i64,
}

impl Rect {
    pub fn dentro(self, margen: i64) -> Rect {
        Rect {
            x: self.x + margen,
            y: self.y + margen,
            ancho: (self.ancho - 2 * margen).max(1),
            alto: (self.alto - 2 * margen).max(1),
        }
    }

    pub fn centro_y(self) -> i64 {
        self.y + self.alto / 2
    }

    pub fn derecha(self) -> i64 {
        self.x + self.ancho
    }

    pub fn abajo(self) -> i64 {
        self.y + self.alto
    }
}

/// Proporciones fijas: es un esquema para entender la ventana, no un plano de fabricación.
pub struct Plano {
    /// La medida escrita: con persiana, el conjunto; sin ella, el propio marco.
    pub contorno: Rect,
    pub marco: Rect,
    pub hojas: Vec<Rect>,
    pub grosor_marco: i64,
    pub grosor_hoja: i64,
    pub margen: i64,
    pub tipo: TipoDibujo,
    /// Altura (coordenada) del parteluces, para colocar el número de su pieza sobre la barra.
    pub parteluces_eje: Option<i64>,
    pub persiana: Option<PersianaEnPlano>,
}

impl Plano {
    pub fn nuevo(datos: &DatosDeDibujo) -> Plano {
        let ancho = datos.ancho.decimas_de_mm().clamp(100, 1_000_000);
        let alto = datos.alto.decimas_de_mm().clamp(100, 1_000_000);
        let contorno = Rect {
            x: 0,
            y: 0,
            ancho,
            alto,
        };
        let (persiana, marco) = match &datos.persiana {
            Some(p) => {
                let (partes, marco) = super::persiana::repartir(contorno, p);
                (Some(partes), marco)
            }
            None => (None, contorno),
        };
        let menor = marco.ancho.min(marco.alto);
        let grosor_marco = (menor / 16).max(1);
        let grosor_hoja = (menor / 22).max(1);
        let interior = marco.dentro(grosor_marco);
        let hojas = repartir_hojas(interior, datos, grosor_hoja);
        let mut plano = Plano {
            contorno,
            marco,
            hojas,
            grosor_marco,
            grosor_hoja,
            margen: ancho.max(alto) / 7,
            tipo: datos.dibujo.tipo,
            parteluces_eje: None,
            persiana,
        };
        plano.parteluces_eje = datos
            .parteluces
            .as_ref()
            .map(|p| super::parteluces::eje(&plano, p));
        plano
    }

    pub fn tamano_de_letra(&self) -> i64 {
        (self.contorno.ancho.max(self.contorno.alto) / 30).max(1)
    }
}

/// Las correderas se solapan en el encuentro; las abatibles reparten el hueco sin solape.
fn repartir_hojas(interior: Rect, datos: &DatosDeDibujo, grosor_hoja: i64) -> Vec<Rect> {
    let n = i64::from(datos.dibujo.hojas.max(1));
    if matches!(datos.dibujo.tipo, TipoDibujo::Fija | TipoDibujo::Mosquitera) {
        return vec![interior];
    }
    let solape = if datos.dibujo.tipo == TipoDibujo::Corredera {
        grosor_hoja
    } else {
        0
    };
    let ancho_hoja = (interior.ancho + (n - 1) * solape) / n;
    (0..n)
        .map(|i| Rect {
            x: interior.x + i * (ancho_hoja - solape),
            y: interior.y,
            ancho: ancho_hoja,
            alto: interior.alto,
        })
        .collect()
}
