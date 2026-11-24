# Contexto del proyecto (documento de traspaso)

Este archivo permite retomar el trabajo en una conversación nueva sin perder lo decidido.

## Estado
H0 a H7 terminados (2026-11-23): el MVP completo, del presupuesto a la factura, con stock, copias automáticas y migraciones que conservan los datos. El estado al día, la versión publicada y la siguiente tarea están siempre en `docs/PROGRESS.md` (este apartado no se repite para no quedarse viejo); mensaje para pegar en `docs/PROMPT_INICIAL.md`.

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
- Accesorios (herrajes): kit de plásticos de corredera (1, un único artículo), 2 cierres embutidos (uno en cada lateral), 4 ruedas, 17 tornillos 4,8 × 25 (9 en el marco y 8 en las hojas).

**Aluprom 21, corredera de 4 hojas (borrador)**
- Igual que la de 2 hojas salvo la hoja de rodamiento: `(ancho − 3) / 4` (primero se resta y luego se divide). El doble de hojas: 4 de cierre, 4 centrales y 8 de rodamiento. Además, 1 cierre 4 hojas (21344) con el alto de la hoja (`alto − 5,3`). 2 kits iguales y 4 cierres embutidos (dos en los laterales y dos en el centro).
- Supuestos por confirmar: 8 ruedas y 25 tornillos (9 en el marco y 16 en las hojas). Cristal pendiente del descuento.

**Aluprom 21, parteluces (propietario, 2026-11-24; ADR-0030)**
- Uno por hoja: 21564 con cristal normal, 21565 con Climalit. Se escribe la medida de la chapa (o del cristal) de abajo; lo habitual es 50 cm porque se aprovecha la plancha.
- Chapa del color de la ventana: con cristal normal, 1 por hoja lacada por las dos caras; con Climalit, 2 por hoja, de preferencia lacadas por una cara (o por dos si se aprovecha un recorte). Planchas de 200 × 125 y 300 × 125 cm (por confirmar), en los mismos RAL que el aluminio.
- Pendiente: medida de corte del parteluces, su altura real, ancho de la chapa, alto del cristal de arriba y cómo se cobra la chapa.

**Mosquitera enrollable (propietario, 2026-11-24; ADR-0031, en borrador)**
- Cajón de 45 hasta 140 cm de alto; por encima, de 50. Cajón = `ancho − 2`. Guía (2) = `alto − 4` con cajón de 45 y `alto − 5` con cajón de 50.
- El terminal no se corta y la malla no se cuenta: vienen dentro del cajón al cortarlo. Un kit de plástico aparte por mosquitera. Dos tiradores, siempre negros. Mismos colores que las ventanas.
- Se vende suelta; con las persianas podrá ir dentro de la ventana. Pendiente: referencias, proveedor, longitud de barra, cortes a 90° y límites.

**Persianas (propietario, 2026-11-24; ADR-0032 y ADR-0033).** No se fabrican: se piden hechas.
- **En la ventana** (correderas de Aluprom 21): la medida escrita es la del conjunto; guía de 4 cm a cada lado (el marco, 8 cm menos de ancho); el alto del marco, el conjunto menos el cajón. Cinta a la derecha por defecto.
- **Suelta en pared** con guías de L (perfil plano, canal de 53 mm y ala de 80 o 100, por confirmar): se escribe el hueco; va al presupuesto con su dibujo, sola o con la ventana.
- Pendiente: alturas de los cajones, medida que se da al proveedor y cómo se cobran.

**Herrajes (propietario, 2026-11-24; ADR-0029):** cierre de embutir que se usa hoy, Alualpha 267 i (2 por corredera de 2 hojas, 4 por la de 4); kit de plásticos Al21 de Aluval (9426065); tornillo 4,8 × 25 de cabeza redonda y estrella. El modelo concreto lo dice el artículo del catálogo; la serie usa referencias genéricas.

**Sopena V-94, ventana europea abatible de 2 hojas (borrador)**
- Marco: `ancho` y `alto` sin descuento. Hoja: alto `alto − 4,1` y ancho `(ancho − 4,85) / 2`, 4 piezas de cada. Inversor: `alto − 10,3`. Cortes a 45°.
- Accesorios: 4 escuadras para el marco y 4 por hoja, 1 cremona, 1 kit de 2 hojas, bisagras (2 por hoja hasta 140 cm de alto incluido, 3 por encima). Barras de 640 cm. Pendientes: pletinas de falleba, tipos de escuadra, cantidad de inversores y cristal.

**Referencias Aluprom 21 (catálogo Aluval, confirmadas por el propietario el 2026-11-22):** marco lateral 21210, superior 21220, inferior 21230. Hojas con cristal normal: lateral 21310, cierre 21340 (la del gancho, en el centro), rodamiento 21350; con Climalit: 21312, 21341, 21351. En la corredera de 2 hojas se abren las dos y el cierre va en cada lateral.

**Colores:** blanco RAL 9016 brillo por defecto con cierres blancos; con cualquier otro color, cierres negros. Lista de serie en `data/acabados.yaml` (RAL y superficies por confirmar).

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
