# ADR-0010: lectura de YAML con yaml-rust2 y un árbol neutro en el dominio

**Estado:** aceptado (2026-11-21, H1; decisión delegada por el propietario).

1. **Requisito:** leer los archivos de series (YAML) sin perder precisión en decimales como `4.85`, con errores en español que digan dónde está el fallo.
2. **Elección:** `yaml-rust2` 0.13.0. Devuelve los números reales como texto, así que el dominio los convierte a punto fijo sin pasar por `f64`. Una dependencia transitiva propia (`arraydeque`); `hashlink` ya estaba. Licencia MIT o Apache-2.0.
3. **Aislamiento:** solo `crates/servidor/src/series/yaml.rs` la usa, para convertir el YAML en `dominio::series::Valor` (texto, número como texto, lista, mapa). Toda la validación vive en el dominio y se prueba sin YAML.
4. **Sustitución:** cambiar de biblioteca, o pasar a TOML o JSON, solo toca ese archivo.
5. **Descartadas:** `serde_yaml` (abandonada en 2024), `serde_yml` (calidad dudosa), `serde_norway` (bifurcación sin actividad reciente) y derivar el modelo con serde (los números pasarían por `f64` y los errores serían genéricos, en inglés).
