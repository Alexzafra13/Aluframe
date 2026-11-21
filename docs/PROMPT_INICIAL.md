# Prompts para Claude Code

Abre Claude Code en la raíz del repositorio. No hace falta ejecutar `/init`. Trabaja con una sesión por hito y usa `/clear` entre hitos.

## Siguiente sesión (H4), en modo plan
```
Lee CLAUDE.md y docs/PROGRESS.md (apartado «Cómo retomar»), y después docs/CONTEXTO.md, docs/OPEN_QUESTIONS.md, el apartado H4 de SPEC.md, docs/DATOS.md (3.6, 4 y 7.7), docs/PANTALLAS.md (8 y 13), docs/design.md y design/03-hoja-taller.png. Trabaja sobre la rama que indica PROGRESS.md (H2 y H3 aún no están en main). Comprueba que todo está en verde. Después, sin escribir código, propón el plan de H4: tablas que añadirás a esquema.sql, casos de uso, pantallas (con el estilo iOS ya acordado), pruebas y cómo comprobarás los criterios de aceptación. Señala huecos o contradicciones y espera mi aprobación.
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
