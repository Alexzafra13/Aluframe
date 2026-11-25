# Progreso

Resumen para retomar sin contexto. El detalle de cada versión está en `docs/CAMBIOS.md`; los porqués, en `docs/adr/`.

## Estado (2026-11-25)
- **H0–H7 terminados.** Publicada la **v0.24.0** (nombre visible «Aluframe», ADR-0034). Los datos se conservan al actualizar (migraciones, ADR-0021) y hay copias automáticas (ADR-0022).
- **Fase: versión mínima.** El propietario usa la versión publicada (también la demostración, ADR-0028), apunta lo que ve mal y se arregla. **Los datos que faltan los pasará él: no se inventan, se esperan.**
- **Compilación y CI** (ADR-0035, sin versión nueva): todo va más rápido; las cifras están en `docs/METRICAS.md`. **La primera publicación con el workflow nuevo está por estrenar:** comprueba que salen el `.tar.gz` y el instalador.

## Siguiente tarea
1. **Lo que pida el propietario:** arreglos directamente, cada uno con su prueba, versión nueva y publicación.
2. **Datos que vaya pasando:** van en `data/series/*.yaml` o `data/acabados.yaml`, sin tocar código. Un fixture real solo cambia con datos suyos. Una serie con todos sus datos pasa de `borrador` a `validada`.
3. **Tareas grandes ya habladas** (antes de empezar cada una: plan y aprobación):
   - **Chapa** (ADR-0030): fase 2, en catálogo y stock con color, lacado por 1 o 2 caras y planchas (migración nueva); fase 3, aprovechar sus recortes (optimizador 2D).
   - **Mosquitera dentro de la ventana con persiana** (ADR-0031, ADR-0032).
   - **Fotos en los apuntes** (ADR-0025, «más adelante»).
   - **Importar tarifas desde Excel y PDF**, con la misma revisión en 3 pasos que el CSV (`aplicacion/importar_tarifa.rs`, ADR-0012). Necesita un ADR de dependencia (p. ej. `calamine`).
4. **Pequeñas, cuando haya hueco:**
   - pruebas E2E en paralelo, con un servidor por trabajador (hoy van en serie y dependen del orden);
   - `cargo-nextest`;
   - `cargo-mutants` sobre `dominio::precio` y `dominio::corte`.

## Cómo trabajar
- Lee `CLAUDE.md`, este archivo y `docs/OPEN_QUESTIONS.md`. Según la tarea:
  - interfaz: `docs/design.md` y `docs/PANTALLAS.md`;
  - datos: `docs/DATOS.md` y `docs/FORMATO_SERIES.md`;
  - servidor: `docs/BACKEND.md`;
  - contexto del taller: `docs/CONTEXTO.md`.
- Trabaja en una rama. Para publicar: sube `version` en `Cargo.toml` y llévalo a `main` (el propietario ya dio permiso). CI prueba en Linux y Windows y, si todo pasa, publica los mismos ejecutables que ha probado.
- Al propietario, en español llano y sin jerga. Tras publicar, dale el comando de actualización del servidor y dile si hay que borrar datos (desde la 0.11.0, nunca).

## Comprobar antes de llevar a `main`
En la nube no hay `just`. Ejecuta lo mismo que `just check` y la E2E:
```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings      # después de fmt
find crates -name '*.rs' -exec wc -l {} + | awk '$1>300 && $2!="total"'   # debe salir vacío
cargo test --workspace --locked
cargo build -p servidor --offline                  # si subiste la versión: actualiza Cargo.lock
cargo build --profile e2e --locked -p servidor     # no --release: ~4 s en vez de ~2 min
cd e2e && npm ci && ALUMFRAME_BIN=../target/e2e/alumframe npx playwright test --trace off
```
- **E2E:** mientras trabajas, solo el archivo que tocas (`npx playwright test pruebas/h3-presupuestos.spec.js`); la entera (~2,5 min) antes de `main`.
- **Capturas** de lo visual que cambies: PC a 1366 × 768 y 1366 × 620, e iPhone 13. Espera ~1 s a que terminen las animaciones.
- **Windows:** cuidado con los finales de línea (los `.sql` van con LF) y con las rutas.
- **Pruebas de integración:** una nueva es un módulo en `crates/servidor/tests/integracion/main.rs`, no un archivo suelto en `tests/`.
- **Al publicar,** `scripts/metricas.sh` y su fila en `docs/METRICAS.md`. Si algo empeora, díselo al propietario.
- **Disco lleno:** `cargo clean` o borra capturas del scratchpad.
- **Rendimiento:** hoy todo responde en menos de 10 ms (`docs/METRICAS.md`). Si el propietario nota lentitud, mira en su servidor `journalctl -u alumframe | grep "petición lenta"`.

## Reglas que conviene no olvidar
- **Migraciones** (ADR-0021): cada cambio de la base es un archivo nuevo en `crates/servidor/src/bd/migraciones/`; las publicadas no se editan. Una destructiva exige preguntar.
- **Datos que vienen con el programa** se corrigen en el repositorio, no en pantalla: series (`data/series/`, ADR-0014), colores (`data/acabados.yaml`, ADR-0019) y siluetas (`data/perfiles/`, ADR-0017).
- **Nada se inventa:** lo no confirmado va como `CONFIRMAR` o a `OPEN_QUESTIONS.md`. Sin margen ni modo de cobro elegidos, los presupuestos salen «Provisional».
- **Dependencia nueva:** ADR corto y versión con más de 14 días.
- **Tamaños:** archivos de 300 líneas como máximo y funciones de 40.
- **Interfaz:**
  - estilo iOS adaptado al móvil (`docs/design.md`);
  - la CSP no deja estilos en línea, así que los colores del dibujo van en atributos SVG;
  - estilos y scripts llevan `?v=` con la versión.
- **Datos de prueba:** ficticios, en `fixtures/demo/` (E2E con `ALUMFRAME_DATOS_DEMO=si`).

## Instalación del propietario
- Servidor Ubuntu en casa: `192.168.1.100`, servicio `alumframe`, datos en `/var/lib/alumframe`.
- Desde fuera, con Tailscale: `100.104.229.16:9090`.
- Para actualizar: `docs/MAINTENANCE.md`, apartado 4.
- **Pendiente de probar por él:**
  - el disco de copias montado en el servidor;
  - que el servicio arranca solo tras reiniciar;
  - el instalador de Windows.

## Datos que faltan del taller (detalle en `OPEN_QUESTIONS.md`)
- Medidas mínimas y máximas de cada serie (`limites`, hoy `CONFIRMAR`). Con ellas, el corte marcará como desecho lo que sobre por debajo de la pieza más corta (ADR-0026).
- Precio:
  - tarifa real de Aluval (y luego Sopena) y cómo la manda;
  - margen sobre coste o sobre venta;
  - mano de obra;
  - perfil cobrado por barra o por metros.
- Cristal: el de la 4 hojas de Aluprom 21, el alto real de la 2 hojas, Climalit y el vidrio habitual.
- Parteluces y chapa (medidas, cómo se cobra, planchas), persianas (cajones, medida de pedido, cobro, guía de 80 o 100 mm), mosquiteras (referencias, barras, máximos; su descuento con persiana) y Sopena V-94 (catálogo).
- Colores (RAL y «Embero»). Gestor: IVA, menciones legales y cómo refacturar tras una rectificativa.

## Vistos sin arreglar (poco probables con un solo dueño)
- «Terminar orden» recalcula el plan al confirmar. Si entre medias otra orden gastó un retal, los retales revisados se asignan por posición.
- La lista de Presupuestos muestra los 500 últimos sin paginar (los antiguos siguen en la ficha de cada cliente).

## ADR pendientes
- Servicio de Windows, solo si el servidor del taller fuera Windows (ADR-0022).
- PDF automático en el servidor (ADR-0003), si hace falta enviar documentos por correo.

## Hitos
H0 Cimientos · H1 Motor de dominio · H2 Catálogo, clientes e importación · H3 Presupuestos (2026-11-21) · H4 Trabajos y orden de taller · H5 Stock · H6 Facturación · H7 Copias y despliegue (2026-11-22). Todos hechos; el detalle está en `docs/CAMBIOS.md`.
