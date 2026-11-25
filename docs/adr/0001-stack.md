# ADR-0001: elección de stack

**Estado:** aceptado por el usuario (versión 1), con punto de revisión al terminar el hito H1.

## Contexto
ERP local para un taller pequeño. Un solo desarrollador, que trabaja con Claude Code. El programa se construye una vez y después apenas se toca; el poco mantenimiento lo hará Claude. El mayor riesgo es el dominio (fórmulas y precios correctos), no el rendimiento: la carga es de unos pocos usuarios. El usuario tiene interés en Rust.

## Opciones
| Opción | A favor | En contra |
|---|---|---|
| **A. TypeScript en todo** (Node, SQLite, React) | Un solo lenguaje, desarrollo más rápido, el desarrollador lo domina, ecosistema enorme | Necesita Node o Docker; muchas dependencias que actualizar; los tipos desaparecen al ejecutar |
| **B. Dominio en Rust compilado a WASM + resto en TypeScript** | Motor robusto compartido con la web | Dos cadenas de herramientas y fronteras de datos que depurar |
| **C. Rust completo** (Axum, SQLite, interfaz generada en el servidor) | Ejecutable único y pequeño; el compilador protege los cambios futuros; casi sin dependencias de ejecución | Desarrollo más lento; menos ejemplos de ERP |
| **D. Go + SQLite + React embebida** | Ejecutable único, sencillo, conocido | Menos garantías en el motor de cálculo; PDF menos pulido |

## Decisión
**C. Rust completo.** Motivos:
1. El mantenimiento posterior será mínimo y lo hará Claude: lo que más importa es que el programa no se deteriore y que el compilador impida regresiones.
2. Instalación sencilla: un ejecutable y un archivo de datos, sin Node ni Docker.
3. Corrección del motor de cálculo (enteros estrictos, casos exhaustivos).
4. Interés del desarrollador, que sostiene el proyecto a largo plazo.

## Consecuencias y mitigaciones
- La construcción es más lenta. Mitigación: hitos pequeños, pruebas con las ventanas reales y el diseño ya cerrado.
- Dependencia de Claude para modificar el código. Mitigación: pruebas completas, ADR, `docs/MAINTENANCE.md` y `docs/CONTEXTO.md`.
- Una sola base de código en Rust: la interfaz se genera en el servidor (ADR-0002), sin cadena de herramientas de Node.
- Sin comprobación de SQL al compilar. Mitigación: tests de integración contra SQLite real (ADR-0002).

## Punto de revisión (tras H1)
El hito H1 (motor de cálculo y optimizador, probado contra `fixtures/real/`) es la prueba de viabilidad. Si el desarrollo se atasca o resulta inmanejable, se reabre este ADR y se valora la opción A o D; el motor ya probado y los casos en JSON sirven como contrato independiente del lenguaje.

**Resultado (2026-11-21, al terminar H1):** el motor salió sin atascos; se mantiene Rust.

## Pendiente de ADR posteriores
- ADR-0002: criterios de selección y tecnologías concretas (ya redactado).
- ADR-0003: generación automática de PDF en el servidor (spike con Typst u otras).

## Comprobación 2026-11-20: ejecutable local o servidor con navegador
Se comprobó si el stack soporta los dos usos: un ejecutable que funcione solo en un PC y un servidor al que se accede por navegador. Es el mismo programa con distinta configuración (dirección de escucha, servicio y apertura del navegador), sin código duplicado: ver `docs/BACKEND.md`, apartado 14. Este requisito no distingue entre Rust y Go, que también sería válido, por lo que **no cambia la decisión**. El punto débil sigue siendo el de las consecuencias: el propietario conoce Go, no Rust, y depende de Claude para modificar el código. Se mantiene el punto de revisión tras H1.
