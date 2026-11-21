# ADR-0007: dependencias añadidas en H0

**Estado:** aceptado (2026-11-21, H0). Todas las versiones tenían más de 14 días al fijarlas (`just antiguedad`).

| Crate | Requisito | Alternativa más simple y por qué no | Licencia |
|---|---|---|---|
| `tracing-subscriber` (sin `env-filter`) | Que `tracing` escriba los registros por la salida estándar (systemd los guarda en el journal) | `println!`: sin niveles ni campos | MIT |
| `getrandom` | 256 bits aleatorios para sesiones y tokens CSRF | Ninguna en la biblioteca estándar; ya la trae `argon2` | MIT o Apache-2.0 |
| `sha2` | Guardar solo la huella del token de sesión; la usará también la huella de los documentos emitidos (H4, H6) | Ninguna en la biblioteca estándar; misma familia RustCrypto que `argon2` | MIT o Apache-2.0 |
| `toml` | Archivo de configuración (ADR-0006) | Formato propio | MIT o Apache-2.0 |
| `rust-embed` | Plantillas no, pero sí CSS, JS y fuentes dentro del ejecutable (ya prevista en `CLAUDE.md`) | `include_bytes!` archivo a archivo: más código al añadir recursos | MIT |
| `tower` (solo pruebas) | Llamar al router en las pruebas de integración sin abrir un puerto | Arrancar el servidor real: más lento | MIT |

Lo que se ha escrito a mano para no añadir dependencias: lectura de cookies, codificación hexadecimal, comparación de tokens en tiempo constante, tipo de contenido de los recursos y la zona horaria (ADR-0005).

**Cómo se sustituyen:** cada una está usada en un único archivo (`main.rs`, `autenticacion/tokens.rs`, `config.rs`, `http/recursos.rs`).
