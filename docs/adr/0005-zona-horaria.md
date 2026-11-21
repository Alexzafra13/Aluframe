# ADR-0005: conversión a Europe/Madrid sin base de datos de zonas horarias

**Estado:** aceptado (2026-11-21, H0).

## Contexto
Las fechas se guardan en UTC y se muestran en Europe/Madrid. `time` no incluye zonas horarias.

## Decisión
**Regla propia en `crates/servidor/src/fecha.rs`** (unas 30 líneas con pruebas): UTC+1 en invierno y UTC+2 desde el último domingo de marzo a la 01:00 UTC hasta el último domingo de octubre a la 01:00 UTC (directiva europea 2000/84/CE).

## Por qué
- Basta la biblioteca estándar y `time`: no hace falta una base de datos de zonas horarias ni `chrono-tz` o `jiff`.
- Solo existe una zona (el taller está en Extremadura; Canarias no aplica).
- Las pruebas cubren invierno, verano y los dos instantes de cambio.

## Revisar si
La UE suprime el cambio de hora (bastará cambiar una función) o el programa se usa fuera de la península.
