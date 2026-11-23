use rusqlite::Connection;
use time::{Duration, OffsetDateTime};

use super::Actor;
use crate::autenticacion::{generar_token, huella_de_token};
use crate::bd::consultas::{permisos, sesiones, usuarios};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{formatear_utc, leer_utc};

const INACTIVIDAD_MAXIMA: Duration = Duration::hours(2);
const DURACION_MAXIMA: Duration = Duration::hours(12);
const DURACION_MAXIMA_HASTA_CERRAR: Duration = Duration::days(30);

/// La sesión hasta cerrar el navegador no caduca por inactividad; el tope de 30 días
/// solo evita sesiones olvidadas en un equipo que nunca se apaga.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuracionDeSesion {
    Normal,
    HastaCerrarElNavegador,
}
const INTERVALO_DE_ACTIVIDAD: Duration = Duration::minutes(1);

pub fn crear_sesion(
    con: &Connection,
    usuario_id: i64,
    ahora: OffsetDateTime,
    duracion: DuracionDeSesion,
) -> Result<String, ErrorApp> {
    let hasta_cerrar_navegador = duracion == DuracionDeSesion::HastaCerrarElNavegador;
    let maxima = if hasta_cerrar_navegador {
        DURACION_MAXIMA_HASTA_CERRAR
    } else {
        DURACION_MAXIMA
    };
    let token = generar_token()?;
    let token_csrf = generar_token()?;
    sesiones::borrar_caducadas(con, &formatear_utc(ahora))?;
    sesiones::insertar(
        con,
        &sesiones::NuevaSesion {
            huella: &huella_de_token(&token),
            usuario_id,
            creada_en: &formatear_utc(ahora),
            caduca_en: &formatear_utc(ahora + maxima),
            token_csrf: &token_csrf,
            hasta_cerrar_navegador,
        },
    )?;
    Ok(token)
}

pub async fn identificar(estado: &EstadoApp, token: &str) -> Result<Option<Actor>, ErrorApp> {
    let huella = huella_de_token(token);
    let ahora = estado.ahora();
    let buscada = huella.clone();
    let encontrada = estado
        .bd
        .leer(move |con| buscar_actor(con, &buscada, ahora))
        .await?;
    match encontrada {
        Some((actor, necesita_anotar)) => {
            if necesita_anotar {
                let ahora_texto = formatear_utc(ahora);
                estado
                    .bd
                    .escribir(move |tx| sesiones::anotar_actividad(tx, &huella, &ahora_texto))
                    .await?;
            }
            Ok(Some(actor))
        }
        None => {
            estado
                .bd
                .escribir(move |tx| sesiones::borrar(tx, &huella))
                .await?;
            Ok(None)
        }
    }
}

fn buscar_actor(
    con: &Connection,
    huella: &str,
    ahora: OffsetDateTime,
) -> Result<Option<(Actor, bool)>, ErrorApp> {
    let Some(sesion) = sesiones::buscar(con, huella)? else {
        return Ok(None);
    };
    let ultima = leer_utc(&sesion.ultima_actividad_en).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let caduca = leer_utc(&sesion.caduca_en).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let inactiva = !sesion.hasta_cerrar_navegador && ahora - ultima >= INACTIVIDAD_MAXIMA;
    if ahora >= caduca || inactiva {
        return Ok(None);
    }
    let Some(usuario) = usuarios::buscar_por_id(con, sesion.usuario_id)? else {
        return Ok(None);
    };
    if !usuario.activo {
        return Ok(None);
    }
    let actor = Actor {
        permisos: permisos::leer(con, usuario.id)?,
        id: usuario.id,
        nombre: usuario.nombre,
        rol: usuario.rol,
        token_csrf: sesion.token_csrf,
        huella_sesion: huella.to_owned(),
    };
    Ok(Some((actor, ahora - ultima >= INTERVALO_DE_ACTIVIDAD)))
}

pub async fn cerrar_sesion(estado: &EstadoApp, actor: &Actor) -> Result<(), ErrorApp> {
    let huella = actor.huella_sesion.clone();
    let (usuario_id, ahora) = (actor.id, estado.ahora());
    estado
        .bd
        .escribir(move |tx| {
            sesiones::borrar(tx, &huella)?;
            super::auditar::auditar(
                tx,
                super::auditar::Evento {
                    ahora,
                    autor: Some(usuario_id),
                    accion: "salir",
                    usuario: usuario_id,
                },
            )
        })
        .await
}
