# ADR-0011: esquema único de base de datos hasta el primer despliegue real

**Estado:** sustituido por ADR-0021 (2026-11-22). Aceptado el 2026-11-21 por decisión del propietario.

## Contexto
H0 y H1 crearon tres migraciones (0001 a 0003) para un programa que todavía no está instalado en el taller. Mantener migraciones antes de tener datos reales solo añade archivos y pruebas de actualización sin valor.

## Decisión
- Un solo archivo, `crates/servidor/src/bd/esquema.sql`, que se **amplía directamente** en cada hito.
- La tabla `esquema` guarda la huella SHA-256 del archivo. Al arrancar:
  - base vacía: se crea el esquema;
  - misma huella: se usa;
  - huella distinta o tablas sin huella: **no arranca** y pide borrar la carpeta de datos de desarrollo.
- `alumframe preparar-base` (`just db-migrate`) crea o comprueba la base y termina.

## Consecuencias
- Cambiar el esquema obliga a borrar `datos-dev/` y las bases de prueba; los datos de desarrollo no son valiosos.
- Las versiones 0.1.0 y 0.2.0 publicadas usan el sistema anterior: sus bases no se abren con versiones nuevas (mensaje claro). No hay instalaciones reales.

## Cuándo se revisa
**Antes del primer despliegue con datos reales** (como tarde en H7): el esquema de ese momento se congela como migración 0001 y desde entonces cada cambio es una migración numerada que nunca se edita, con copia previa y pruebas desde bases antiguas (BACKEND.md, apartado 8).
