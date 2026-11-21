# Prompts para Claude Code

Abre Claude Code en la raíz del repositorio. No hace falta ejecutar `/init`. Trabaja con una sesión por hito y usa `/clear` entre hitos.

## Primera sesión (H0), en modo plan
Activa el modo plan (Mayús+Tab o `/plan`) y pega:
```
Lee CLAUDE.md, SPEC.md, docs/CONTEXTO.md, docs/BACKEND.md, docs/DATOS.md, docs/PANTALLAS.md y docs/adr/ (docs/FORMATO_SERIES.md a partir de H1). Después, sin escribir código, propón el plan detallado del hito H0 según el apartado H0 de SPEC.md: orden de tareas, archivos que crearás, dependencias que añadirás (con su justificación según CLAUDE.md) y cómo comprobarás cada criterio de aceptación. Señala contradicciones o huecos que encuentres. Espera mi aprobación.
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
