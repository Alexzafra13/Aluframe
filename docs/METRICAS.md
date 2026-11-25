# Métricas del proyecto

Para ver versión a versión si trabajar en el programa se vuelve más lento o más pesado (ADR-0035). Se añade una fila al publicar cada versión con `just metricas` (o `scripts/metricas.sh`; con `--limpia` mide también desde cero, borrando `target`). Si algo empeora de forma clara, se comenta al propietario y se busca la causa.

Las cifras son de la máquina de la nube de Claude (4 núcleos), así que sirven para comparar entre filas, no como tiempo absoluto. «Tras tocar un archivo» es recompilar las pruebas tras cambiar la fecha de un archivo del servidor; un cambio real de una línea cuesta algo más (ver abajo).

## Trabajo en local

| Fecha | Versión | Compilación limpia | Tras tocar un archivo | Pruebas (n.º y tiempo) | Líneas de Rust | target | Ejecutable |
|---|---|---|---|---|---|---|---|
| 2026-11-25 | 0.24.0, antes de ADR-0035 | 95 s | — | 519 en 28 s | 47 900 | 9,5 GB | 10,0 MB |
| 2026-11-25 | 0.24.0, con ADR-0035 | 61 s | 5 s | 522 en 15 s | 47 945 | 2,1 GB | — |

### Mediciones sueltas del 2026-11-25 (antes → después de ADR-0035)
- Cambiar una línea del servidor y recompilar las pruebas: **24–28 s → 5,5 s**; una del dominio: **28 s → 6,3 s**.
- Pruebas de integración: **17 s → 8,6–11 s** (las de escritorio, de 10,2 s a 2,6 s cada una; la más larga es ahora la del latido, 8,5 s, que espera de verdad 6 latidos).
- Recompilar el ejecutable tras cambiar una línea: `release` **122 s**; perfil `e2e` **4 s** (12,8 MB frente a 10,5 MB, solo para la E2E local).
- E2E completa (36 pruebas, una tras otra): 2,6 min.
- Leer una serie guardada (Aluprom 21, YAML y prueba de sus combinaciones): **4,8 ms** en `release`; ahora solo la primera vez por proceso.

## GitHub Actions

| Fecha | Versión | Desde el push a `main` hasta publicada | Notas |
|---|---|---|---|
| 2026-11-25 | 0.24.0 | 12 min | Antes de ADR-0035: Linux 3,5 min y después Windows sin caché 8,5 min; CI repetía las pruebas a la vez (7 min). |
