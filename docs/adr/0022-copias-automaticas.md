# ADR-0022: copias de seguridad automáticas

**Estado:** aceptado (2026-11-22, decisiones del propietario en `docs/PROGRESS.md`).

## Decisión
- **Cuándo:** cada día a las **14:00** (hora de Madrid), **al cerrar** el programa (ventana de escritorio cerrada, Ctrl+C o `systemctl stop`, que envía SIGTERM) salvo si se hizo una hace menos de 10 minutos, y **al arrancar** si falta la de las 14:00 (apagón, equipo apagado). Un programador mira cada minuto; tras un fallo reintenta a los 30 minutos (por ejemplo, cuando se conecta el pendrive).
- **Dónde:** siempre en el equipo (`copias/automaticas` de la carpeta de datos) y en cada disco conectado que tenga una carpeta **`AlumFrame-copias`** en su raíz. Se reconoce por la carpeta, no por la letra ni el punto de montaje. En Windows se miran las unidades C: a Z:; en Linux, los montajes bajo `/media`, `/run/media` y `/mnt` (de `/proc/self/mounts`) y lo indicado en `discos_de_copia` (configuración o `ALUMFRAME_DISCOS_COPIA`). Ajustes > Copias ofrece preparar un disco conectado (crea la carpeta y un `LEEME.txt`).
- **Cómo:** `VACUUM INTO` a un archivo temporal (nunca se copia la base en caliente), `integrity_check` y huella de versión (`comprobar_copia`), y se renombra. En cada disco: se copia a `.tmp`, `sync_all`, se renombra y se **vuelve a comprobar allí**. Mismo formato que Exportar (ADR-0016): un `.alumframe` se puede importar en cualquier equipo.
- **Nombre:** `alumframe-AAAA-MM-DD-HHMM.alumframe` (hora de Madrid). Solo se borran archivos con ese nombre: nada más del disco se toca.
- **Retención** (`dominio::copias`): la más reciente de cada uno de los **7** últimos días con copia y, de lo anterior, la más reciente de cada una de **4** semanas (lunes a domingo). Unas 5 semanas hacia atrás con 11 archivos. Las 12 mensuales de BACKEND 8 se descartan: el propietario eligió 7 + 4.
- **Registro:** `copias/automaticas/registro.json` (última copia completa, último intento, aviso y discos). Va en un archivo y no en la base para que no retroceda al restaurar una copia antigua.
- **Avisos:** en el Inicio, si el último intento falló, si no hay disco (la copia solo está en el equipo) o si la última copia en un disco tiene 3 días o más.
- **Restaurar:** Ajustes > Copias > «Restaurar…» lleva la copia elegida (del equipo o de un disco conectado) a la revisión de la importación (ADR-0016): se ve qué contiene y, al confirmar, lo actual se guarda antes.
- **Servicio:** en Linux, systemd con `ReadWritePaths` para `/mnt`, `/media` y `/run/media`. En Windows el programa se usa como aplicación de escritorio (ADR-0015), que copia al cerrar y al arrancar; el **servicio de Windows** no hace falta mientras el servidor sea Linux y se aplaza hasta que alguien lo necesite (requeriría un envoltorio y su propio ADR).
- La copia fuera del taller (el PC descarga la del servidor) queda para después del MVP; mientras, se recomienda alternar dos pendrives y guardar uno fuera.

## Requisitos del ADR-0002
- Sin dependencias nuevas: se activa la característica `time` de Tokio (temporizador del programador).
- **Pruebas:** `dominio/src/copias_pruebas.rs` (calendario, nombres, cuándo toca, reintentos, al cerrar, retención); `tests/copias.rs` (sin disco avisa, copia comprobada en el disco, 30 días dejan 7 + 4, **restaurar en un equipo limpio reproduce los datos**, solo se restaura una copia de la lista); `tests/copias_pantallas.rs`; E2E `h7-copias` con una carpeta como pendrive. Comprobado a mano: SIGTERM hace la copia de cierre.
