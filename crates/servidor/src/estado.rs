use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use time::OffsetDateTime;

use crate::bd::BaseDeDatos;
use crate::bd::consultas::usuarios;
use crate::errores::ErrorApp;
use crate::reloj::Reloj;

#[derive(Clone)]
pub struct EstadoApp {
    pub bd: BaseDeDatos,
    reloj: Arc<dyn Reloj>,
    hay_usuarios: Arc<AtomicBool>,
    permitir_datos_demo: bool,
}

impl EstadoApp {
    pub async fn nuevo(bd: BaseDeDatos, reloj: Arc<dyn Reloj>) -> Result<EstadoApp, ErrorApp> {
        let cuantos = bd.leer(usuarios::contar_usuarios).await?;
        Ok(EstadoApp {
            bd,
            reloj,
            hay_usuarios: Arc::new(AtomicBool::new(cuantos > 0)),
            permitir_datos_demo: false,
        })
    }

    pub fn ahora(&self) -> OffsetDateTime {
        self.reloj.ahora()
    }

    /// Las series de demostración (datos ficticios) solo se aceptan en pruebas y demos.
    pub fn con_datos_demo(mut self, permitir: bool) -> EstadoApp {
        self.permitir_datos_demo = permitir;
        self
    }

    pub fn permite_datos_demo(&self) -> bool {
        self.permitir_datos_demo
    }

    pub fn hay_usuarios(&self) -> bool {
        self.hay_usuarios.load(Ordering::Acquire)
    }

    pub fn anotar_que_hay_usuarios(&self) {
        self.hay_usuarios.store(true, Ordering::Release);
    }
}
