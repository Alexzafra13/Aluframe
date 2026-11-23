//! Dónde se guardan las copias: cualquier disco o pendrive con la carpeta `AlumFrame-copias`
//! en su raíz (decisión del propietario). Se reconoce por la carpeta, no por la letra, que en
//! Windows cambia según el orden en que se conectan.

use std::path::{Path, PathBuf};

pub const CARPETA_DE_COPIAS: &str = "AlumFrame-copias";
const LEEME: &str = "LEEME.txt";
const TEXTO_LEEME: &str = "Copias de seguridad automáticas de AlumFrame.\r\n\
Se guardan las 7 últimas diarias y 4 semanales. Para recuperar una, abre AlumFrame,\r\n\
Ajustes > Copias, con este disco conectado.\r\n";

#[derive(Debug, Clone, Default)]
pub struct DiscosDeCopia {
    /// Raíces indicadas en la configuración (`discos_de_copia`).
    pub indicados: Vec<PathBuf>,
    /// Buscar también en las unidades (Windows) o en los montajes de `/media`, `/run/media` y
    /// `/mnt` (Linux). Las pruebas lo desactivan.
    pub buscar_en_el_sistema: bool,
}

impl DiscosDeCopia {
    fn raices(&self) -> Vec<PathBuf> {
        let mut raices = self.indicados.clone();
        if self.buscar_en_el_sistema {
            raices.extend(raices_del_sistema());
        }
        raices.dedup();
        raices
    }

    /// Las carpetas `AlumFrame-copias` de los discos conectados ahora.
    pub fn destinos(&self) -> Vec<PathBuf> {
        self.raices()
            .into_iter()
            .map(|raiz| raiz.join(CARPETA_DE_COPIAS))
            .filter(|carpeta| carpeta.is_dir())
            .collect()
    }

    /// Discos conectados que todavía no tienen la carpeta, para ofrecer prepararlos. El disco
    /// del sistema solo se descarta entre los encontrados: una carpeta indicada a propósito en
    /// la configuración se respeta aunque esté en `C:`.
    pub fn sin_preparar(&self) -> Vec<PathBuf> {
        let encontrados = if self.buscar_en_el_sistema {
            raices_del_sistema()
        } else {
            Vec::new()
        };
        let mut raices = self.indicados.clone();
        raices.extend(
            encontrados
                .into_iter()
                .filter(|raiz| !es_disco_del_sistema(raiz)),
        );
        raices.dedup();
        raices
            .into_iter()
            .filter(|raiz| raiz.is_dir() && !raiz.join(CARPETA_DE_COPIAS).exists())
            .collect()
    }

    /// Solo se prepara un disco de la lista: la ruta llega de un formulario.
    pub fn preparar(&self, raiz: &Path) -> std::io::Result<PathBuf> {
        if !self.sin_preparar().iter().any(|r| r == raiz) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "ese disco ya no está conectado o ya está preparado",
            ));
        }
        let carpeta = raiz.join(CARPETA_DE_COPIAS);
        std::fs::create_dir(&carpeta)?;
        std::fs::write(carpeta.join(LEEME), TEXTO_LEEME)?;
        Ok(carpeta)
    }
}

fn es_disco_del_sistema(raiz: &Path) -> bool {
    cfg!(windows)
        && raiz
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("C:")
}

#[cfg(windows)]
fn raices_del_sistema() -> Vec<PathBuf> {
    ('C'..='Z')
        .map(|letra| PathBuf::from(format!("{letra}:\\")))
        .filter(|raiz| raiz.is_dir())
        .collect()
}

#[cfg(not(windows))]
fn raices_del_sistema() -> Vec<PathBuf> {
    std::fs::read_to_string("/proc/self/mounts")
        .map(|texto| montajes_de_discos(&texto))
        .unwrap_or_default()
}

/// Los puntos de montaje de discos extraíbles o añadidos, de `/proc/self/mounts`.
#[cfg_attr(windows, allow(dead_code))]
fn montajes_de_discos(texto: &str) -> Vec<PathBuf> {
    texto
        .lines()
        .filter_map(|linea| linea.split(' ').nth(1))
        .map(|ruta| ruta.replace("\\040", " "))
        .filter(|ruta| {
            ["/media/", "/run/media/", "/mnt/"]
                .iter()
                .any(|p| ruta.starts_with(p))
        })
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn una_carpeta_indicada_a_mano_se_ofrece_aunque_este_en_el_disco_del_sistema() {
        let raiz = if cfg!(windows) {
            PathBuf::from("C:\\")
        } else {
            std::env::temp_dir()
        };
        let discos = DiscosDeCopia {
            indicados: vec![raiz.clone()],
            buscar_en_el_sistema: false,
        };
        // La raíz de C: de los ejecutores de CI no tiene la carpeta de copias.
        assert_eq!(discos.sin_preparar(), [raiz]);
    }

    #[test]
    fn lee_los_montajes_de_discos_y_no_los_del_sistema() {
        let montajes = "/dev/sda1 / ext4 rw 0 0\n\
                        /dev/sdb1 /media/alex/PEN\\040AZUL vfat rw 0 0\n\
                        /dev/sdc1 /mnt/copias ext4 rw 0 0\n\
                        tmpfs /run/user/1000 tmpfs rw 0 0\n";
        assert_eq!(
            montajes_de_discos(montajes),
            [
                PathBuf::from("/media/alex/PEN AZUL"),
                PathBuf::from("/mnt/copias")
            ]
        );
    }
}
