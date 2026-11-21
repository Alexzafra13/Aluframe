# ADR-0008: integración continua, cobertura y publicación

**Estado:** aceptado (2026-11-21, H0).

## Decisión
- **CI en GitHub Actions** (el repositorio ya está en GitHub): `just check` y `cargo deny check` en Linux, E2E con Playwright contra el ejecutable de release y compilación más pruebas en Windows.
- **Publicación:** al subir una etiqueta `vX.Y.Z` se generan dos paquetes en la página de versiones: Linux estático (`x86_64-unknown-linux-musl`, funciona en cualquier distribución sin dependencias) con el servicio systemd y el instalador, y Windows (`alumframe.exe` en modo local) con el script del acceso directo.
- **Cobertura:** `cargo-llvm-cov` (`just cobertura`, mínimo 90 % en `crates/dominio`). Se exige en CI desde H1 (96,9 % al activarla).
- **Antigüedad de dependencias:** `scripts/comprobar_antiguedad.py` consulta crates.io y lista lo publicado hace menos de 14 días. No va en CI para no depender de la red de crates.io en cada ejecución.

## Alternativas
Compilar el ejecutable de Linux con glibc: depende de la versión de la distribución del servidor. musl evita ese riesgo a cambio de un SQLite algo más lento al asignar memoria, irrelevante con 5 usuarios.
