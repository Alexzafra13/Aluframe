use dominio::acceso::{Rol, calcular_segundos_de_bloqueo, validar_pin};
use rusqlite::Connection;
use time::{Duration, OffsetDateTime};

use super::auditar::{Evento, auditar};
use super::sesion::crear_sesion;
use crate::autenticacion::comprobar_clave;
use crate::bd::consultas::{bloqueo_ip, usuarios};
use crate::errores::ErrorApp;
use crate::estado::EstadoApp;
use crate::fecha::{formatear_utc, leer_utc};

pub struct SolicitudDeEntrada {
    pub usuario_id: i64,
    pub clave: String,
    pub ip: String,
}

#[derive(Debug)]
pub enum ResultadoDeEntrada {
    Correcta { token: String, rol: Rol },
    Incorrecta,
    Bloqueada { minutos: u64 },
}

pub async fn entrar(
    estado: &EstadoApp,
    solicitud: SolicitudDeEntrada,
) -> Result<ResultadoDeEntrada, ErrorApp> {
    let ahora = estado.ahora();
    estado
        .bd
        .escribir(move |tx| intentar_entrar(tx, &solicitud, ahora))
        .await
}

fn intentar_entrar(
    tx: &Connection,
    solicitud: &SolicitudDeEntrada,
    ahora: OffsetDateTime,
) -> Result<ResultadoDeEntrada, ErrorApp> {
    let intento = Intento {
        tx,
        solicitud,
        ahora,
    };
    let bloqueo = bloqueo_ip::buscar(tx, &solicitud.ip)?;
    let fallos_ip = bloqueo.as_ref().map_or(0, |b| b.intentos_fallidos);
    if let Some(minutos) = minutos_restantes(bloqueo.and_then(|b| b.bloqueado_hasta), ahora) {
        return Ok(ResultadoDeEntrada::Bloqueada { minutos });
    }
    let usuario = usuarios::buscar_por_id(tx, solicitud.usuario_id)?.filter(|u| u.activo);
    let Some(usuario) = usuario else {
        intento.anotar_fallo_de_ip(fallos_ip)?;
        return Ok(ResultadoDeEntrada::Incorrecta);
    };
    if let Some(minutos) = minutos_restantes(usuario.bloqueado_hasta.clone(), ahora) {
        return Ok(ResultadoDeEntrada::Bloqueada { minutos });
    }
    if es_clave_correcta(&usuario, &solicitud.clave) {
        intento.aceptar(&usuario)
    } else {
        intento.anotar_fallo_de_ip(fallos_ip)?;
        intento.rechazar(&usuario)
    }
}

struct Intento<'a> {
    tx: &'a Connection,
    solicitud: &'a SolicitudDeEntrada,
    ahora: OffsetDateTime,
}

impl Intento<'_> {
    fn aceptar(&self, usuario: &usuarios::FilaUsuario) -> Result<ResultadoDeEntrada, ErrorApp> {
        usuarios::reiniciar_fallos(self.tx, usuario.id)?;
        bloqueo_ip::olvidar(self.tx, &self.solicitud.ip)?;
        let token = crear_sesion(self.tx, usuario.id, self.ahora)?;
        self.auditar(Some(usuario.id), "entrar", usuario.id)?;
        Ok(ResultadoDeEntrada::Correcta {
            token,
            rol: usuario.rol,
        })
    }

    fn rechazar(&self, usuario: &usuarios::FilaUsuario) -> Result<ResultadoDeEntrada, ErrorApp> {
        let hasta = bloqueo_tras_fallo(usuario.intentos_fallidos, self.ahora);
        usuarios::anotar_fallo(self.tx, usuario.id, hasta.as_deref())?;
        self.auditar(None, "entrada_fallida", usuario.id)?;
        Ok(match minutos_restantes(hasta, self.ahora) {
            Some(minutos) => ResultadoDeEntrada::Bloqueada { minutos },
            None => ResultadoDeEntrada::Incorrecta,
        })
    }
    fn anotar_fallo_de_ip(&self, fallos_previos: u32) -> Result<(), ErrorApp> {
        let hasta = bloqueo_tras_fallo(fallos_previos, self.ahora);
        bloqueo_ip::anotar_fallo(self.tx, &self.solicitud.ip, hasta.as_deref())
    }

    fn auditar(&self, autor: Option<i64>, accion: &str, usuario: i64) -> Result<(), ErrorApp> {
        let evento = Evento {
            ahora: self.ahora,
            autor,
            accion,
            usuario,
        };
        auditar(self.tx, evento)
    }
}

fn es_clave_correcta(usuario: &usuarios::FilaUsuario, clave: &str) -> bool {
    let formato_valido = match usuario.rol {
        Rol::Operario => validar_pin(clave).is_ok(),
        Rol::Propietario => !clave.is_empty(),
    };
    formato_valido && comprobar_clave(clave, &usuario.clave_hash)
}

fn bloqueo_tras_fallo(fallos_previos: u32, ahora: OffsetDateTime) -> Option<String> {
    let segundos = calcular_segundos_de_bloqueo(fallos_previos.saturating_add(1))?;
    let segundos = i64::try_from(segundos).unwrap_or(i64::MAX);
    Some(formatear_utc(ahora + Duration::seconds(segundos)))
}

fn minutos_restantes(hasta: Option<String>, ahora: OffsetDateTime) -> Option<u64> {
    let hasta = leer_utc(&hasta?)?;
    let segundos = (hasta - ahora).whole_seconds();
    (segundos > 0).then(|| u64::try_from(segundos).unwrap_or(0).div_ceil(60))
}
