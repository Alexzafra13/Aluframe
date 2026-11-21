use dominio::acceso::Accion;
use dominio::corte::{ParametrosDeCorte, PlanDeCorte, optimizar_cortes};
use dominio::series::{Dato, ResultadoVentana, Serie, calcular_ventana, importar_serie};
use dominio::unidades::{Decimal, Longitud};

use super::{Actor, autorizar};
use crate::bd::consultas::series::{self, VersionGuardada};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::series::leer_yaml;

#[derive(Debug, Clone)]
pub struct SerieGuardada {
    pub serie: Serie,
    pub numero_version: i64,
    pub avisos: i64,
}

#[derive(Debug, Clone)]
pub struct CorteDePerfil {
    pub perfil: String,
    pub plan: PlanDeCorte,
}

#[derive(Debug, Clone)]
pub struct PruebaDeVentana {
    pub resultado: ResultadoVentana,
    pub cortes: Vec<CorteDePerfil>,
    pub avisos_de_corte: Vec<String>,
}

pub async fn listar_series(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<Vec<SerieGuardada>, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let versiones = estado.bd.leer(series::listar_ultimas).await?;
    versiones.into_iter().map(reconstruir).collect()
}

pub async fn cargar_serie(
    estado: &EstadoApp,
    actor: &Actor,
    codigo: String,
) -> Result<SerieGuardada, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    let version = estado
        .bd
        .leer(move |con| series::buscar_ultima(con, &codigo))
        .await?
        .ok_or(ErrorApp::NoEncontrado)?;
    reconstruir(version)
}

/// Lo guardado ya se validó al importar; si ahora no se lee, el programa cambió y hay que saberlo.
fn reconstruir(version: VersionGuardada) -> Result<SerieGuardada, ErrorApp> {
    let fallo = |detalle: String| {
        ErrorApp::Interno(format!(
            "serie {} v{}: {detalle}",
            version.codigo, version.numero
        ))
    };
    let valor = leer_yaml(&version.origen_yaml).map_err(fallo)?;
    let (serie, _) = importar_serie(&valor).map_err(|informe| {
        fallo(
            informe
                .errores
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        )
    })?;
    Ok(SerieGuardada {
        serie,
        numero_version: version.numero,
        avisos: version.avisos,
    })
}

pub fn leer_medida(texto: &str, nombre: &str) -> Result<Decimal, ErrorApp> {
    let normalizado = texto.trim().replace(',', ".");
    Decimal::desde_texto(&normalizado)
        .ok()
        .filter(|d| d.es_positivo())
        .ok_or_else(|| {
            ErrorApp::Validacion(format!(
                "Escribe el {nombre} en cm, por ejemplo 150 o 145,9."
            ))
        })
}

pub fn probar_ventana(
    serie: &Serie,
    tipologia: &str,
    ancho: Decimal,
    alto: Decimal,
) -> Result<PruebaDeVentana, ErrorApp> {
    let resultado = calcular_ventana(serie, tipologia, ancho, alto)
        .map_err(|error| ErrorApp::Validacion(error.to_string()))?;
    let mut avisos_de_corte = Vec::new();
    let Dato::Valor(longitud_barra) = serie.longitud_barra else {
        avisos_de_corte
            .push("No se calculan las barras: falta la longitud de barra de la serie.".into());
        return Ok(PruebaDeVentana {
            resultado,
            cortes: Vec::new(),
            avisos_de_corte,
        });
    };
    avisos_de_corte.push(
        "Grosor de sierra, recorte de extremos y retal mínimo sin confirmar: se toman como 0."
            .into(),
    );
    let parametros = ParametrosDeCorte {
        longitud_barra,
        grosor_sierra: Longitud::CERO,
        recorte_extremos: Longitud::CERO,
        retal_minimo: Longitud::CERO,
    };
    let mut cortes = Vec::new();
    for (perfil, piezas) in resultado.piezas_a_cortar() {
        match optimizar_cortes(&piezas, &parametros) {
            Ok(plan) => cortes.push(CorteDePerfil { perfil, plan }),
            Err(error) => avisos_de_corte.push(format!("{perfil}: {error}")),
        }
    }
    Ok(PruebaDeVentana {
        resultado,
        cortes,
        avisos_de_corte,
    })
}
