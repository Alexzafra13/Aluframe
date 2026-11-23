use dominio::acceso::Accion;
use dominio::corte::{PlanDeCorte, ValoracionDeSobrantes, planificar_cortes};
use dominio::series::{Dato, ResultadoVentana, Serie, calcular_ventana, importar_serie};
use dominio::unidades::Decimal;

use super::taller::AjustesDeTaller;
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

/// Cuántas referencias de cada serie tienen ya artículo (y por tanto precio posible en H3).
pub async fn contar_referencias_enlazadas(
    estado: &EstadoApp,
    actor: &Actor,
) -> Result<std::collections::BTreeMap<String, (i64, i64)>, ErrorApp> {
    autorizar(actor, Accion::GestionarCatalogo)?;
    estado.bd.leer(series::contar_referencias_enlazadas).await
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
pub(crate) fn reconstruir(version: VersionGuardada) -> Result<SerieGuardada, ErrorApp> {
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
    (tipologia, ancho, alto): (&str, Decimal, Decimal),
    taller: &AjustesDeTaller,
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
    avisos_de_corte.push(describir_ajustes(taller));
    let parametros = taller.parametros_de_corte(longitud_barra);
    // Una ventana sola: lo que cuenta son las barras; los sobrantes se valoran en el trabajo.
    let sin_historial = ValoracionDeSobrantes::default();
    let mut cortes = Vec::new();
    for (perfil, piezas) in resultado.piezas_a_cortar() {
        match planificar_cortes((&piezas, &[]), &parametros, (taller.metodo, &sin_historial)) {
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

fn describir_ajustes(taller: &AjustesDeTaller) -> String {
    let texto = format!(
        "Sierra {} mm, recorte de extremos {} mm y retal mínimo {} cm (Ajustes > Taller).",
        taller.grosor_sierra.formatear_mm(),
        taller.recorte_extremos.formatear_mm(),
        taller.retal_minimo.formatear_cm()
    );
    if taller.confirmados {
        texto
    } else {
        format!("{texto} Son valores provisionales: confírmalos allí.")
    }
}
