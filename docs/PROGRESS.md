# Progreso

Estado: **H1 terminado** (2026-11-21). Siguiente: **H2, catálogo, clientes e importación de tarifas**.

| Hito | Estado |
|---|---|
| H0 Cimientos | **hecho** (2026-11-21) |
| H1 Motor de dominio | **hecho** (2026-11-21) |
| H2 Catálogo, clientes e importación | pendiente |
| H3 Presupuestos | pendiente |
| H4 Trabajos y orden de taller | pendiente |
| H5 Stock | pendiente |
| H6 Facturación | pendiente |
| H7 Copias y despliegue | pendiente |

## ADR pendientes
- Antes de H3: ADR-0003, PDF automático en el servidor.

## Registro de decisiones y cambios
### H0. Cimientos (2026-11-21)
**Hecho**
- Workspace `crates/dominio` y `crates/servidor`, Rust 1.97.0 fijado, lints que prohíben `unwrap`, `expect`, `panic` y `unsafe`.
- `just check` (formato, clippy `-D warnings`, pruebas, archivos de hasta 300 líneas), hook `.githooks/pre-push`, `cargo deny` y CI en GitHub Actions (Linux, E2E y Windows). Publicación por etiqueta `v*` con ejecutable Linux estático y `.exe` de Windows.
- Servidor Axum en el **puerto 9090**: `/health`, configuración `alumframe.toml` y variables `ALUMFRAME_*`, modos servidor y local, órdenes `migrar` y `restablecer-contrasena`.
- SQLite con WAL, `synchronous=FULL`, un escritor y cuatro lectores; migraciones 0001 (usuario, sesion, bloqueo_ip, auditoria y version_esquema) y 0002 (sesión hasta cerrar el navegador), copia previa a cada migración y negativa a arrancar con una base más nueva.
- Acceso: asistente de primer arranque, propietario con contraseña (argon2id) y operario con PIN de 6 cifras en teclado táctil, sesiones con cookie `HttpOnly` (2 h de inactividad y 12 h máximo, o hasta cerrar el navegador si el propietario lo marca al entrar), CSRF por origen y token, bloqueo progresivo por usuario y por IP (1, 5 y 15 min), permisos en la capa de aplicación (404 para lo ajeno) y auditoría de solo añadir.
- Interfaz: menú de 8 entradas, Inicio vacío con el tablero, Mi día del operario, Ajustes > Usuarios (alta, cambio de PIN y desactivar). IBM Plex y HTMX incluidos.
- Despliegue: servicio systemd e instalador (`despliegue/linux`), acceso directo de Edge en modo aplicación para Windows (`despliegue/windows`).
- Pruebas: 41 unitarias, 22 de integración y 4 E2E.

**Decisiones:** ADR-0004 (Askama), 0005 (zona horaria propia), 0006 (configuración, puerto 9090, servidor Linux), 0007 (dependencias de H0) y 0008 (CI, cobertura y publicación). Tabla `bloqueo_ip` añadida al modelo de datos para el bloqueo por IP que pide `BACKEND.md`.

**Pendiente**
- La cobertura del 90 % del dominio se exigirá en CI desde H1.
- Pendiente de confirmar: menú de 8 entradas.
- Los `fixtures/demo` ya venían en el paquete; se usarán en los E2E desde H3.

### H1. Motor de dominio (2026-11-21)
**Hecho**
- Unidades exactas: `Decimal` de punto fijo (4 decimales, sin `f64`), `Longitud` en décimas de mm, `Dinero` en céntimos y redondeo abajo, arriba o cercano.
- Evaluador de fórmulas propio: `+ - * /`, paréntesis, `minimo`, `maximo`, `redondear`, `techo`, `suelo` y `si(...)` con comparaciones; límites de 200 caracteres y 16 niveles; errores en español; pruebas de propiedades.
- Series como datos: lectura con `yaml-rust2` a un árbol neutro y validación completa en el dominio (apartado 11 de FORMATO_SERIES.md), con todos los errores a la vez y avisos por cada `CONFIRMAR`.
- Cálculo de una ventana: piezas numeradas, vidrio, accesorios desglosados y sumados, límites y lista de pendientes; la línea sale **incompleta** mientras falte cualquier dato.
- Optimizador Best Fit Decreasing con grosor de sierra, recorte y retal; pruebas de propiedades y comparación con la solución óptima en casos pequeños.
- **Tests dorados:** las 3 ventanas de `fixtures/real/` coinciden con el cálculo (criterio de aceptación).
- Servidor: migración 0003 (series versionadas), Catálogo > Series, importación en 3 pasos con vista previa y pantalla **«Probar una ventana»** con el plan de corte.
- Pruebas: 132 entre unitarias e integración y 5 E2E; cobertura del dominio 95,7 %, exigida en CI (mínimo 90 %).

**Decisiones:** ADR-0009 (series como YAML versionado en la base de datos) y ADR-0010 (`yaml-rust2`). Bisagras de la abatible confirmadas por el propietario. **Revisión de ADR-0001 tras H1:** el motor salió sin atascos; se mantiene Rust.

**Pendiente**
- El **coste y el precio** de la ventana se calculan en H2/H3, cuando existan tarifas y se confirme si el margen va sobre coste o sobre venta.
- Grosor de sierra, recorte de extremos y retal mínimo: se usan 0 hasta que el taller los dé (`OPEN_QUESTIONS.md`).
- Validar una serie (pasarla de borrador a validada) con ventanas reales: acción explícita que llegará con el punto de validación tras H3.
