//! Ajustes > Empresa y Ajustes > Presupuestos. Lo que no se ha elegido queda vacío y la línea
//! lo dice («elige el margen»): nunca se inventa un valor del taller.

use dominio::acceso::Accion;
use dominio::precio::{ModoMargen, ModoPerfiles, ReglasDePrecio};
use dominio::unidades::{Decimal, Dinero, Porcentaje, leer_decimal_es};
use rusqlite::Connection;

use super::auditar::{Cambio, anotar};
use super::{Actor, autorizar};
use crate::bd::consultas::configuracion::{self, Valor};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

/// Los textos tal como se ven y se escriben en el formulario (claves de `configuracion`).
pub const CLAVES: [&str; 22] = [
    "empresa.nombre",
    "empresa.nombre_comercial",
    "empresa.nif",
    "empresa.direccion",
    "empresa.cp",
    "empresa.localidad",
    "empresa.provincia",
    "empresa.telefono",
    "empresa.movil",
    "empresa.email",
    "empresa.web",
    "empresa.iban",
    "empresa.pie",
    "precio.margen_modo",
    "precio.margen",
    "precio.perfiles",
    "precio.merma",
    "precio.mano_de_obra",
    "iva.general",
    "iva.reducido",
    "presupuesto.validez_dias",
    "presupuesto.condiciones",
];

/// El IVA general (21 %) y el reducido (10 %) son los de la ley; cuándo aplica cada uno lo
/// confirma el gestor (OPEN_QUESTIONS). 30 días de validez, porque los precios cambian
/// (propietario, 2026-11-23); dejarlo vacío en Ajustes es «sin fecha».
const POR_DEFECTO: [(&str, &str); 3] = [
    ("iva.general", "21"),
    ("iva.reducido", "10"),
    ("presupuesto.validez_dias", "30"),
];

#[derive(Debug, Clone, Default)]
pub struct TextosDeAjustes(pub Vec<(String, String)>);

impl TextosDeAjustes {
    pub fn valor(&self, clave: &str) -> &str {
        self.0
            .iter()
            .find(|(c, _)| c == clave)
            .map_or("", |(_, v)| v.as_str())
    }

    pub fn cambiar(&mut self, clave: &str, valor: String) {
        if let Some((_, v)) = self.0.iter_mut().find(|(c, _)| c == clave) {
            *v = valor;
        }
    }
}

#[derive(Debug, Clone)]
pub struct AjustesDePresupuesto {
    pub textos: TextosDeAjustes,
    pub reglas: ReglasDePrecio,
    pub mano_de_obra: Dinero,
    pub iva_general: Porcentaje,
    pub iva_reducido: Porcentaje,
    pub validez_dias: Option<i64>,
}

pub fn leer_de_la_base(con: &Connection) -> Result<AjustesDePresupuesto, ErrorApp> {
    let mut textos = Vec::with_capacity(CLAVES.len());
    for clave in CLAVES {
        let guardado = configuracion::leer(con, clave)?;
        let defecto = POR_DEFECTO
            .iter()
            .find(|(c, _)| *c == clave)
            .map(|(_, v)| (*v).to_owned());
        textos.push((clave.to_owned(), guardado.or(defecto).unwrap_or_default()));
    }
    interpretar(TextosDeAjustes(textos))
        .map_err(|e| ErrorApp::Interno(format!("ajustes guardados no válidos: {e}")))
}

pub async fn leer_ajustes_de_presupuesto(
    estado: &EstadoApp,
) -> Result<AjustesDePresupuesto, ErrorApp> {
    estado.bd.leer(leer_de_la_base).await
}

fn porcentaje(
    textos: &TextosDeAjustes,
    clave: &str,
    nombre: &str,
) -> Result<Option<Porcentaje>, String> {
    let texto = textos.valor(clave).trim();
    if texto.is_empty() {
        return Ok(None);
    }
    match Porcentaje::desde_texto_es(texto) {
        Ok(p) if (0..=9_999).contains(&p.puntos_basicos()) => Ok(Some(p)),
        _ => Err(format!("{nombre}: escribe un porcentaje entre 0 y 99,99")),
    }
}

pub fn leer_dinero(texto: &str) -> Option<Dinero> {
    let centimos = leer_decimal_es(texto)
        .ok()?
        .multiplicar(Decimal::desde_entero(100))
        .ok()?
        .a_entero()?;
    (centimos >= 0).then(|| Dinero::desde_centimos(centimos))
}

fn leer_reglas(textos: &TextosDeAjustes) -> Result<ReglasDePrecio, String> {
    let margen = porcentaje(textos, "precio.margen", "Margen")?;
    let modo_margen = match textos.valor("precio.margen_modo") {
        "coste" => Some(ModoMargen::SobreCoste),
        "venta" => Some(ModoMargen::SobreVenta),
        _ => None,
    };
    let merma = porcentaje(textos, "precio.merma", "Merma")?.unwrap_or(Porcentaje::CERO);
    let modo_perfiles = match textos.valor("precio.perfiles") {
        "barras" => Some(ModoPerfiles::BarrasEnteras),
        "metros" => Some(ModoPerfiles::MetrosConMerma(merma)),
        _ => None,
    };
    Ok(ReglasDePrecio {
        modo_perfiles,
        margen: modo_margen.zip(margen),
    })
}

/// Comprueba los textos y los convierte; el primer error se muestra junto al formulario.
pub fn interpretar(mut textos: TextosDeAjustes) -> Result<AjustesDePresupuesto, String> {
    super::datos_de_empresa::normalizar_empresa(&mut textos)?;
    let reglas = leer_reglas(&textos)?;
    let mano = textos.valor("precio.mano_de_obra").trim();
    let mano_de_obra = if mano.is_empty() {
        Some(Dinero::CERO)
    } else {
        leer_dinero(mano)
    }
    .ok_or("Mano de obra: escribe un importe en euros, por ejemplo 25 o 25,50")?;
    let iva_general = porcentaje(&textos, "iva.general", "IVA general")?
        .ok_or("IVA general: escribe el tipo (21 %)")?;
    let iva_reducido = porcentaje(&textos, "iva.reducido", "IVA reducido")?.unwrap_or(iva_general);
    let validez = textos.valor("presupuesto.validez_dias").trim();
    let validez_dias = if validez.is_empty() {
        None
    } else {
        Some(
            validez
                .parse::<i64>()
                .ok()
                .filter(|d| (1..=365).contains(d))
                .ok_or("Validez: escribe un número de días entre 1 y 365")?,
        )
    };
    Ok(AjustesDePresupuesto {
        reglas,
        mano_de_obra,
        iva_general,
        iva_reducido,
        validez_dias,
        textos,
    })
}

pub async fn guardar_ajustes_de_presupuesto(
    estado: &EstadoApp,
    actor: &Actor,
    textos: TextosDeAjustes,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarUsuarios)?;
    let ajustes = interpretar(textos).map_err(ErrorApp::Validacion)?;
    let (ahora, autor) = (estado.ahora(), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let cuando = formatear_utc(ahora);
            for clave in CLAVES {
                let valor = ajustes
                    .textos
                    .valor(clave)
                    .trim()
                    .chars()
                    .take(2_000)
                    .collect::<String>();
                configuracion::guardar(
                    tx,
                    &Valor {
                        clave,
                        valor: &valor,
                        ahora: &cuando,
                        usuario_id: autor,
                    },
                )?;
            }
            anotar(
                tx,
                &Cambio {
                    ahora,
                    autor,
                    accion: "ajustes_de_presupuesto",
                    entidad: "configuracion",
                    entidad_id: 0,
                },
            )
        })
        .await
}
