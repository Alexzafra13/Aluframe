# AlumFrame: instrucciones para Claude Code

ERP local para un taller de carpintería de aluminio y cerrajería (España).
Para retomar, lee primero `docs/PROGRESS.md` (estado, siguiente tarea, cómo trabajar y comprobar) y `docs/OPEN_QUESTIONS.md`. El resto, según la tarea: `SPEC.md`, `docs/CONTEXTO.md`, `docs/BACKEND.md`, `docs/DATOS.md`, `docs/FORMATO_SERIES.md`, `docs/PANTALLAS.md` y `docs/adr/`.

## Condiciones para empezar
- H0–H7 están hechos: ADR-0001 y ADR-0002 aceptados y diseño de `design/` aprobado (consta en `docs/CONTEXTO.md`). Ahora se pule la versión mínima con lo que pide el propietario.
- Para una tarea nueva grande, trabaja en modo plan: propón el plan y espera la aprobación antes de escribir código. Los arreglos y retoques que pida el propietario se hacen directamente.
- Estas instrucciones no se han probado con una construcción real. Si algo es inviable o contradictorio, dilo, propón una alternativa y anótalo en un ADR; no lo resuelvas en silencio.
- No ejecutes `/init`: este `CLAUDE.md` ya existe.
- Toda la documentación está en **versión 1** hasta que el propietario declare la definitiva. No cambies números de versión; anota cada cambio relevante en `docs/CAMBIOS.md`.

## Objetivo de diseño
Construirlo una vez, bien, y que apenas necesite mantenimiento. Prioriza robustez, pruebas y documentación sobre rapidez o ingenio. Otra sesión de Claude debe poder retomar el proyecto sin contexto previo.

## Idioma
- Interfaz, documentos, documentación, comentarios y mensajes de commit: **español (es-ES)**.
- **El código también va en español, sin tildes ni eñes** (ASCII): funciones, tipos, variables, módulos, archivos, tablas y columnas. Se usa el vocabulario del taller tal cual (presupuesto, albaran, retal) para que el código se lea como el negocio. Ejemplos: `calcular_cuota_iva`, `optimizar_cortes`, `aceptar_presupuesto`, `PlanDeCorte`, `EstadoPresupuesto::Aceptado`.
- Funciones con verbo en infinitivo y un nombre que dice qué hacen y qué devuelven. Las preguntas de sí o no empiezan por `es_`, `tiene_` o `puede_` (`es_incompleta`). Las pruebas se nombran por el comportamiento (`rechaza_division_por_cero`).
- Los términos técnicos del lenguaje y de las bibliotecas (`Result`, `Mutex`, `Router`) se dejan como están. En los commits, el prefijo de Conventional Commits (`feat`, `fix`, `docs`) va en inglés y el resto en español.
- Vocabulario de nombres en `docs/GLOSSARY.md`.

## Estructura del repositorio
```
crates/dominio   lógica pura: series, optimizador de barras, dibujo SVG, dinero, longitudes (sin entrada/salida)
crates/servidor  servidor, SQLite, plantillas HTML, recursos estáticos (CSS, JS, fuentes) y copias de seguridad
e2e/            pruebas de extremo a extremo con Playwright (solo desarrollo)
data/series/    series de perfiles como datos (YAML)
fixtures/real/  ventanas reales con el resultado esperado (fuente de verdad)
design/         capturas del diseño aprobado
docs/           PROGRESS, GLOSSARY, OPEN_QUESTIONS, MAINTENANCE, CONTEXTO, design, adr/
```

## Bucle de trabajo autónomo
1. Lee el hito actual en `docs/PROGRESS.md` y sus criterios de aceptación en `SPEC.md`.
2. Planifica en pocas líneas. Para la lógica de dominio, escribe los tests primero.
3. Implementa en pasos pequeños. Tras cada paso ejecuta `just check`.
4. Haz un commit por unidad lógica (Conventional Commits). Nunca dejes la rama principal en rojo.
5. Al terminar el hito: comprueba los criterios de aceptación, actualiza `docs/PROGRESS.md` (hecho, decisiones, pendiente) y **detente**.

**Para y pregunta** si ocurre cualquiera de estas cosas:
- Faltan datos reales del taller (fórmulas, precios, referencias). **Nunca inventes datos reales.** Trabaja solo con lo que hay en `data/series/` y `fixtures/real/`; lo marcado `CONFIRMAR` o `validado: false` no puede producir un precio final. Excepción: `fixtures/demo/` contiene datos **ficticios** (`estado: demo`) para pruebas E2E y demostraciones; nunca se cargan en el uso real ni se mezclan con datos reales.
- Una migración sería destructiva.
- Necesitas una dependencia que no encaje con el stack.
- Hay dos interpretaciones razonables de un requisito que afecten a precios, medidas o facturas.

Si te bloqueas, anota la pregunta en `docs/OPEN_QUESTIONS.md` y continúa con lo que no dependa de ella.

## Criterios para elegir tecnología
La popularidad **no** es un argumento, ni lo es que un modelo de IA conozca mejor una herramienta. Antes de añadir una dependencia o cambiar una pieza del stack, escribe un ADR corto que responda:
1. ¿Qué requisito concreto resuelve? ¿Basta la biblioteca estándar o unas pocas líneas propias?
2. Estabilidad: ¿hay compromiso de compatibilidad o la API cambia a menudo?
3. Coste de mantenimiento: dependencias transitivas, actividad real y riesgo de abandono.
4. Licencia compatible, tamaño y tiempo de compilación.
5. ¿Cómo se prueba y cómo se sustituye si desaparece?
6. Alternativas consideradas y por qué se descartan.
Prefiere lo simple y estable. Una herramienta compleja solo se justifica si resuelve algo que la simple no puede.

## Stack (ver ADR-0001 y ADR-0002; cambiarlo exige un ADR nuevo)
- **Rust estable**, versión fijada en `rust-toolchain.toml`. Workspace de Cargo.
- Servidor: Tokio + Axum. **rusqlite** con SQLite incluido (tablas `STRICT`, claves foráneas activas, modo WAL, `busy_timeout`), usado desde tareas bloqueantes con una conexión de escritura y un grupo pequeño de lectura. Base de datos con migraciones numeradas (ADR-0021). Además `serde`, `thiserror`, `tracing`, `argon2` y `time` (`time` no incluye zonas horarias: la conversión a Europe/Madrid se decide en un ADR durante H0).
- Interfaz: **HTML generado en el servidor** con plantillas comprobadas al compilar (Askama o Maud; decide en un ADR) y **HTMX** incluido en el repositorio, sin npm. JavaScript mínimo, sin empaquetador. **CSS propio con variables**, sin Tailwind ni frameworks CSS.
- Dibujo de la ventana: función de Rust que devuelve SVG, usada en pantalla y en la hoja impresa.
- Impresión: hojas HTML con hoja de estilos de impresión A4 (`@page`, saltos de página). PDF automático en el servidor (correo, archivo): spike y ADR-0003; aíslalo tras un puerto, porque su API puede cambiar.
- Calidad: `cargo fmt`, `clippy` con `-D warnings`, `cargo test` o `nextest`, `proptest`, `cargo deny`. Playwright solo en `e2e/`.
- Tareas: `just`. Despliegue: un único ejecutable `alumframe` con plantillas y recursos incrustados (`rust-embed`) más `alumframe.db`, instalado como servicio systemd en el servidor Linux o como aplicación de escritorio en Windows (ADR-0015); un servicio de Windows solo haría falta si el servidor fuera Windows (aplazado, ADR-0022). El mismo ejecutable funciona en **modo servidor** (red local) o **modo local** (solo ese PC): ver `docs/BACKEND.md`, apartado 14. Acceso remoto opcional con Tailscale.

## Arquitectura
- `crates/dominio` **no depende** de tokio, axum ni rusqlite; Cargo lo garantiza.
- En `crates/servidor`: `http` (rutas), `vistas` (plantillas), `aplicacion` (casos de uso) y `bd` (consultas SQL propias). Sin traits de repositorio salvo necesidad concreta; los tests usan SQLite real en memoria. Los puertos solo existen para lo que cambia desde fuera (PDF, reloj).
- **El motor de cálculo vive solo en Rust.** La página pide el cálculo al servidor con HTMX y un retardo corto, y recibe un fragmento HTML con el dibujo y las piezas.
- Sin lógica de negocio en rutas ni plantillas.

## Reglas de código
- `#![forbid(unsafe_code)]`. Prohibido `unwrap()`, `expect()` y `panic!` fuera de tests (lints de clippy).
- Errores de dominio tipados con `thiserror`; `anyhow` solo en `main`.
- **Dinero:** `Dinero(i64)` en céntimos. IVA y totales se calculan en un único sitio.
- **Longitudes:** `Longitud(i64)` en décimas de milímetro. Las series y la interfaz usan **cm** (un decimal de cm = 1 mm); por dentro todo va en décimas de mm. Las divisiones redondean de forma explícita y la regla es configurable por serie (por ahora, a 0,1 cm hacia abajo; provisional hasta validarlo con ventanas reales). Nunca `f64` para medidas ni dinero.
- Las fórmulas de las series se evalúan con un **evaluador propio de gramática mínima** (números, `ancho`, `alto`, parámetros, `+ - * /`, paréntesis, `minimo`, `maximo`, `redondear`, `techo`, `suelo` y `si(condición, entonces, si_no)` con las comparaciones `>`, `>=`, `<`, `<=`, `==` solo dentro de `si`, para cantidades o longitudes que dependen de la medida; primer caso real: las bisagras de la abatible).
- Valida cada entrada al deserializarla, con tipos fuertes.
- Base de datos: migraciones numeradas en `crates/servidor/src/bd/migraciones/` que **nunca se editan** una vez publicadas (ADR-0021, que sustituye al esquema único de ADR-0011). Cada cambio es una migración nueva que conserva los datos.
- Los **documentos emitidos son inmutables** (presupuesto aceptado, orden de taller, factura): copia los datos al emitir; no los releas del catálogo.
- Identificadores enteros autoincrementales. Fechas en UTC, mostradas en Europe/Madrid.
- Un concepto por archivo: hasta 200 líneas (máximo 300) y funciones de hasta 40 líneas. El detalle de módulos y algoritmos está en `docs/BACKEND.md`.
- Nombres que explican la intención. **Comentarios solo para el porqué** (decisiones no evidentes, invariantes, reglas del taller); nada que repita el código, ni bloques largos ni código comentado.

## Tests
- Dominio: unitarios y de propiedades (el optimizador nunca supera la longitud de barra ni pierde piezas).
- **Tests dorados:** las ventanas reales de `fixtures/real/` se ejecutan sobre el motor con cada cambio.
- Integración: casos de uso contra SQLite real en memoria.
- E2E con Playwright de los flujos principales de cada hito, contra el ejecutable compilado.
- Cobertura mínima del 90 % en `crates/dominio` (herramienta a decidir en H0, por ejemplo `cargo-llvm-cov`). En el resto importan los flujos críticos.
- **Nunca debilites un test ni cambies el resultado esperado de `fixtures/real/` para que pase.** Si un fixture parece incorrecto, para y pregunta.

## Interfaz y diseño
- **Diseño de referencia** en `design/` (presupuesto, trabajos en el taller y hoja de taller), aprobado (versión 1). Reprodúcelo con HTML y CSS propio; no lo reinterpretes. Documenta los tokens en `docs/design.md`.
- Tokens: papel `#ECE9E0`, superficie `#FBFAF6`, tinta `#15191D`, texto apagado `#5C636B`, línea `#D5D1C6`, azul acero `#1F4E79` (tinte `#E3EDF6`), aviso `#8A3207` sobre `#F8E6DA`, indicador de menú `#E8702A`. Solo azul y naranja como acentos.
- Tipografía: IBM Plex Sans, Sans Condensed (títulos y etiquetas) y Mono (medidas y códigos), **incluidas en el proyecto**: el servidor puede no tener internet.
- Las pantallas se actualizan con fragmentos HTML. El dibujo lleva las piezas numeradas como en la lista.
- Las hojas de taller y presupuesto se diseñan para **imprimir en A4** y deben parecerse a la captura aprobada.
- Objetivos táctiles de al menos 44 px (se usará en tablet). Formularios con etiquetas, foco visible y buen contraste. Formatos es-ES. Estados vacíos, de carga y de error en cada pantalla.

## Seguridad y dependencias
- Sesiones por cookie `httpOnly`, hash `argon2id` y roles (`propietario`, `operario`). El operario entra en la tablet con un **PIN** corto y el dueño con contraseña. Limita los intentos de acceso. El operario **no ve precios salvo que el dueño se lo permita, y nunca costes ni márgenes** (permisos por operario, ADR-0024).
- Protección contra CSRF en los formularios. El servidor escucha solo en la red local. Datos personales de clientes (RGPD): solo los necesarios, exportables y borrables.
- Sin secretos en el repositorio. Configuración junto al ejecutable o por variables de entorno.
- Versiones fijadas por `Cargo.lock`. No instales versiones publicadas hace menos de 14 días. `cargo deny` en CI.
- Copias de seguridad con la API de copia de SQLite o `VACUUM INTO` (nunca copies el archivo en caliente) y un test de restauración.

## Mantenimiento futuro
- Mantén `docs/MAINTENANCE.md` al día: cómo compilar, publicar una versión, instalar y actualizar en el taller, restaurar una copia, actualizar tarifas, añadir una serie y actualizar el toolchain y las dependencias.
- Cada decisión importante deja un ADR. Cada fallo corregido deja un test que lo reproduce.
- Las series, tarifas, IVA y plantillas de documentos son **datos**, no código: cambiarlos no debe exigir recompilar.

## Comandos
`just dev` · `just check` (fmt + clippy + tests) · `just test` · `just e2e` · `just build` · `just release` · `just db-migrate`

## Definición de hecho
Formato y clippy limpios, tests nuevos que fallaban antes y ahora pasan, sin duplicar lógica de dominio, y `docs/PROGRESS.md` actualizado si afecta a una decisión.
`just check` se ejecuta también en un hook previo al push (versionado en `.githooks/` y activado con `git config core.hooksPath .githooks`) y en CI.
