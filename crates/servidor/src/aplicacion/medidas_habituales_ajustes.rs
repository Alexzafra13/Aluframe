//! Ajustes > Taller > Medidas habituales: verlas, añadir y quitar (ADR-0026).

use dominio::acceso::Accion;
use dominio::texto::nombre_legible;
use dominio::unidades::{Longitud, Unidad};
use rusqlite::Connection;

use super::medidas_habituales::{CLAVE, MedidaHabitual, leer_de_la_base, piezas_de};
use super::series::{leer_medida, reconstruir};
use super::{Actor, autorizar};
use crate::bd::consultas::auditoria::{Apunte, registrar};
use crate::bd::consultas::configuracion::{self, Valor};
use crate::bd::consultas::series;
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::formatear_utc;

/// Bastan unas pocas: orientan hasta que haya historial.
const MAXIMO: usize = 30;

pub struct MedidaEnPantalla {
    pub indice: usize,
    pub texto: String,
    /// Por qué no se puede usar (la serie cambió, la medida no la admite…).
    pub aviso: Option<String>,
}

pub struct MedidasHabituales {
    pub medidas: Vec<MedidaEnPantalla>,
    /// Valor «SERIE|tipologia» y texto «Aluprom 21 · Corredera 2 hojas».
    pub tipologias: Vec<(String, String)>,
}

pub async fn ver_medidas_habituales(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<MedidasHabituales, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado.bd.leer(cargar).await
}

fn cargar(con: &Connection) -> Result<MedidasHabituales, ErrorApp> {
    let mut tipologias = Vec::new();
    for version in series::listar_ultimas(con)? {
        let serie = reconstruir(version)?.serie;
        for t in &serie.tipologias {
            let valor = format!("{}|{}", serie.codigo, t.codigo);
            tipologias.push((
                valor,
                format!("{} · {}", nombre_legible(&serie.codigo), t.nombre),
            ));
        }
    }
    let mut medidas = Vec::new();
    for (indice, m) in leer_de_la_base(con)?.into_iter().enumerate() {
        let cm = |dmm| Longitud::desde_decimas_de_mm(dmm).formatear_cm();
        medidas.push(MedidaEnPantalla {
            indice,
            texto: format!(
                "{} · {} · {} × {} cm",
                nombre_legible(&m.serie),
                nombre_legible(&m.tipologia),
                cm(m.ancho_dmm),
                cm(m.alto_dmm)
            ),
            aviso: piezas_de(con, &m)?.err(),
        });
    }
    Ok(MedidasHabituales {
        medidas,
        tipologias,
    })
}

/// Añade una ventana habitual. `tipologia` es «SERIE|tipologia»; las medidas, en cm.
pub async fn anadir_medida_habitual(
    estado: &EstadoApp,
    actor: &Actor,
    (tipologia, ancho, alto): (String, String, String),
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let (serie, tipologia) = tipologia
        .split_once('|')
        .map(|(s, t)| (s.to_owned(), t.to_owned()))
        .ok_or_else(|| ErrorApp::Validacion("Elige la serie y la tipología.".into()))?;
    let en_dmm = |texto: &str, nombre: &str| {
        let medida = leer_medida(texto, nombre)?;
        Longitud::desde_decimal(medida, Unidad::Cm)
            .ok()
            .filter(|l| l.decimas_de_mm() <= 100_000)
            .map(Longitud::decimas_de_mm)
            .ok_or_else(|| ErrorApp::Validacion(format!("El {nombre} no es válido.")))
    };
    let nueva = MedidaHabitual {
        serie,
        tipologia,
        ancho_dmm: en_dmm(&ancho, "ancho")?,
        alto_dmm: en_dmm(&alto, "alto")?,
    };
    cambiar(estado, actor, move |con, mut lista| {
        if let Err(motivo) = piezas_de(con, &nueva)? {
            return Err(ErrorApp::Validacion(format!("No se puede usar: {motivo}.")));
        }
        if lista.contains(&nueva) {
            return Err(ErrorApp::Validacion(
                "Esa medida ya está en la lista.".into(),
            ));
        }
        if lista.len() >= MAXIMO {
            return Err(ErrorApp::Validacion(format!(
                "Como mucho {MAXIMO}: el historial hace el resto."
            )));
        }
        lista.push(nueva);
        Ok(lista)
    })
    .await
}

pub async fn quitar_medida_habitual(
    estado: &EstadoApp,
    actor: &Actor,
    indice: usize,
) -> Result<(), ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    cambiar(estado, actor, move |_, mut lista| {
        if indice >= lista.len() {
            return Err(ErrorApp::NoEncontrado);
        }
        lista.remove(indice);
        Ok(lista)
    })
    .await
}

async fn cambiar(
    estado: &EstadoApp,
    actor: &Actor,
    cambio: impl FnOnce(&Connection, Vec<MedidaHabitual>) -> Result<Vec<MedidaHabitual>, ErrorApp>
    + Send
    + 'static,
) -> Result<(), ErrorApp> {
    let (ahora, usuario_id) = (formatear_utc(estado.ahora()), actor.id);
    estado
        .bd
        .escribir(move |tx| {
            let lista = cambio(tx, leer_de_la_base(tx)?)?;
            let texto = serde_json::to_string(&lista)
                .map_err(|e| ErrorApp::Interno(format!("medidas habituales: {e}")))?;
            let valor = Valor {
                clave: CLAVE,
                valor: &texto,
                ahora: &ahora,
                usuario_id,
            };
            configuracion::guardar(tx, &valor)?;
            let apunte = Apunte {
                cuando_en: &ahora,
                usuario_id: Some(usuario_id),
                accion: "medidas_habituales",
                entidad: "configuracion",
                entidad_id: None,
            };
            registrar(tx, &apunte)
        })
        .await
}
