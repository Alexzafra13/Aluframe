use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use time::OffsetDateTime;

use crate::aplicacion::copias::discos::DiscosDeCopia;
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
    carpeta_copias: PathBuf,
    preferencias_escritorio: Option<PathBuf>,
    discos_de_copia: Arc<DiscosDeCopia>,
    /// Una copia cada vez: la programada, la de «Copiar ahora» y la de cerrar no se pisan.
    turno_de_copia: Arc<tokio::sync::Mutex<()>>,
    /// Esta es la base de la demostración (ADR-0028), con datos inventados.
    es_demostracion: bool,
    /// En la de verdad: la demostración que se abre desde la pantalla de entrada, aparte.
    demostracion: Option<Arc<EstadoApp>>,
    /// La demostración se rellena una sola vez aunque entren dos a la vez.
    turno_de_demostracion: Arc<tokio::sync::Mutex<()>>,
}

impl EstadoApp {
    pub async fn nuevo(bd: BaseDeDatos, reloj: Arc<dyn Reloj>) -> Result<EstadoApp, ErrorApp> {
        let cuantos = bd.leer(usuarios::contar_usuarios).await?;
        Ok(EstadoApp {
            bd,
            reloj,
            hay_usuarios: Arc::new(AtomicBool::new(cuantos > 0)),
            permitir_datos_demo: false,
            carpeta_copias: std::env::temp_dir().join("alumframe-copias"),
            preferencias_escritorio: None,
            discos_de_copia: Arc::new(DiscosDeCopia::default()),
            turno_de_copia: Arc::new(tokio::sync::Mutex::new(())),
            es_demostracion: false,
            demostracion: None,
            turno_de_demostracion: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    pub fn reloj(&self) -> Arc<dyn Reloj> {
        Arc::clone(&self.reloj)
    }

    /// Marca este estado como el de la demostración (su base es otra, con datos inventados).
    pub fn como_demostracion(mut self) -> EstadoApp {
        self.es_demostracion = true;
        self
    }

    pub fn es_demostracion(&self) -> bool {
        self.es_demostracion
    }

    /// Engancha la demostración a la base de verdad: se entra y se sale desde la entrada.
    pub fn con_demostracion(mut self, demostracion: EstadoApp) -> EstadoApp {
        self.demostracion = Some(Arc::new(demostracion));
        self
    }

    pub fn demostracion(&self) -> Option<&EstadoApp> {
        self.demostracion.as_deref()
    }

    pub fn turno_de_demostracion(&self) -> &tokio::sync::Mutex<()> {
        &self.turno_de_demostracion
    }

    pub fn ahora(&self) -> OffsetDateTime {
        self.reloj.ahora()
    }

    /// Las series de demostración (datos ficticios) solo se aceptan en pruebas y demos.
    pub fn con_datos_demo(mut self, permitir: bool) -> EstadoApp {
        self.permitir_datos_demo = permitir;
        self
    }

    /// Donde se guardan las copias previas a importar y los archivos en tránsito.
    pub fn con_carpeta_copias(mut self, carpeta: PathBuf) -> EstadoApp {
        self.carpeta_copias = carpeta;
        self
    }

    /// Solo en la aplicación de escritorio: activa la pantalla para elegir dónde trabajar.
    pub fn con_preferencias_escritorio(mut self, archivo: PathBuf) -> EstadoApp {
        self.preferencias_escritorio = Some(archivo);
        self
    }

    pub fn con_discos_de_copia(mut self, discos: DiscosDeCopia) -> EstadoApp {
        self.discos_de_copia = Arc::new(discos);
        self
    }

    pub fn discos_de_copia(&self) -> &DiscosDeCopia {
        &self.discos_de_copia
    }

    pub fn turno_de_copia(&self) -> &tokio::sync::Mutex<()> {
        &self.turno_de_copia
    }

    pub fn preferencias_escritorio(&self) -> Option<&Path> {
        self.preferencias_escritorio.as_deref()
    }

    pub fn carpeta_copias(&self) -> &Path {
        &self.carpeta_copias
    }

    pub fn permite_datos_demo(&self) -> bool {
        self.permitir_datos_demo
    }

    pub fn hay_usuarios(&self) -> bool {
        self.hay_usuarios.load(Ordering::Acquire)
    }

    /// Tras importar datos: una base sin usuarios vuelve a pedir el primer arranque.
    pub fn fijar_hay_usuarios(&self, hay: bool) {
        self.hay_usuarios.store(hay, Ordering::Release);
    }

    pub fn anotar_que_hay_usuarios(&self) {
        self.hay_usuarios.store(true, Ordering::Release);
    }
}
