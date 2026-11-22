# ADR-0016: exportar e importar todos los datos

**Estado:** aceptado (2026-11-22, decisión del propietario).

## Contexto
Con la aplicación de escritorio (ADR-0015) se puede trabajar solo en un PC o contra el servidor del taller. El propietario quiere poder llevar los datos de uno a otro: empezar en local si todavía no tiene servidor y subirlos después, o sacar una copia del servidor.

## Decisión
- **Ajustes → Datos**, solo para el propietario (`Accion::GestionarDatos`).
- **Exportar:** descarga `alumframe-AAAA-MM-DD.alumframe`, una copia coherente hecha con `VACUUM INTO` (nunca se copia el archivo en caliente, `CLAUDE.md`). Queda en la auditoría.
- **Importar** en dos pasos:
  1. Se sube el archivo tal cual (binario, con el token CSRF en una cabecera; `recursos/js/subir-copia.js`). Se comprueba que es SQLite, que `PRAGMA integrity_check` da `ok` y que su huella de esquema es la de este programa. Si no, se borra y se explica el motivo.
  2. Una pantalla compara lo que hay y lo que trae la copia (clientes, presupuestos, usuarios, series y último cambio). Al confirmar se guarda antes una copia de lo actual en la carpeta de copias (`antes-de-importar-…`) y se sustituye todo con la API de copia de SQLite, sin parar el programa.
- **Importar sustituye, no mezcla.** Mezclar dos bases exige resolver conflictos (dos personas cambiando lo mismo, numeraciones que chocan) y es frágil; para el uso previsto (trabajar en un sitio cada vez) basta sustituir con una copia de seguridad previa.
- **Solo entre la misma versión del esquema.** Hasta congelar el esquema (ADR-0011) no hay migraciones; después, importar una copia antigua la migrará primero.
- La copia en tránsito vive en la carpeta de copias con un nombre aleatorio de 64 cifras hexadecimales; la URL solo acepta ese formato. Subir otra borra las pendientes.

## Requisitos del ADR-0002
- **Sin dependencias nuevas:** se activa la característica `backup` de `rusqlite`, que no añade crates (es la API de copia de SQLite).
- **Pruebas:**
  - `tests/datos.rs`: devuelve los datos al momento de exportar; rechaza un archivo que no es copia o de otro esquema; no acepta claves inventadas; el operario no puede; y restaura sobre una base en archivo, en WAL y con lectores abiertos;
  - `tests/datos_pantallas.rs`: las pantallas;
  - E2E `transferir-datos`: el flujo completo.

## Consecuencias
- Tras importar se entra con los usuarios de la copia.
- Es también la base de la restauración guiada de H7 (copias nocturnas).
