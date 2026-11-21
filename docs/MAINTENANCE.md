# Guía de mantenimiento (plantilla: completar en el hito H7)

Objetivo: que cualquier persona, o una sesión nueva de Claude sin contexto, pueda mantener AlumFrame.

## Contenido obligatorio
1. **Requisitos y compilación:** versión de Rust (`rust-toolchain.toml`), Node solo para los E2E con Playwright (`e2e/`); el ejecutable no lo necesita, `just`, comandos exactos.
2. **Pruebas:** cómo ejecutar tests, los tests dorados y los E2E.
3. **Publicar una versión:** `just release`, dónde queda el ejecutable y cómo se numera.
4. **Instalar y actualizar en el taller:** copiar el ejecutable, instalar el servicio, comprobar `/health`, qué hacer si falla la actualización (volver a la versión anterior).
5. **Copias de seguridad:** dónde se guardan, cómo comprobarlas y **cómo restaurar** paso a paso.
6. **Operaciones frecuentes sin tocar código:** actualizar tarifas (importación), añadir una serie (`data/series/`), cambiar tipos de IVA, cambiar contraseña o PIN, dar de alta usuarios.
7. **Actualizar el toolchain y las dependencias:** cada 6 meses o ante un aviso de seguridad (`cargo update`, `cargo deny`, tests completos, probar en copia de los datos).
8. **Si la base de datos falla:** diagnóstico, restauración y comprobación de integridad.
9. **Dónde están las decisiones:** `docs/adr/`, `docs/CONTEXTO.md`, `docs/PROGRESS.md`.
10. **Cómo retomar el proyecto con Claude:** indicaciones de arranque (leer `CLAUDE.md`, `SPEC.md`, `docs/adr/`, ejecutar `just check`).
