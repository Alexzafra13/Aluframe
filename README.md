# AlumFrame

Programa de gestión para un taller de carpintería de aluminio y cerrajería: presupuestos con dibujo, orden de taller con medidas de corte, trabajos asignados a operarios, stock y facturación.

Estado: **fase de especificación completa (versión 1, borrador)**. Aún no hay código: se construye con Claude Code, hito a hito (H0 a H7).

## Cómo empezar con Claude Code
1. **Instala en tu PC:** Git, Rust (con `rustup`), `just`, Node (solo para las pruebas E2E) y Claude Code, con una suscripción que lo incluya.
2. **Crea el repositorio** (privado: contiene fórmulas y datos del negocio): descomprime este paquete en una carpeta, entra en ella y ejecuta `git init` y un primer `git commit`.
3. **Abre Claude Code en esa carpeta.** No hace falta `/init`. Activa el modo plan.
4. **Pega el primer mensaje** de `docs/PROMPT_INICIAL.md` (sección "Primera sesión (H0)"). Claude propondrá el plan de H0 y esperará tu aprobación.
5. Trabaja **una sesión por hito** y limpia el contexto (`/clear`) entre hitos. Los demás mensajes (empezar, cerrar y retomar un hito) están en el mismo archivo.

## Qué leer y en qué orden
1. `docs/CONTEXTO.md`: decisiones tomadas, datos reales y lo que falta.
2. `SPEC.md`: alcance, dominio e hitos con criterios de aceptación.
3. `CLAUDE.md`: reglas para Claude Code (stack, arquitectura, tests).
4. `docs/BACKEND.md`, `docs/DATOS.md`, `docs/FORMATO_SERIES.md` y `docs/PANTALLAS.md`: algoritmos y reglas, modelo de datos, formato de las series e interfaz.
5. `docs/adr/`: decisiones de arquitectura. `docs/OPEN_QUESTIONS.md`: lo pendiente. `docs/CAMBIOS.md`: registro de cambios.

## Contenido
- `data/series/`: series de perfiles como datos (en borrador; lo no confirmado está como `CONFIRMAR`).
- `fixtures/real/`: ventanas con el resultado esperado (`validado: false` hasta contrastarlas con hojas de corte reales).
- `fixtures/demo/`: datos **ficticios** para pruebas y demostraciones.
- `design/`: capturas del diseño aprobado y, en `design/prototipos/`, un prototipo de dibujo solo de referencia.

## Datos que aún tienes que aportar
Los descuentos del cristal de cada serie, la fórmula de las pletinas de falleba, los tipos de escuadra y la longitud de barra de la V-94, los límites de medida de cada tipología, una tarifa real de proveedor, 3 a 5 ventanas reales con su hoja de corte (plantilla en `docs/PLANTILLA_VENTANA_REAL.md`) y los datos fiscales y el logotipo del taller. Se pueden añadir en cualquier momento editando los archivos de `data/series/`.
