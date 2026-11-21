# ADR-0006: configuración, modos de ejecución y puerto

**Estado:** aceptado (2026-11-21, H0).

## Decisión
- **Archivo `alumframe.toml`** junto al ejecutable (o la ruta de `ALUMFRAME_CONFIG`), con `modo`, `puerto`, `carpeta_datos` y `abrir_navegador`. Las variables `ALUMFRAME_MODO`, `ALUMFRAME_PUERTO`, `ALUMFRAME_CARPETA_DATOS` y `ALUMFRAME_ABRIR_NAVEGADOR` mandan sobre el archivo. Claves desconocidas = error (evita erratas silenciosas).
- **Puerto 9090** por defecto (decisión del propietario).
- **Modo por defecto:** `local` en Windows (doble clic, solo 127.0.0.1, abre el navegador) y `servidor` en Linux (0.0.0.0, sin navegador).
- **Servidor del taller: Linux con systemd** (decisión del propietario). El PC de la oficina usa Windows y entra por navegador. Desaparece el ADR pendiente del envoltorio de servicio de Windows.
- Las rutas relativas cuentan desde la carpeta del ejecutable, no desde la carpeta actual (un servicio arranca en `/`).

## Por qué TOML
Es legible y admite comentarios. La crate `toml` se integra con `serde`, que ya usamos. Alternativa descartada: solo variables de entorno (incómodo en Windows con doble clic) o un formato propio (más código que mantener).
