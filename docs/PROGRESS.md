# Progreso

Estado (2026-11-22): **H0–H3 terminados**, publicada la **v0.6.1** en `main`. Siguiente: **H4, trabajos y orden de taller** (plan propuesto, falta el visto bueno del propietario; ver abajo).

## Cómo retomar en una sesión nueva
1. Lee `CLAUDE.md`, este archivo, `docs/CONTEXTO.md` y `docs/OPEN_QUESTIONS.md`. Para H4, además `SPEC.md` (H4), `docs/DATOS.md` (3.6, 4 y 7.7), `docs/PANTALLAS.md` (8 y 13), `design/03-hoja-taller.png` y ADR-0003.
2. Trabaja en una rama; al terminar algo que el propietario quiera probar, sube `version` en `Cargo.toml` y llévalo a `main` (ya ha dado permiso para publicar así): GitHub Actions prueba, compila y publica el `.tar.gz` de Linux y el instalador de Windows.
3. Comprueba que todo está en verde. En la nube no hay `just`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`; E2E: `cargo build --release --locked -p servidor` y en `e2e/` `npm ci` y `ALUMFRAME_BIN=../target/release/alumframe npx playwright test --trace off`.
4. Para un hito, propón el plan y **espera la aprobación** antes de escribir código.

## Reglas que conviene no olvidar
- **Sin migraciones** hasta el primer despliegue real (ADR-0011): se amplía `crates/servidor/src/bd/esquema.sql`. Si cambia, en el servidor de pruebas hay que borrar los datos (`docs/MAINTENANCE.md`, apartado 4) y avisar al propietario.
- **Datos de serie que vienen con el programa** y se cargan al arrancar: series (`data/series/`, ADR-0014), colores (`data/acabados.yaml`, ADR-0019) y siluetas de perfiles (`data/perfiles/`, ADR-0017). Se corrigen en el repositorio, no en pantalla.
- **Nada se inventa:** lo no confirmado va como `CONFIRMAR` o queda anotado en `OPEN_QUESTIONS.md`. Sin margen ni modo de cobro elegidos, los presupuestos salen «Provisional».
- Dependencia nueva: ADR corto y versión con más de 14 días. Archivos ≤ 300 líneas, funciones ≤ 40 (clippy lo vigila). Comentarios solo para el porqué.
- **Interfaz:** estilo iOS (`docs/design.md`), adaptada a móvil (barra de pestañas, tablas como celdas; E2E `movil.spec.js`). La CSP no deja estilos en línea: los colores del dibujo van en atributos SVG. Los estilos y scripts llevan `?v=` con la versión.
- **Datos de prueba:** serie y tarifa ficticias en `fixtures/demo/` (E2E con `ALUMFRAME_DATOS_DEMO=si`).

## Instalación del propietario (para pruebas, sin datos reales)
Servidor Ubuntu en casa (`192.168.1.100`, servicio `alumframe`, datos en `/var/lib/alumframe`), acceso desde fuera con Tailscale (`100.104.229.16:9090`). Pasos en `docs/MAINTENANCE.md`, apartado 4. **Pendiente de probar por él:** el instalador de Windows (que no salga consola y que se cierre al cerrar la ventana).

## H4: plan propuesto, a la espera del propietario
Tablas de DATOS 7.7, aceptar crea el trabajo con su copia congelada, hoja de taller A4, asignar operarios, «Mi día» y pasos del operario, modificar medidas con revisiones. Seis decisiones propuestas:
1. Hoja de taller en mm; el resto de pantallas en cm.
2. Plan de corte por trabajo (todas sus ventanas, por perfil y color).
3. Una línea con cantidad 3 son 3 ventanas de trabajo, cada una con sus pasos.
4. Al modificar medidas, el propietario decide con una casilla si se reinician los pasos.
5. Pasos por defecto iguales para todas las ventanas, editables en Ajustes.
6. «Falta material» llega en H5; «Entregado» se marca a mano hasta H6.

## Datos que faltan del taller (lista completa en `OPEN_QUESTIONS.md`)
- **Aluprom 21:** descuento del cristal, referencias de accesorios (kit, ruedas, tornillos), si la de 4 hojas lleva el «cierre 4 hojas» 21344, y si el Climalit cambia descuentos. Los perfiles ya tienen referencia (21210, 21220, 21230; hojas 21310/21340/21350 y Climalit 21312/21341/21351).
- **Precio:** una tarifa real de Aluval, margen (sobre coste o venta) y mano de obra.
- **Colores:** confirmar RAL y superficies de `data/acabados.yaml` y el nombre «Embero».
- **Validación:** 5 ventanas reales con su hoja de corte y precio. Catálogo PDF de la Sopena V-94. Datos fiscales y logotipo.

## Hecho después de H3 (resumen; detalle en `docs/CAMBIOS.md`)
| Versión | Qué |
|---|---|
| 0.4.1 | Series incluidas en el programa (ADR-0014) |
| 0.5.0 | Aplicación de Windows con instalador, en este equipo o contra el servidor (ADR-0015); exportar e importar datos (ADR-0016) |
| 0.5.1 | Versión móvil con estética iOS |
| 0.5.2–0.5.4 | Siluetas de perfiles junto a cada pieza (ADR-0017); hojas con los nombres del catálogo; referencias de Aluprom 21 |
| 0.5.3 | Acristalamiento «Cristal normal» o «Climalit» que cambia las hojas (`perfil_segun`, ADR-0018) |
| 0.5.5–0.5.6 | Arreglos del móvil; corredera de 2 hojas como Aluval (abren las dos, cierre en los laterales) |
| 0.6.0–0.6.1 | Ventana dibujada en el color del acabado, con superficie (brillo, mate, texturado, anodizado) y veta de madera (ADR-0019) |

| Hito | Estado |
|---|---|
| H0 Cimientos | **hecho** (2026-11-21) |
| H1 Motor de dominio | **hecho** (2026-11-21) |
| H2 Catálogo, clientes e importación | **hecho** (2026-11-21) |
| H3 Presupuestos | **hecho** (2026-11-21) |
| H4 Trabajos y orden de taller | pendiente |
| H5 Stock | pendiente |
| H6 Facturación | pendiente |
| H7 Copias y despliegue | pendiente |

## ADR pendientes
- Antes del primer despliegue real: congelar el esquema como migración 0001 (ADR-0011).

## Registro de decisiones y cambios
### H0. Cimientos (2026-11-21)
**Hecho**
- Workspace `crates/dominio` y `crates/servidor`, Rust 1.97.0 fijado, lints que prohíben `unwrap`, `expect`, `panic` y `unsafe`.
- `just check` (formato, clippy `-D warnings`, pruebas, archivos de hasta 300 líneas), hook `.githooks/pre-push`, `cargo deny` y CI en GitHub Actions (Linux, E2E y Windows). Publicación por etiqueta `v*` con ejecutable Linux estático y `.exe` de Windows.
- Servidor Axum en el **puerto 9090**: `/health`, configuración `alumframe.toml` y variables `ALUMFRAME_*`, modos servidor y local, órdenes `migrar` y `restablecer-contrasena`.
- SQLite con WAL, `synchronous=FULL`, un escritor y cuatro lectores; (luego sustituidas por el esquema único, ADR-0011) migraciones 0001 (usuario, sesion, bloqueo_ip, auditoria y version_esquema) y 0002 (sesión hasta cerrar el navegador), copia previa a cada migración y negativa a arrancar con una base más nueva.
- Acceso: asistente de primer arranque, propietario con contraseña (argon2id) y operario con PIN de 6 cifras en teclado táctil, sesiones con cookie `HttpOnly` (2 h de inactividad y 12 h máximo, o hasta cerrar el navegador si el propietario lo marca al entrar), CSRF por origen y token, bloqueo progresivo por usuario y por IP (1, 5 y 15 min), permisos en la capa de aplicación (404 para lo ajeno) y auditoría de solo añadir.
- Interfaz: menú de 8 entradas, Inicio vacío con el tablero, Mi día del operario, Ajustes > Usuarios (alta, cambio de PIN y desactivar). IBM Plex y HTMX incluidos.
- Despliegue: servicio systemd e instalador (`despliegue/linux`), acceso directo de Edge en modo aplicación para Windows (`despliegue/windows`).
- Pruebas: 41 unitarias, 22 de integración y 4 E2E.

**Decisiones:** ADR-0004 (Askama), 0005 (zona horaria propia), 0006 (configuración, puerto 9090, servidor Linux), 0007 (dependencias de H0) y 0008 (CI, cobertura y publicación). Tabla `bloqueo_ip` añadida al modelo de datos para el bloqueo por IP que pide `BACKEND.md`.

**Pendiente**
- La cobertura del 90 % del dominio se exigirá en CI desde H1.
- Pendiente de confirmar: menú de 8 entradas.
- Los `fixtures/demo` ya venían en el paquete; se usarán en los E2E desde H3.

### H1. Motor de dominio (2026-11-21)
**Hecho**
- Unidades exactas: `Decimal` de punto fijo (4 decimales, sin `f64`), `Longitud` en décimas de mm, `Dinero` en céntimos y redondeo abajo, arriba o cercano.
- Evaluador de fórmulas propio: `+ - * /`, paréntesis, `minimo`, `maximo`, `redondear`, `techo`, `suelo` y `si(...)` con comparaciones; límites de 200 caracteres y 16 niveles; errores en español; pruebas de propiedades.
- Series como datos: lectura con `yaml-rust2` a un árbol neutro y validación completa en el dominio (apartado 11 de FORMATO_SERIES.md), con todos los errores a la vez y avisos por cada `CONFIRMAR`.
- Cálculo de una ventana: piezas numeradas, vidrio, accesorios desglosados y sumados, límites y lista de pendientes; la línea sale **incompleta** mientras falte cualquier dato.
- Optimizador Best Fit Decreasing con grosor de sierra, recorte y retal; pruebas de propiedades y comparación con la solución óptima en casos pequeños.
- **Tests dorados:** las 3 ventanas de `fixtures/real/` coinciden con el cálculo (criterio de aceptación).
- Servidor: migración 0003 (series versionadas), Catálogo > Series, importación en 3 pasos con vista previa y pantalla **«Probar una ventana»** con el plan de corte.
- Pruebas: 132 entre unitarias e integración y 5 E2E; cobertura del dominio 95,7 %, exigida en CI (mínimo 90 %).

**Decisiones:** ADR-0009 (series como YAML versionado en la base de datos) y ADR-0010 (`yaml-rust2`). Bisagras de la abatible confirmadas por el propietario. **Revisión de ADR-0001 tras H1:** el motor salió sin atascos; se mantiene Rust.

**Pendiente**
- El **coste y el precio** de la ventana se calculan en H2/H3, cuando existan tarifas y se confirme si el margen va sobre coste o sobre venta.
- Grosor de sierra, recorte de extremos y retal mínimo: se usan 0 hasta que el taller los dé (`OPEN_QUESTIONS.md`).
- Validar una serie (pasarla de borrador a validada) con ventanas reales: acción explícita que llegará con el punto de validación tras H3.

### Después de H1 (2026-11-21)
- **Esquema único sin migraciones** hasta el primer despliegue (ADR-0011, decisión del propietario).
- **Movimiento al estilo iOS** sin cambiar colores ni estructura: transiciones entre pantallas, indicador del menú que se desliza, respuesta al pulsar, barra superior translúcida y PIN con puntos y entrada automática (`docs/design.md`).
- **Ajustes > Taller:** grosor de sierra (2 mm provisional, dado por el propietario), recorte de extremos y retal mínimo, guardados en la tabla `configuracion` y usados por «Probar una ventana».
- La serie V-94 usa barras de 6,40 m (confirmado).

### H2. Catálogo, clientes e importación (2026-11-21)
**Hecho**
- **Clientes:** lista con búsqueda mientras se escribe (sin tildes ni mayúsculas) y paginación, alta sin ningún campo obligatorio («Cliente sin nombre (n.º N)»), NIF/NIE/CIF comprobado al teclear, varias direcciones de montaje (y «la misma que la fiscal»), ficha, edición con aviso si otra persona guardó antes sin perder lo escrito, archivo, **exportación y borrado definitivo** (RGPD). El buscador de la barra superior busca clientes.
- **Importar clientes** desde CSV en 3 pasos; los NIF que ya existen se saltan con aviso; deshacer quita los no modificados.
- **Catálogo:** pestañas Artículos, Proveedores, Acabados (un único acabado base sin recargo), Tarifas y Series (con «referencias con artículo»).
- **Tarifas:** importación CSV en 3 pasos con vista previa (nuevos, cambios con diferencia en %, sin cambios, ausentes, errores y avisos; salto > 30 % avisado), confirmación que vuelve a analizar y exige la misma huella SHA-256, **versiones completas** por proveedor, edición de precios en la tabla y ajuste por porcentaje como versiones nuevas, histórico por artículo y **deshacer** la versión vigente.
- Dominio nuevo: lector CSV propio, NIF, `PrecioUnitario` (1/10 000 €), `Porcentaje` (puntos básicos), lectura y comparación de tarifas, validación de clientes y del catálogo.
- **Criterios de aceptación:** «importar un CSV con errores muestra los avisos y no guarda nada hasta confirmar» (`tests/tarifas.rs::un_csv_con_errores_muestra_los_avisos_y_no_guarda_nada` y E2E `h2-tarifas`) y «deshacer restaura los precios» (`deshacer_restaura_los_precios_y_quita_los_articulos_nuevos` y E2E): **cumplidos**.
- Pruebas: 224 entre unitarias e integración y 12 E2E, todas en verde; cobertura del dominio 96,7 % de líneas.

**Decisiones:** ADR-0012 (solo CSV, todo o nada, versiones completas, deshacer la última de cada proveedor). `proveedor.descuento_habitual_bp` añadido. El navegador lee el CSV como UTF-8 o Windows-1252. Los mensajes de confirmación viajan como claves fijas en la URL, nunca como texto libre.

**Pendiente**
- Una **tarifa real** para comprobar su formato (y si compensa leer Excel directamente).
- `kit_componente` (H5) y recargo de color por proveedor (decisión 4 de DATOS).
- Al llegar los presupuestos (H3): bloquear deshacer una tarifa usada y anonimizar en lugar de borrar clientes con documentos.
- `cargo deny` no está instalado en las sesiones en la nube; se ejecuta en CI.

### H3. Presupuestos (2026-11-21)
**Hecho**
- **Dibujo SVG** en el dominio: corredera de 2 y 4 hojas, abatible con bisagras y cremona, fija; piezas numeradas como en la hoja de taller, cotas y capas (Cotas, Piezas, Cristal). Mismo dibujo en pantalla, en las miniaturas y en la hoja impresa.
- **Precio** en el dominio: perfiles por barras enteras del plan de corte o por metros con merma (se calculan los dos), cristal por m², accesorios por unidad, metro o caja, descuento de compra, recargo de color, margen sobre coste o sobre venta, mano de obra, cantidad y descuento; totales con descuento general, base y cuota por tipo de IVA en un único sitio.
- **Presupuestos:** numeración `P-AAAA-NNNN`, versiones, estados (borrador, enviado, aceptado, rechazado), líneas de ventana y libres, dirección de montaje, validez, condiciones y descuento. «Recalcular precios» si la tarifa cambió. **Aceptar** exige todo completo y congela una copia con su SHA-256; la tarifa usada ya no se puede deshacer y el cliente con presupuestos no se puede borrar.
- **Pantallas al estilo iOS** (petición del propietario): editor de línea como `design/01-presupuesto.png` con cálculo en vivo, control segmentado con píldora deslizante, hojas (`<dialog>`) que entran y salen, avisos efímeros, tarjetas de línea con miniatura y pie fijo oscuro con el desglose y el total; los avisos de confirmación de todo el programa pasan a ser efímeros. **Ajustes > Empresa y precios.** Hoja **A4 imprimible** (o PDF desde el navegador) sin costes.
- **Criterio de aceptación:** presupuesto de 3 ventanas distintas y una línea libre, con dibujo, imprimible, y total igual al del dominio (`tests/presupuestos.rs::tres_ventanas_y_una_linea_libre_suman_el_total_del_dominio` y E2E `h3-presupuestos`): **cumplido** con datos ficticios.
- Pruebas: 248 entre unitarias e integración y 15 E2E, en verde; cobertura del dominio 95,8 % de líneas.

**Decisiones:** ADR-0003 (PDF desde el navegador hasta H6), ADR-0013 (`serde_json`). Margen, cobro de perfiles y mano de obra se eligen en Ajustes; sin elegir, no hay precio final. IVA 21 % y 10 % por defecto (cuándo aplica el reducido, con el gestor). La mano de obra se suma después del margen (provisional). Una serie de demostración nunca da precio final: la aceptación se prueba con una copia «validada» que solo existe en `crates/servidor/tests/datos/`.

**Pendiente**
- **Punto de validación** (SPEC H3): 5 ventanas reales comparadas con el cálculo a mano. Hacen falta los descuentos del cristal, las referencias de proveedor de las series y una tarifa real; hoy las series reales salen incompletas, como debe ser.
- El trabajo que nace al aceptar llega en H4. Anonimizar clientes con documentos, en H6.
- Posición real de las bisagras en el dibujo y logotipo en la hoja impresa.

