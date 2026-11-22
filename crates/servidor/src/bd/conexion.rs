use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};

use crate::errores::ErrorApp;

const LECTORES: usize = 4;

/// Un único escritor tras un Mutex y un grupo pequeño de lectores (BACKEND.md, apartado 6).
#[derive(Clone)]
pub struct BaseDeDatos {
    escritor: Arc<Mutex<Connection>>,
    lectores: Arc<Vec<Mutex<Connection>>>,
    siguiente_lector: Arc<AtomicUsize>,
}

impl BaseDeDatos {
    pub fn abrir(ruta: &Path) -> Result<BaseDeDatos, rusqlite::Error> {
        let escritor = Connection::open(ruta)?;
        configurar(&escritor)?;
        escritor.pragma_update(None, "journal_mode", "WAL")?;
        let mut lectores = Vec::with_capacity(LECTORES);
        for _ in 0..LECTORES {
            let lector = Connection::open_with_flags(ruta, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            configurar(&lector)?;
            lectores.push(Mutex::new(lector));
        }
        Ok(BaseDeDatos::con(escritor, lectores))
    }

    pub fn en_memoria() -> Result<BaseDeDatos, rusqlite::Error> {
        let conexion = Connection::open_in_memory()?;
        configurar(&conexion)?;
        Ok(BaseDeDatos::con(conexion, Vec::new()))
    }

    fn con(escritor: Connection, lectores: Vec<Mutex<Connection>>) -> BaseDeDatos {
        BaseDeDatos {
            escritor: Arc::new(Mutex::new(escritor)),
            lectores: Arc::new(lectores),
            siguiente_lector: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn escribir<T, F>(&self, operacion: F) -> Result<T, ErrorApp>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction) -> Result<T, ErrorApp> + Send + 'static,
    {
        let escritor = Arc::clone(&self.escritor);
        en_hilo_bloqueante(move || {
            let mut conexion = escritor.lock().map_err(|_| envenenado())?;
            let transaccion = conexion.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let resultado = operacion(&transaccion)?;
            transaccion.commit()?;
            Ok(resultado)
        })
        .await
    }

    pub async fn leer<T, F>(&self, consulta: F) -> Result<T, ErrorApp>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, ErrorApp> + Send + 'static,
    {
        let base = self.clone();
        en_hilo_bloqueante(move || base.leer_ya(consulta)).await
    }

    fn leer_ya<T>(
        &self,
        consulta: impl FnOnce(&Connection) -> Result<T, ErrorApp>,
    ) -> Result<T, ErrorApp> {
        if self.lectores.is_empty() {
            let conexion = self.escritor.lock().map_err(|_| envenenado())?;
            return consulta(&conexion);
        }
        let indice = self.siguiente_lector.fetch_add(1, Ordering::Relaxed) % self.lectores.len();
        let lector = self.lectores.get(indice).ok_or_else(envenenado)?;
        let conexion = lector.lock().map_err(|_| envenenado())?;
        consulta(&conexion)
    }

    /// La conexión de escritura sin transacción: para copiar o restaurar la base entera.
    pub async fn con_escritor_entero<T, F>(&self, operacion: F) -> Result<T, ErrorApp>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, ErrorApp> + Send + 'static,
    {
        let escritor = Arc::clone(&self.escritor);
        en_hilo_bloqueante(move || {
            let mut conexion = escritor.lock().map_err(|_| envenenado())?;
            operacion(&mut conexion)
        })
        .await
    }

    pub fn con_escritor<T>(
        &self,
        operacion: impl FnOnce(&mut Connection) -> T,
    ) -> Result<T, ErrorApp> {
        let mut conexion = self.escritor.lock().map_err(|_| envenenado())?;
        Ok(operacion(&mut conexion))
    }
}

fn configurar(conexion: &Connection) -> Result<(), rusqlite::Error> {
    conexion.pragma_update(None, "foreign_keys", "ON")?;
    conexion.pragma_update(None, "synchronous", "FULL")?;
    conexion.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}

fn envenenado() -> ErrorApp {
    ErrorApp::Interno("conexión a la base de datos no disponible".into())
}

async fn en_hilo_bloqueante<T: Send + 'static>(
    tarea: impl FnOnce() -> Result<T, ErrorApp> + Send + 'static,
) -> Result<T, ErrorApp> {
    tokio::task::spawn_blocking(tarea)
        .await
        .map_err(|error| ErrorApp::Interno(format!("tarea interrumpida: {error}")))?
}
