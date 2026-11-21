# ADR-0004: plantillas con Askama

**Estado:** aceptado (2026-11-21, H0).

## Contexto
ADR-0002 deja elegir entre Askama y Maud para generar el HTML en el servidor con comprobación al compilar.

## Decisión
**Askama 0.16.**
1. **Requisito:** HTML comprobado al compilar (variables y tipos), sin JavaScript en el servidor.
2. **Estabilidad:** API pequeña (`#[derive(Template)]` y `render()`); los cambios entre versiones menores afectan sobre todo a la integración con frameworks, que no usamos (la respuesta se arma con `Html(plantilla.render()?)`).
3. **Mantenimiento:** 4 crates propios (askama, askama_derive, askama_macros, askama_parser); se compila una vez.
4. **Licencia:** MIT o Apache-2.0.
5. **Pruebas y sustitución:** las plantillas son archivos `.html` en `crates/servidor/plantillas/`; si Askama desapareciera, se portan a otro motor con sintaxis Jinja (MiniJinja) cambiando solo `vistas/`.
6. **Alternativa descartada:** Maud. Escribe el HTML dentro de Rust con macros: menos legible para el propietario, que es desarrollador web, y más difícil de comparar con el diseño.
