# ADR-0002: criterios de selección y tecnologías concretas

**Estado:** aceptado por el usuario el 2026-11-19 (versión 1; se mantiene en versión 1 hasta la versión definitiva).

## Principio
Se elige por **requisito técnico**, no por popularidad ni porque un modelo de IA conozca mejor una herramienta. La familiaridad del desarrollador cuenta solo en la medida en que reduce el riesgo de mantenimiento.

## Requisitos que deciden
1. Construir una vez y casi no mantener (lo hará Claude): pocas dependencias y API estables.
2. Cálculo correcto de dinero y medidas.
3. Un servidor en red local, pocos usuarios, sin internet garantizado.
4. Interfaz con formularios, listas, vista previa en vivo, dibujo SVG y hojas imprimibles.
5. Despliegue sencillo, probablemente en Windows.
6. Construcción autónoma con Claude Code: retroalimentación fuerte (tipos y tests).

## Decisiones
| Pieza | Elección | Por qué | Descartado | Revisar si |
|---|---|---|---|---|
| Lenguaje | Rust | Newtypes y enums exhaustivos para dinero y medidas; ejecutable único; pocas dependencias de ejecución | Go (casi igual de bueno en mantenimiento, con menos garantías de dominio), TypeScript | H1 se atasca |
| Base de datos | SQLite (`STRICT`, claves foráneas, WAL) | Embebida, un archivo, copia consistente, carga mínima | PostgreSQL (servicio extra) | Se ofrece como servicio en la nube a muchos talleres |
| Acceso a datos | `rusqlite` con SQLite incluido | Capa fina con control de pragmas y copias; las comprobaciones de SQLx en compilación son más limitadas con el tipado dinámico de SQLite y añaden fricción | SQLx, Diesel, SeaORM | Se cambia a PostgreSQL |
| Servidor web | Axum | API estable sobre Tokio y Hyper; el rendimiento no es el motivo | Actix, Rocket | Aparece una alternativa claramente más simple y estable |
| Interfaz | HTML generado en el servidor (Askama o Maud) + HTMX incluido en el repo | Sin Node ni empaquetador, sin generación de tipos, el SVG se genera una vez y sirve en pantalla e impresión, casi sin dependencias | SPA con React, Leptos o Dioxus | Hacen falta editores muy ricos (tablas tipo hoja de cálculo, arrastrar y soltar complejo) |
| CSS | CSS propio con variables (tokens) | Estable a largo plazo; el diseño es pequeño; sin paso de compilación | Tailwind (aporta más en componentes grandes y cambia entre versiones mayores) | El diseño crece mucho |
| JavaScript | Mínimo, sin empaquetador, archivos en el repo | Cero dependencias de npm en producción | Frameworks de JavaScript | Se necesita interactividad que HTMX no cubre |
| Impresión | Hojas HTML con CSS de impresión A4 en el MVP | La más simple y robusta para la hoja del operario | Generar PDF desde el primer día | Hace falta enviar o archivar PDF automáticamente (ADR-0003) |
| Identificadores | Enteros autoincrementales | Una sola base de datos, sin sincronización | UUIDv7 | Hay sincronización entre instalaciones |
| Fórmulas | Evaluador propio de gramática mínima | Seguro y verificable | Librerías de expresiones generales | La gramática crece más de lo previsto |
| Arquitectura | Dominio puro y módulos finos; sin traits de repositorio salvo necesidad | Menos ceremonia; los tests usan SQLite real en memoria | Arquitectura hexagonal completa | Se separan varios almacenes de datos |
| E2E | Playwright solo en desarrollo | No se distribuye y es la mejor herramienta para el caso | WebDriver en Rust (menos madura) | Aparece una alternativa madura en Rust |
| Autenticación | Propia y simple con `argon2` | Alcance pequeño | Frameworks de autenticación | Se añade inicio de sesión externo |

## Notas honestas
- Con una interfaz generada en el servidor, **Go sería casi igual de adecuado** que Rust: biblioteca estándar completa, compatibilidad garantizada y compilación rápida. La ventaja real de Rust aquí es la corrección del motor de cálculo y el interés del desarrollador.
- Tailwind, React, SQLx y los UUID eran elecciones por defecto y no han resistido el análisis por requisitos.
- Typst como librería es atractivo para PDF, pero su API puede cambiar entre versiones (a comprobar en el spike de ADR-0003).

## Regla para el futuro
Toda dependencia nueva exige un ADR corto con: requisito que resuelve, alternativa más simple, estabilidad, dependencias transitivas, licencia, cómo se prueba y por qué se descartan las alternativas.
