# ADR-0035: compilación, pruebas y publicación más rápidas

**Estado:** aceptado (2026-11-25, aprobado por el propietario). Sustituye la parte de publicación de ADR-0008.

## Contexto
El propietario nota que cada versión tarda mucho y que trabajar con Claude se alarga (y gasta más) mientras compila. Medido el 2026-11-25 (máquina de la nube, 4 núcleos):
- Cambiar una línea y recompilar las pruebas costaba **24–28 s**: el programa en sí, 5 s; el resto era enlazar **45 ejecutables de prueba** (uno por archivo de `crates/servidor/tests/`), de unos 175 MB cada uno. `target/` llegaba a **9,5 GB**.
- Las pruebas tardaban 28 s, y 10 s eran esperas de la aplicación de escritorio, que comprobaba cada 10 s si la ventana seguía abierta aunque la prueba solo espere 2 s de silencio.
- Recompilar el ejecutable de `release` tras cambiar una línea tarda **122 s** (LTO completo con una sola unidad de código), y la guía «Antes de publicar» lo pedía para la E2E local.
- Publicar tardaba **12 min**: Windows esperaba a Linux y compilaba sin caché (4 min de pruebas y 4 del ejecutable), mientras CI repetía a la vez las mismas pruebas. Además, una versión podía publicarse con CI en rojo (ya pasó, ver `PROGRESS.md`).
- Cada recálculo del editor volvía a leer el YAML de la serie y a probar todas sus combinaciones: unos 5 ms por serie, y el editor carga todas al abrirse.

## Decisión
1. **Un solo ejecutable de pruebas de integración:** `crates/servidor/tests/integracion/main.rs` con un módulo por archivo. Las pruebas no cambian. Las carpetas temporales ya llevaban un nombre único, así que pueden correr juntas.
2. **`debug = "line-tables-only"` en el perfil `dev`:** los mensajes de error conservan archivo y línea, y los ejecutables pesan menos. Para depurar paso a paso, `debug = true` en local.
3. **Perfil `e2e`** (hereda de `release`, sin LTO, 16 unidades, incremental): `just e2e` lo usa en local y recompila en ~4 s. CI sigue pasando la E2E con el ejecutable de `release`, el que se publica.
4. **La espera de la aplicación de escritorio** se comprueba cada 10 s o cada cuarto del silencio, lo que sea menor: igual en el uso real y 0,5 s en las pruebas.
5. **CI y publicación en un solo workflow** (`ci.yml`): Linux (musl + E2E sobre ese mismo ejecutable), Windows (pruebas + instalador) y las comprobaciones van en paralelo y con caché; el trabajo `publicar` solo corre en `main` si **todo** ha pasado y la versión de `Cargo.toml` aún no tiene etiqueta, y publica los ejecutables ya probados, sin compilar de nuevo. Los commits que solo tocan `docs/`, `design/` o `.md` no lanzan CI. En una rama, un push nuevo cancela la ejecución anterior.
6. **Series leídas en memoria** (`aplicacion/series_leidas.rs`), recordadas por su texto y no por el id, porque en un proceso conviven la base real y la de la demostración. Una versión guardada no cambia nunca; validarla sigue tocando al importarla.
7. **Métricas:** `just metricas` (y `just metricas --limpia`) mide tiempos y tamaños y da una fila para `docs/METRICAS.md`, que se rellena en cada versión para ver si algo empeora.

## Alternativas descartadas
- **Bajar el LTO del ejecutable publicado:** ahorraría ~2 min por versión, pero el ejecutable vuelve a crecer (decisión de la 0.13.1).
- **Partir `servidor` en varios crates:** el programa en sí recompila en 5 s; no compensa la reorganización.
- **sccache, mold u otras herramientas:** dependencias nuevas para el entorno; Rust ya enlaza con `lld` en Linux y lo anterior basta.
- **E2E en paralelo:** las pruebas comparten un servidor y una base y dependen del orden; queda para cuando duela.

## Consecuencias
- Medidas del antes y el después en `docs/METRICAS.md`.
- Una prueba de integración nueva se añade como módulo en `tests/integracion/main.rs`, no como archivo suelto en `tests/`.
- La publicación a mano es Actions > «CI y publicación» > *Run workflow* sobre `main`.
