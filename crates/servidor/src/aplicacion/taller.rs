use dominio::acceso::Accion;
use dominio::corte::{MetodoDeCorte, ParametrosDeCorte};
use dominio::unidades::{Decimal, Longitud, Unidad};
use rusqlite::Connection;

use super::{Actor, autorizar};
use crate::bd::consultas::auditoria::{Apunte, registrar};
use crate::bd::consultas::configuracion::{self, Valor};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

const GROSOR_SIERRA: &str = "taller.grosor_sierra_dmm";
const RECORTE_EXTREMOS: &str = "taller.recorte_extremos_dmm";
const RETAL_MINIMO: &str = "taller.retal_minimo_dmm";
const METODO_DE_CORTE: &str = "taller.metodo_corte";

/// 2 mm lo dio el propietario como valor provisional (2026-11-21); recorte y retal, sin datos.
const POR_DEFECTO: [(&str, i64); 3] = [
    (GROSOR_SIERRA, 20),
    (RECORTE_EXTREMOS, 0),
    (RETAL_MINIMO, 0),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AjustesDeTaller {
    pub grosor_sierra: Longitud,
    pub recorte_extremos: Longitud,
    pub retal_minimo: Longitud,
    pub confirmados: bool,
    /// Sin elegir, el mejor aprovechamiento (ADR-0026).
    pub metodo: MetodoDeCorte,
}

impl AjustesDeTaller {
    pub fn parametros_de_corte(&self, longitud_barra: Longitud) -> ParametrosDeCorte {
        ParametrosDeCorte {
            longitud_barra,
            grosor_sierra: self.grosor_sierra,
            recorte_extremos: self.recorte_extremos,
            retal_minimo: self.retal_minimo,
        }
    }
}

pub struct TextosDeTaller {
    pub grosor_sierra_mm: String,
    pub recorte_extremos_mm: String,
    pub retal_minimo_cm: String,
}

pub async fn leer_ajustes_de_taller(estado: &EstadoApp) -> Result<AjustesDeTaller, ErrorApp> {
    estado.bd.leer(leer_de_la_base).await
}

pub(crate) fn leer_de_la_base(con: &Connection) -> Result<AjustesDeTaller, ErrorApp> {
    let mut valores = [0i64; 3];
    let mut confirmados = true;
    for (indice, (clave, defecto)) in POR_DEFECTO.iter().enumerate() {
        let guardado = configuracion::leer(con, clave)?.and_then(|v| v.parse().ok());
        confirmados &= guardado.is_some();
        valores[indice] = guardado.unwrap_or(*defecto);
    }
    let [grosor, recorte, retal] = valores.map(Longitud::desde_decimas_de_mm);
    Ok(AjustesDeTaller {
        grosor_sierra: grosor,
        recorte_extremos: recorte,
        retal_minimo: retal,
        confirmados,
        metodo: configuracion::leer(con, METODO_DE_CORTE)?
            .and_then(|texto| MetodoDeCorte::desde_texto(&texto))
            .unwrap_or_default(),
    })
}

/// Cambia cómo se reparten las piezas en las barras (la interfaz deja elegir, SPEC 6).
pub async fn guardar_metodo_de_corte(
    estado: &EstadoApp,
    actor: &Actor,
    texto: String,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let metodo = MetodoDeCorte::desde_texto(&texto)
        .ok_or_else(|| ErrorApp::Validacion("Elige cómo se reparten las piezas.".into()))?;
    let (ahora, usuario_id) = (formatear_utc(estado.ahora()), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let nuevo = Valor {
                clave: METODO_DE_CORTE,
                valor: metodo.como_texto(),
                ahora: &ahora,
                usuario_id,
            };
            configuracion::guardar(tx, &nuevo)?;
            registrar(tx, &apunte_de_ajustes(&ahora, usuario_id))
        })
        .await
}

fn apunte_de_ajustes(ahora: &str, usuario_id: i64) -> Apunte<'_> {
    Apunte {
        cuando_en: ahora,
        usuario_id: Some(usuario_id),
        accion: "ajustes_de_taller",
        entidad: "configuracion",
        entidad_id: None,
    }
}

fn leer_medida(
    texto: &str,
    unidad: Unidad,
    maximo_dmm: i64,
    nombre: &str,
) -> Result<Longitud, ErrorApp> {
    let error = || ErrorApp::Validacion(format!("El {nombre} no es válido."));
    let numero = Decimal::desde_texto(&texto.trim().replace(',', ".")).map_err(|_| error())?;
    Longitud::desde_decimal(numero, unidad)
        .ok()
        .filter(|l| (0..=maximo_dmm).contains(&l.decimas_de_mm()))
        .ok_or_else(error)
}

type ValoresDeTaller = [(&'static str, Longitud); 3];

fn validar_textos(textos: &TextosDeTaller) -> Result<ValoresDeTaller, ErrorApp> {
    Ok([
        (
            GROSOR_SIERRA,
            leer_medida(
                &textos.grosor_sierra_mm,
                Unidad::Mm,
                100,
                "grosor de sierra",
            )?,
        ),
        (
            RECORTE_EXTREMOS,
            leer_medida(
                &textos.recorte_extremos_mm,
                Unidad::Mm,
                1_000,
                "recorte de extremos",
            )?,
        ),
        (
            RETAL_MINIMO,
            leer_medida(&textos.retal_minimo_cm, Unidad::Cm, 64_000, "retal mínimo")?,
        ),
    ])
}

pub async fn guardar_ajustes_de_taller(
    estado: &EstadoApp,
    actor: &Actor,
    textos: TextosDeTaller,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let valores = validar_textos(&textos)?;
    let (ahora, usuario_id) = (formatear_utc(estado.ahora()), actor.id);
    estado
        .bd
        .escribir(move |tx| guardar_en_la_base(tx, &valores, &ahora, usuario_id))
        .await
}

fn guardar_en_la_base(
    tx: &Connection,
    valores: &ValoresDeTaller,
    ahora: &str,
    usuario_id: i64,
) -> Result<(), ErrorApp> {
    for (clave, longitud) in valores {
        let valor = longitud.decimas_de_mm().to_string();
        let nuevo = Valor {
            clave,
            valor: &valor,
            ahora,
            usuario_id,
        };
        configuracion::guardar(tx, &nuevo)?;
    }
    registrar(tx, &apunte_de_ajustes(ahora, usuario_id))
}
