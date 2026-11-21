# Contexto del proyecto (documento de traspaso)

Este archivo permite retomar el trabajo en una conversación nueva sin perder lo decidido.

## Estado
H0, H1 y H2 terminados: acceso, series importables, cálculo de ventanas con plan de corte, clientes y catálogo con tarifas versionadas. Siguiente: H3 (presupuestos). El punto de partida de una sesión nueva está en `docs/PROGRESS.md`, apartado «Cómo retomar».

## Qué es
AlumFrame: programa de gestión para el taller de cerrajería y ventanas de aluminio del padre de Alejandro (web developer en Zafra, Extremadura). Presupuestar ventanas con precio y dibujo, orden de taller con medidas de corte y cristal, stock con avisos y facturación. Se construirá con Claude Code a partir de `CLAUDE.md` y `SPEC.md`.

## Decisiones tomadas
- **Stack:** Rust completo (Axum, rusqlite/SQLite, interfaz HTML generada en el servidor con HTMX, CSS propio, hojas imprimibles). Ver `docs/adr/0001-stack.md` y `0002`. Revisión tras H1.
- **Nombres en español:** el código (funciones, tipos, archivos, tablas) y las variables de las fórmulas (`ancho`, `alto`) van en español sin tildes ni eñes.
- **Versión de las notas:** v1, en concreción antes de construir. El backend está detallado en `docs/BACKEND.md`.
- **Servidor:** un PC pequeño dedicado en el taller; la oficina se conecta por navegador y más adelante una tablet de operario.
- **Criterio de elección:** idoneidad técnica, no popularidad (ADR-0002). Cada dependencia nueva exige justificarse.
- **Motivo:** construir una vez y casi no mantener; el mantenimiento lo hará Claude.
- **Arquitectura:** un único servidor con todo; los demás equipos solo abren la web. No hay cliente gráfico aparte.
- **Verifactu:** fuera por ahora. La facturación guarda campos reservados. Hacienda ha anunciado un aplazamiento hasta octubre de 2028 (pendiente de norma).
- **Diseño:** aprobado en `design/` (versión 1) (paleta de papel y grafito con azul acero; IBM Plex; dibujo con piezas numeradas; tablero de trabajos).
- **Alcance MVP:** clientes, catálogo, presupuestos, trabajos con orden de taller, stock y facturas, con una sola serie al principio.
- **Datos como datos:** series, tarifas, IVA y plantillas no son código.
- **Corte:** todo a mano en el taller (sin tronzadora conectada). La hoja impresa es el documento clave.

## Política de versiones
- Toda la documentación está en **versión 1** y se mantiene así hasta que el propietario declare la versión definitiva. Los cambios se hacen sobre la versión 1 y se anotan en `docs/CAMBIOS.md`; no se renumera.
- **Criterios para declarar la definitiva:** 1) lista de preparación completa; 2) documentos por redactar terminados; 3) al menos 3 ventanas reales validadas con sus fixtures; 4) una tarifa real importada de prueba; 5) una revisión cruzada sin contradicciones; 6) visto bueno del propietario.
- Una vez declarada, cualquier cambio exige un ADR nuevo.

## Estado de aprobación
| Documento | Estado |
|---|---|
| ADR-0001 (Rust completo) | aceptado (versión 1) |
| ADR-0002 (tecnologías concretas) | aceptado (versión 1) |
| Diseño de `design/` | aprobado (versión 1) |
| `SPEC.md`, `docs/BACKEND.md`, `CLAUDE.md` | versión 1, borrador |
| `docs/DATOS.md`, `docs/FORMATO_SERIES.md`, `docs/PANTALLAS.md` | versión 1, borrador (2026-11-21); pendiente de la revisión del propietario, sobre todo el menú de `PANTALLAS.md` |

## Datos reales ya obtenidos (en cm)
Todo se maneja en cm (un decimal de cm = 1 mm). Redondeo provisional: a 0,1 cm hacia abajo.

**Aluprom 21 (fabricante Aluval), corredera de 2 hojas**
- Marco lateral: `alto`. Marco superior e inferior: `ancho − 4,1`. Hoja de cierre y hoja central: `alto − 5,3` (2 y 2). Hoja de rodamiento: `ancho/2 − 2` (4 piezas). Todos los cortes a 90°. Barras de 640 cm.
- Accesorios: kit de corredera (1, un único artículo), 4 ruedas, 17 tornillos 4,8 × 25 (9 en el marco y 8 en las hojas).

**Aluprom 21, corredera de 4 hojas (borrador)**
- Igual que la de 2 hojas salvo la hoja de rodamiento: `(ancho − 3) / 4` (primero se resta y luego se divide). El doble de hojas: 4 de cierre, 4 centrales y 8 de rodamiento. 2 kits iguales.
- Supuestos por confirmar: 8 ruedas y 25 tornillos (9 en el marco y 16 en las hojas). Cristal pendiente del descuento.

**Sopena V-94, ventana europea abatible de 2 hojas (borrador)**
- Marco: `ancho` y `alto` sin descuento. Hoja: alto `alto − 4,1` y ancho `(ancho − 4,85) / 2`, 4 piezas de cada. Inversor: `alto − 10,3`. Cortes a 45°.
- Accesorios: 4 escuadras para el marco y 4 por hoja, 1 cremona, 1 kit de 2 hojas, bisagras (2 por hoja hasta 140 cm de alto incluido, 3 por encima). Barras de 640 cm. Pendientes: pletinas de falleba, tipos de escuadra, cantidad de inversores y cristal.

**Proveedores:** las barras de Aluval vienen en 640 cm (confirmado por el propietario, 2026-11-21); las de la V-94 (Sopena), también en 6,40 m.

**Taller:** grosor de sierra 2 mm (provisional, por comprobar).

- Ver `data/series/aluprom-21.yaml`, `data/series/sopena-v94.yaml` y los tres fixtures de `fixtures/real/` (`validado: false`: derivados de las fórmulas, no de hojas de corte reales).

## Lo que falta antes de construir (lista de preparación)
- [ ] Fórmulas de ancho y alto del cristal y tipo de vidrio habitual.
- [ ] Al menos 3 ventanas reales con hoja de corte y precio cobrado, para marcar los fixtures como validados.
- [ ] Una tarifa real de un proveedor (cualquier formato).
- [ ] Definir si `ancho` y `alto` son hueco de obra o medida de la ventana, y si marco superior, lateral e inferior salen del mismo perfil.
- [ ] Cómo se cobra el material (barra, metro o kilo) y margen habitual.
- [ ] Tipos de IVA y casos especiales (confirmar con el gestor).
- [ ] Datos fiscales del taller y logotipo para los PDF; formato de numeración de documentos.
- [x] Servidor: mini PC con Linux; PC de oficina con Windows (navegador). Pendiente: cuándo llega la tablet.
- [x] Serie de 4 hojas (en borrador; ruedas y tornillos son supuestos).
- [x] Dar el visto bueno a ADR-0002 (interfaz generada en el servidor en lugar de React, rusqlite en lugar de SQLx, CSS propio en lugar de Tailwind).
- [x] Dar el visto bueno al diseño de `design/`.
- [ ] Revisar y dar el visto bueno final a `CLAUDE.md`, `SPEC.md` y los ADR.
- [ ] Comprobar en la página oficial qué plan de Claude cubre un uso intensivo de Claude Code.

## Documentos que faltan por redactar (antes de construir)
- [ ] **Dónde viven las series en ejecución:** propuesta de base de datos versionada con YAML para importar y exportar (hoy `CLAUDE.md` y `SPEC.md` no lo cierran).
- [x] **Modelo de datos:** tablas, columnas, relaciones y restricciones (`docs/DATOS.md`).
- [x] **Formato formal de las series:** campos, tipos, versiones y validaciones (`docs/FORMATO_SERIES.md`).
- [x] **Mapa de pantallas, rutas y permisos por rol** (`docs/PANTALLAS.md`), en borrador; el diseño visual de las pantallas nuevas se hace en H0 a H4 con `design/` como referencia.
- [ ] **Entorno de desarrollo y CI:** sistema operativo de desarrollo, servidor de integración y compilación para Windows o Linux.
- [ ] **Criterios de aceptación detallados** (dado, cuando, entonces) de cada hito, al empezar cada uno.

## Cómo ayudar en la conversación de refinamiento
- Revisar con espíritu crítico `CLAUDE.md`, `SPEC.md` y los ADR: incoherencias, huecos, riesgos y decisiones sin justificar.
- No inventar datos reales del taller; marcar `CONFIRMAR`.
- Mantener los archivos organizados y coherentes entre sí; avisar si una decisión nueva contradice otra.
- Ser honesto sobre ventajas e inconvenientes, también de las preferencias del usuario.
