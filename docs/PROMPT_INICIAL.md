# Prompts para Claude Code

Abre Claude Code en la raíz del repositorio. No hace falta ejecutar `/init`. Trabaja con una sesión por hito y usa `/clear` entre hitos.

## Siguiente sesión: importar tarifas desde Excel y PDF, en modo plan
```
Lee CLAUDE.md y docs/PROGRESS.md («Siguiente tarea», «Reglas» y «Antes de publicar»), docs/OPEN_QUESTIONS.md, ADR-0012 y crates/servidor/src/aplicacion/importar_tarifa.rs. Comprueba que todo está en verde. Sin escribir código, propón el plan para importar tarifas desde Excel (.xlsx, .xls, .ods) y PDF (asistido) en la misma revisión de 3 pasos del CSV: dependencias con su ADR (las seis preguntas de CLAUDE.md), cómo se eligen hoja, fila de cabecera y columnas, qué pasa con un PDF escaneado, pruebas y E2E. Espera mi aprobación.
```

## Empezar un hito
```
Empieza el hito H<n> siguiendo CLAUDE.md. Primero lee docs/PROGRESS.md y el apartado del hito en SPEC.md, propón el plan y espera mi aprobación. Anota las preguntas que te bloqueen en docs/OPEN_QUESTIONS.md.
```

## Cerrar un hito
```
Comprueba uno a uno los criterios de aceptación del hito, ejecuta just check y los E2E, actualiza docs/PROGRESS.md (hecho, decisiones, pendiente) y docs/MAINTENANCE.md si procede, y resume qué falta. No empieces el siguiente hito.
```

## Retomar tras limpiar el contexto
```
Lee CLAUDE.md y docs/PROGRESS.md y dime en qué punto estamos y cuál es el siguiente paso, sin modificar nada.
```

## Permisos
`.claude/settings.json` pide confirmación antes de editar `fixtures/real/`, añadir dependencias (`cargo add`) o hacer `git push`, y bloquea los borrados masivos y los reinicios forzados. Compruébalo con `/permissions` en la primera sesión.
