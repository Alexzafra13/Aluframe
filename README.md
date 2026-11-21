# AlumFrame

Programa de gestión para un taller de carpintería de aluminio y cerrajería: presupuestos con dibujo, orden de taller con medidas de corte, trabajos asignados a operarios, stock y facturación.

Estado: **H0 y H1 terminados** (v0.2.0); siguiente, H2 (clientes, catálogo y tarifas). Se construye con Claude Code, hito a hito (H0 a H7). Para retomar el trabajo: `docs/PROGRESS.md`, apartado «Cómo retomar».

## Cómo funciona
- **Servidor del taller (Linux):** un único ejecutable `alumframe` y su base de datos SQLite `alumframe.db`, como servicio systemd en el **puerto 9090**.
- **PC de la oficina (Windows) y tablet:** entran con el navegador en `http://IP-DEL-SERVIDOR:9090`. En Windows se crea un acceso directo que lo abre como una aplicación.
- **`alumframe.exe` en modo local:** para probarlo en un solo PC o como plan de emergencia.

## Desarrollo
```
just instalar-hooks   # una vez por clon
just dev              # arranca en http://localhost:9090 con datos en datos-dev/
just check            # formato, clippy, pruebas y tamaño de archivos
just e2e              # pruebas con navegador contra el ejecutable
```
La primera vez aparece el asistente para crear la cuenta del propietario. Instalación y mantenimiento: `docs/MAINTENANCE.md`.

## Cómo seguir con Claude Code
Trabaja **una sesión por hito**. Los mensajes para empezar, cerrar y retomar un hito están en `docs/PROMPT_INICIAL.md`.

archivo.

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
