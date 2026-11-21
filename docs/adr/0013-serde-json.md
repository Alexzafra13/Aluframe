# ADR-0013: `serde_json` para los resultados guardados y las copias congeladas

**Estado:** aceptado (2026-11-21, H3).

1. **Requisito:** DATOS 7.1 guarda en columnas `*_json` el resultado del cálculo de cada línea, sus opciones y la copia congelada de un presupuesto aceptado (BACKEND, apartado 4: JSON canónico con su SHA-256). Escribir y leer JSON a mano para estructuras anidadas es fácil de romper.
2. **Elección:** `serde_json` 1.0.154 (la de `Cargo.lock`), que ya estaba en `Cargo.lock` como dependencia de pruebas; pasa a dependencia normal de `crates/servidor`. Sin dependencias nuevas en el árbol (`itoa`, `ryu`, `memchr` ya llegaban por serde). MIT o Apache-2.0.
3. **Estabilidad:** versión 1 desde 2017 con compromiso de compatibilidad; es la referencia del ecosistema serde.
4. **Canónico:** las estructuras se serializan con campos en orden fijo y los mapas son `BTreeMap`, así el mismo contenido da siempre el mismo texto y la misma huella. Sin `f64`: dinero y medidas van como enteros.
5. **Aislamiento y sustitución:** solo el servidor lo usa, en `bd/` y `aplicacion/`; el dominio sigue sin depender de él (las estructuras derivan `Serialize`/`Deserialize` con `serde`, que ya usaba).
6. **Descartado:** escribir JSON a mano (frágil) y guardar el resultado en tablas relacionales (muchas tablas para un dato que solo se lee entero).
