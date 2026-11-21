# Progreso

Estado: **H0 terminado** (2026-11-21). Siguiente: **H1, motor de dominio**.

| Hito | Estado |
|---|---|
| H0 Cimientos | **hecho** (2026-11-21) |
| H1 Motor de dominio | pendiente |
| H2 Catálogo, clientes e importación | pendiente |
| H3 Presupuestos | pendiente |
| H4 Trabajos y orden de taller | pendiente |
| H5 Stock | pendiente |
| H6 Facturación | pendiente |
| H7 Copias y despliegue | pendiente |

## ADR pendientes
- Antes de H1: dónde viven las series en ejecución (propuesta: base de datos, versionadas).
- Antes de H3: ADR-0003, PDF automático en el servidor.

## Registro de decisiones y cambios
### H0. Cimientos (2026-11-21)
**Hecho**
- Workspace `crates/dominio` y `crates/servidor`, Rust 1.97.0 fijado, lints que prohíben `unwrap`, `expect`, `panic` y `unsafe`.
- `just check` (formato, clippy `-D warnings`, pruebas, archivos de hasta 300 líneas), hook `.githooks/pre-push`, `cargo deny` y CI en GitHub Actions (Linux, E2E y Windows). Publicación por etiqueta `v*` con ejecutable Linux estático y `.exe` de Windows.
- Servidor Axum en el **puerto 9090**: `/health`, configuración `alumframe.toml` y variables `ALUMFRAME_*`, modos servidor y local, órdenes `migrar` y `restablecer-contrasena`.
- SQLite con WAL, `synchronous=FULL`, un escritor y cuatro lectores; migración 0001 (usuario, sesion, bloqueo_ip, auditoria y version_esquema), copia previa a cada migración y negativa a arrancar con una base más nueva.
- Acceso: asistente de primer arranque, propietario con contraseña (argon2id) y operario con PIN de 6 cifras en teclado táctil, sesiones con cookie `HttpOnly` (2 h de inactividad, 12 h máximo), CSRF por origen y token, bloqueo progresivo por usuario y por IP (1, 5 y 15 min), permisos en la capa de aplicación (404 para lo ajeno) y auditoría de solo añadir.
- Interfaz: menú de 8 entradas, Inicio vacío con el tablero, Mi día del operario, Ajustes > Usuarios (alta, cambio de PIN y desactivar). IBM Plex y HTMX incluidos.
- Despliegue: servicio systemd e instalador (`despliegue/linux`), acceso directo de Edge en modo aplicación para Windows (`despliegue/windows`).
- Pruebas: 39 unitarias, 19 de integración y 4 E2E.

**Decisiones:** ADR-0004 (Askama), 0005 (zona horaria propia), 0006 (configuración, puerto 9090, servidor Linux), 0007 (dependencias de H0) y 0008 (CI, cobertura y publicación). Tabla `bloqueo_ip` añadida al modelo de datos para el bloqueo por IP que pide `BACKEND.md`.

**Pendiente**
- La cobertura del 90 % del dominio se exigirá en CI desde H1.
- Pendiente de confirmar: caducidad de sesión (2 h y 12 h provisionales) y menú de 8 entradas.
- Los `fixtures/demo` ya venían en el paquete; se usarán en los E2E desde H3.
