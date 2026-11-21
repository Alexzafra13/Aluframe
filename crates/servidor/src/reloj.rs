use std::sync::Mutex;

use time::{Duration, OffsetDateTime};

pub trait Reloj: Send + Sync {
    fn ahora(&self) -> OffsetDateTime;
}

pub struct RelojDelSistema;

impl Reloj for RelojDelSistema {
    fn ahora(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

pub struct RelojFijo {
    instante: Mutex<OffsetDateTime>,
}

impl RelojFijo {
    pub fn new(instante: OffsetDateTime) -> RelojFijo {
        RelojFijo {
            instante: Mutex::new(instante),
        }
    }

    pub fn avanzar(&self, duracion: Duration) {
        if let Ok(mut instante) = self.instante.lock() {
            *instante += duracion;
        }
    }
}

impl Reloj for RelojFijo {
    fn ahora(&self) -> OffsetDateTime {
        self.instante
            .lock()
            .map(|instante| *instante)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH)
    }
}
