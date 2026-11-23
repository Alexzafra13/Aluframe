# ADR-0021: migraciones numeradas desde la versión 0.11.0

**Estado:** aceptado (2026-11-22). Sustituye al esquema único de ADR-0011.

## Contexto
El propietario va a empezar a meter datos (aunque sea de prueba, a mano) y quiere que **sobrevivan a las actualizaciones** (decisión para H7). Con el esquema único de ADR-0011, cualquier cambio de la base obligaba a borrar los datos.

## Decisión
- El esquema de las versiones 0.8.0 a 0.10.0 se congela como **migración 0001** (`crates/servidor/src/bd/migraciones/0001_inicial.sql`). Cada cambio posterior es un archivo nuevo `NNNN_descripcion.sql` añadido a la lista `MIGRACIONES` de `bd/esquema.rs`. **Una migración publicada nunca se edita**; una prueba compara la huella de la 0001.
- La versión de la base es `PRAGMA user_version`. Cada migración se aplica en **una transacción** junto con el cambio de versión: si falla, la base queda como estaba.
- Al arrancar:
  - base vacía: se aplican todas;
  - base de 0.8.0–0.10.0 (sin número y con la huella de la 0001): se **adopta** como versión 1 sin tocar los datos. Valen las dos huellas con que se guardó: la del archivo con LF (Linux) y la del archivo con CRLF (las versiones de Windows, compiladas con el código sacado por Git en Windows). Desde 0.12.1, `.gitattributes` fija LF para los `.sql` y la prueba calcula la huella sin `\r`;
  - base con migraciones pendientes y datos: antes se guarda `copias/antes-de-actualizar-….alumframe` con `VACUUM INTO`;
  - base **más nueva** que el programa: no arranca y pide instalar la versión reciente;
  - base de un desarrollo anterior a 0.8.0: no arranca (no hay instalaciones así con datos).
- Desde la migración 2, la huella de la tabla `esquema` cambia: las versiones 0.10.0 y anteriores, que solo miran la huella, se niegan a abrir una base que ya no entienden.
- **Copias e importación:** se acepta una copia de la misma versión o anterior; al restaurarla se le aplican las migraciones que falten. Una copia más nueva se rechaza con un mensaje claro.
- No hace falta tabla `version_esquema` (BACKEND 8): `user_version` va en la cabecera del archivo, entra en la transacción y no cambia el esquema.

## Requisitos del ADR-0002
- Sin dependencias nuevas.
- **Pruebas** (`bd/esquema_pruebas.rs`): base nueva, preparar dos veces, adopción de una base de 0.10 con datos, rechazo de bases desconocidas o más nuevas, una migración nueva conserva los datos (y `foreign_key_check` limpio), una migración que falla no deja nada a medias, la 0001 no se ha editado. `tests/datos.rs`: copia más nueva rechazada. Comprobado a mano: una base creada por el binario 0.10 arranca con la 0.11 con sus datos.

## Cómo se añade una migración
1. Crea `migraciones/0002_lo_que_cambia.sql` (solo `ALTER TABLE … ADD COLUMN`, tablas nuevas o pasos que conserven los datos; si hay que transformar datos, en el mismo archivo).
2. Añádela a `MIGRACIONES` con su número.
3. Prueba que una base con datos de la versión anterior se actualiza (como `una_migracion_nueva_conserva_los_datos`).
4. Si sería destructiva, **para y pregunta al propietario** (`CLAUDE.md`).

## Consecuencias
- Ya no se borran los datos al actualizar.
- Volver a una versión anterior tras una migración exige restaurar la copia `antes-de-actualizar-…`.
