# ADR-0009: las series viven en la base de datos como el YAML original de cada versión

**Estado:** aceptado (2026-11-21, H1; decisión delegada por el propietario).

## Contexto
SPEC.md y DATOS.md dejaban abierto dónde viven las series en ejecución. DATOS.md proponía tablas detalladas (tipología, pieza, vidrio, accesorio, opción).

## Decisión
- Cada importación crea una fila en `serie_version` con el **texto YAML original**, su **SHA-256**, el estado y el número de avisos. Un trigger impide modificarla o borrarla.
- `serie_referencia` guarda los códigos y referencias de cada versión, porque H2 los enlazará con la tarifa.
- Para calcular, se lee el YAML guardado y se vuelve a validar con el mismo código que en la importación (milisegundos).
- Importar el mismo archivo dos veces no crea versión nueva; un archivo distinto crea la siguiente.

## Por qué
- Una sola fuente de verdad: lo que se calcula es exactamente lo que se comprobó al importar, y se puede exportar igual.
- Las tablas detalladas duplicarían el archivo, exigirían migraciones cada vez que el formato crece y no aportan consultas que hagan falta.
- Si el programa cambia y una versión guardada deja de leerse, falla con un error interno visible, no en silencio.

## Revisar si
Hace falta buscar entre series por pieza o accesorio (por ejemplo, «qué series usan esta referencia»): bastará con añadir una tabla derivada.
