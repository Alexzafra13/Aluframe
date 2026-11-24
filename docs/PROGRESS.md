# Progreso

Estado (2026-11-24): **H0–H7 terminados**; publicada la **v0.21.3** (la mosquitera sin vidrio ni parteluces en el editor; las opciones cambian al cambiar de serie). La **v0.21.2** (siluetas de la guía y el cajón de la mosquitera). La **v0.21.1**: mosquitera enrollable como serie en borrador (ADR-0031), con la guía del cajón de 45 (alto − 4), sin terminal que cortar y dos tiradores negros. Antes, la **v0.20.1** (arreglos de la 0.20.0: cota de la chapa dentro de su hueco, parteluces pendiente en la lista de corte, «4,8», descripción corta con detalles). La **v0.20.0**: parteluces en las correderas de Aluprom 21 con chapa o cristal abajo, su dibujo y el de la chapa a escala (ADR-0030); faltan las medidas del propietario. Antes, publicada la **v0.19.0**: correderas de Aluprom 21 completas con el cierre 4 hojas 21344, los cierres embutidos y los herrajes aparte de los perfiles a cortar, con su dibujo y su modelo; la ventana dice su tipo y su serie con el logotipo de la marca (ADR-0029). Antes, la **v0.18.2**: la demostración, sin contraseña ni PIN (se entra directamente y se cambia de usuario con un toque). Antes, la v0.18.1: si la aplicación de Windows no arranca por la base de datos, dice la carpeta y ofrece abrirla o «Empezar de cero» (aparta la base, no la borra); la v0.18.0 con la **demostración** de datos inventados (ADR-0028), y la v0.17.0 (llevada a `main` con permiso del propietario): arreglos de la segunda auditoría (antes 0.16.1, que no llegó a publicarse), mejor aprovechamiento de las barras con medidas habituales e historial (ADR-0026) y caducidad de los presupuestos (ADR-0027, migración 0004). Los datos se conservan al actualizar (migraciones, ADR-0021) y hay copias automáticas (ADR-0022).

**Fase actual (2026-11-23): versión mínima.** El propietario instala la versión publicada, la prueba (también con la demostración, ADR-0028) y apunta todo lo que vea mal; con su lista se arregla y se pule hasta tener una versión mínima para el taller. Los arreglos que pida se hacen directamente; una tarea nueva grande, con plan y aprobación. **Los datos que faltan (apartado «Datos que faltan del taller») los pasará cuando los tenga: no se inventan, se esperan.**

**Chapa (planteado con el propietario, 2026-11-24, ADR-0030):** fase 1 hecha (0.20.0). **Fase 2:** la chapa como material en catálogo y stock, con color (los RAL del aluminio), lacada por 1 o 2 caras y planchas de 200 × 125 y 300 × 125 cm (por confirmar); migración nueva. **Fase 3:** aprovechar recortes de chapa (optimizador 2D). Proponer plan de cada fase y esperar aprobación.

**Mosquiteras (ADR-0031):** serie en borrador hasta tener los datos de `OPEN_QUESTIONS.md`. **Persianas:** cuando se hagan, la ventana con persiana podrá llevar mosquitera; decidir entonces cómo (plan y aprobación).

**Después:** importar tarifas desde **Excel** (.xlsx, .xls, .ods, directo) y **PDF** (asistido: se extrae el texto, se proponen las columnas y el propietario revisa), entrando en la misma revisión en 3 pasos que ya usa el CSV (`aplicacion/importar_tarifa.rs`, ADR-0012). Necesita ADR de dependencia (por ejemplo `calamine` para hojas de cálculo; para PDF, valorar extracción de texto propia o una biblioteca) con las seis preguntas de `CLAUDE.md`. Proponer el plan y esperar aprobación. Después, el propietario simula con datos a mano y se pule la interfaz con su lista.

**En curso (plan aprobado por el propietario, 2026-11-23; ADR-0024):** cuentas del dueño y de operarios con lo justo. Parte 1, permisos por operario: **hecha (0.14.0)**. Parte 3, pantalla del operario para la tablet: **hecha (0.16.0)**. **El plan está terminado**; falta que el propietario lo pruebe en el taller. Parte 2, apuntes: **hecha (0.15.0, ADR-0025)**: notas y tareas sueltas para lo que no es una ventana; del operario al dueño (le llegan a la campana) y del dueño a un operario (le salen en «Mi día»), con cliente y dirección opcionales, medidas opcionales, estados pendiente/hecho/pasado a presupuesto y un botón «Hacer presupuesto». Migración nueva que solo añade tablas. Parte 3: pantalla del operario para la tablet (Mi día · Apuntes · Clientes y lo permitido; la cabecera actual de «Mi día» se reparte en dos líneas en el móvil). Fotos en los apuntes, más adelante.

**Segunda auditoría (2026-11-23, v0.16.1):** cuatro fallos de facturas y clientes corregidos con su prueba (detalle en `docs/CAMBIOS.md`). Queda pendiente del propietario cómo volver a facturar un trabajo cuya factura se anuló (`OPEN_QUESTIONS.md`). Vistos sin arreglar, por ser poco probables con un solo dueño: «Terminar orden» recalcula el plan al confirmar y, si entre medias otra orden gastó un retal, los retales revisados se asignan por posición del perfil; y la lista de Presupuestos muestra los 500 últimos sin paginar (los antiguos siguen en la ficha de cada cliente).

**Optimizador (hecho en la 0.17.0, ADR-0026):** mejor aprovechamiento con búsqueda acotada por pasos, medidas habituales e historial. Falta la fase que depende de datos: cuando cada serie tenga sus medidas mínimas (`limites`, hoy `CONFIRMAR`), el mínimo útil de cada perfil será su pieza más corta posible y lo que quede por debajo saldrá desmarcado como desecho en «Terminar orden». Valorar también `cargo-mutants` de vez en cuando sobre `dominio::precio` y `dominio::corte`.

## Cómo retomar en una sesión nueva
1. Lee `CLAUDE.md`, este archivo (sobre todo «Siguiente tarea», «Reglas» y «Antes de publicar»), `docs/CONTEXTO.md` y `docs/OPEN_QUESTIONS.md`. Para tocar la interfaz, `docs/design.md` y `docs/PANTALLAS.md`; para datos, `docs/DATOS.md`; los porqués, en `docs/adr/` y `docs/CAMBIOS.md`.
2. Trabaja en una rama; al terminar algo que el propietario quiera probar, sube `version` en `Cargo.toml` y llévalo a `main` (ya ha dado permiso para publicar así): GitHub Actions prueba en Linux y Windows, compila y publica el `.tar.gz` de Linux y el instalador de Windows.
3. Para una tarea nueva grande, propón el plan y **espera la aprobación**. Los arreglos y retoques que pida el propietario se hacen directamente.
4. Al responder al propietario: en español llano, sin jerga; dale el comando de actualización del servidor con la versión nueva y dile si hay que borrar datos (desde 0.11.0, nunca).

## Antes de publicar (lecciones aprendidas)
En la nube no hay `just`; ejecuta lo mismo que `just check` y la E2E:
```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
find crates -name '*.rs' -exec wc -l {} + | awk '$1>300 && $2!="total"'   # debe salir vacío
cargo test --workspace --locked
cargo build -p servidor --offline        # tras subir la versión: actualiza Cargo.lock
cargo build --release --locked -p servidor
cd e2e && npm ci && ALUMFRAME_BIN=../target/release/alumframe npx playwright test --trace off
```
- La comprobación de tamaño (300 líneas) la pasa CI aunque clippy no la vea: ya rompió una publicación.
- Pasa clippy **después** de `cargo fmt`: el formateador alarga funciones y una prueba de 38 líneas pasó a 42, y CI falló (0.19.0). El flujo de publicar no pasa clippy, así que la versión sale igualmente y solo CI queda en rojo.
- CI también prueba en **Windows**: cuidado con lo que dependa de finales de línea (los `.sql` van con LF por `.gitattributes`; una huella de un archivo de texto puede cambiar con CRLF) o de rutas con `/`.
- Si cambias la versión, primero `cargo build --offline` (actualiza `Cargo.lock`) y después la compilación `--locked`, o fallará.
- Revisa con capturas de Playwright lo visual que cambies (PC a 1366 × 768 y 1366 × 620, e iPhone 13); espera ~1 s a que terminen las animaciones antes de capturar.
- El disco de la nube es pequeño: si se llena, borra `target/debug` o carpetas de capturas del scratchpad.

## Reglas que conviene no olvidar
- **Migraciones desde la 0.11.0** (ADR-0021): cada cambio de la base es un archivo nuevo en `crates/servidor/src/bd/migraciones/` que conserva los datos; **las publicadas no se editan**. Una migración destructiva exige preguntar al propietario.
- **Datos de serie que vienen con el programa** y se cargan al arrancar: series (`data/series/`, ADR-0014), colores (`data/acabados.yaml`, ADR-0019) y siluetas de perfiles (`data/perfiles/`, ADR-0017). Se corrigen en el repositorio, no en pantalla.
- **Nada se inventa:** lo no confirmado va como `CONFIRMAR` o queda anotado en `OPEN_QUESTIONS.md`. Sin margen ni modo de cobro elegidos, los presupuestos salen «Provisional».
- Dependencia nueva: ADR corto y versión con más de 14 días. Archivos ≤ 300 líneas, funciones ≤ 40 (clippy lo vigila). Comentarios solo para el porqué.
- **Interfaz:** estilo iOS (`docs/design.md`), adaptada a móvil (barra de pestañas, tablas como celdas; E2E `movil.spec.js`). La CSP no deja estilos en línea: los colores del dibujo van en atributos SVG. Los estilos y scripts llevan `?v=` con la versión.
- **Datos de prueba:** serie y tarifa ficticias en `fixtures/demo/` (E2E con `ALUMFRAME_DATOS_DEMO=si`).

## Instalación del propietario (pruebas; sus datos ya se conservan al actualizar)
Servidor Ubuntu en casa (`192.168.1.100`, servicio `alumframe`, datos en `/var/lib/alumframe`), acceso desde fuera con Tailscale (`100.104.229.16:9090`). Actualizar: `docs/MAINTENANCE.md`, apartado 4 (descargar el `.tar.gz` de la versión y `sudo ./instalar.sh`). **Pendiente de probar por él:** disco de copias montado en el servidor (`despliegue/linux/LEEME.txt`), que el servicio arranca solo tras reiniciar, y el instalador de Windows.

## H4: plan aprobado (sin objeciones del propietario, 2026-11-22) y hecho
Tablas de DATOS 7.7, aceptar crea el trabajo con su copia congelada, hoja de taller A4, asignar operarios, «Mi día» y pasos del operario, modificar medidas con revisiones. Seis decisiones propuestas:
1. Hoja de taller en mm; el resto de pantallas en cm.
2. Plan de corte por trabajo (todas sus ventanas, por perfil y color).
3. Una línea con cantidad 3 son 3 ventanas de trabajo, cada una con sus pasos.
4. Al modificar medidas, el propietario decide con una casilla si se reinician los pasos.
5. Pasos por defecto iguales para todas las ventanas, editables en Ajustes.
6. «Falta material» llega en H5; «Entregado» se marca a mano hasta H6.

## Decisiones del propietario (2026-11-22)
- **Señales:** sí. Un cobro puede colgar del trabajo antes de que exista la factura; al emitirla, cuenta como pagado a cuenta (DATOS 7.9).
- **Orden:** el propietario lo deja a nuestro criterio. Primero **H4** (el trabajo nace al aceptar y es donde se apunta la señal), después **H6** (facturas y cobros: medio efectivo, Bizum, tarjeta, transferencia u otro; pagos parciales y estado calculado).

## Decisiones del propietario para H5 y H7 (2026-11-22)
- **Stock:** el presupuesto y el trabajo no tocan el stock; se descuenta al **terminar la orden**. Durante la fabricación se apunta **«Repetir pieza»** (corte mal, rayón, golpe…) y ese perfil de más también se descuenta. Al terminar se **revisan los retales** que propone el plan antes de que entren en stock. Las correcciones, siempre como ajuste con motivo.
- **Actualizaciones:** sin migraciones hasta la versión mínima; desde que el propietario empiece a usarlo de verdad, esquema fijado y migraciones que nunca pierden datos (H7).
- **Copias (H7):** en un pendrive o disco conectado, reconocido por una marca en la carpeta `AlumFrame-copias` (no por la letra). Copia diaria **a las 14:00 y al cerrar el programa**; si no se hizo (apagón), **al arrancar**. Se comprueba la copia tras hacerla. Se guardan **7 diarias** y **4 semanales**, más una en el propio equipo. Aviso en el Inicio si falla o falta el disco. Mismo formato que Exportar/Importar. En modo servidor, igual en el servidor; la copia fuera de casa (el PC del taller baja la del servidor) queda para después del MVP.

## Datos que faltan del taller (lista completa en `OPEN_QUESTIONS.md`)
El propietario los pasará todos cuando los tenga (2026-11-23). Mientras, el programa funciona con lo que hay y marca lo que falta.
- **Medida mínima (y máxima) de cada serie y tipología** (`limites`, hoy `CONFIRMAR`): avisan de medidas imposibles y, para el corte, marcarán como desecho lo que sobre por debajo de la pieza más corta (ADR-0026).
- **Cristal:** descuentos de la 4 hojas de Aluprom 21, comprobar el alto de la 2 hojas con un cristal real, si el Climalit cambia algo y el tipo de vidrio habitual.
- **Cómo manda Aluval las tarifas** (Excel, PDF, papel o web): decide la siguiente tarea de importación.
- **Precio:** una tarifa real de Aluval (y luego Sopena), margen (sobre coste o venta), mano de obra y cómo se cobra el perfil (barra entera o metros). Sin margen elegido, los presupuestos salen «Provisional».
- **Facturas:** cómo volver a facturar un trabajo cuya factura se anuló con una rectificativa (con el gestor).
- **Aluprom 21:** cristal de la 2 hojas puesto (ancho del propietario; **alto provisional**, comprobar con un cristal real); falta el cristal de la 4 hojas y si el Climalit cambia descuentos. Kit de plásticos, cierres embutidos, ruedas, tornillos y cristal usan referencias genéricas del taller (`KIT-CORREDERA`, `CIERRE-EMBUTIDO`, `RUEDA-CORREDERA`, `TORNILLO-4,8X25`, `CRISTAL`) que se dan de alta con precio en Catálogo > Artículos; la descripción del artículo dice el modelo que se compra (cierre de embutir Alualpha 267 i; tornillo 4,8 × 25 de cabeza redonda y estrella) y sale bajo el herraje (ADR-0029).
- **Sopena V-94:** en borrador hasta tener su catálogo.
- **Colores:** confirmar RAL y superficies de `data/acabados.yaml` y el nombre «Embero».
- **Gestor:** IVA y menciones legales de la factura; Verifactu (fuera del MVP).

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
| 0.7.1 | Arreglo de la aplicación de Windows: no se apaga mientras la ventana siga abierta |
| 0.7.2 | Editor: «Guardar ventana» vuelve al presupuesto; «+ Otro concepto» (antes línea libre); aviso de medida fuera de la serie solo cuando ocurre; menú lateral fijo y plegable; Aluprom 21 validada por el propietario |
| 0.21.3 | Al cambiar de serie en el editor se cambian sus opciones (arreglo); sin cristal se ocultan el vidrio y la capa «Cristal» |
| 0.21.2 | Siluetas de la guía y el cajón de la mosquitera, de los planos del propietario |
| 0.21.1 | Mosquitera: guía con cajón de 45 a alto − 4; el terminal no se corta; dos tiradores negros en el dibujo |
| 0.21.0 | Mosquitera enrollable (serie en borrador): dibujo, cajón de 45 o 50 según el alto; piezas con cantidad 0, vidrio opcional; «+ Ventana o mosquitera» (ADR-0031) |
| 0.20.1 | Cota de la chapa dentro de su hueco; piezas sin medida en la lista de corte como «pendiente»; «Tornillo 4,8x25»; tarjetas de ventana con título corto y «Ver detalles» |
| 0.20.0 | Parteluces en las correderas de Aluprom 21: opción con chapa o cristal abajo y su medida, dibujo con la cota, chapa con su dibujo técnico a escala; formato de series con opciones de medida, `solo_si` y chapas (ADR-0030) |
| 0.19.0 | Aluprom 21: cierre 4 hojas 21344 en la de 4 hojas, cierres embutidos (2 y 4), dibujo de la 4 hojas con un cierre en cada lateral y dos en el centro; herrajes aparte de los perfiles a cortar, con dibujo y el modelo del catálogo; tipo de ventana y serie con el logotipo de la marca (ADR-0029) |
| 0.18.2 | Demostración sin contraseña ni PIN: «Probar la demostración» entra como el dueño y «Cambiar de usuario» deja elegir al operario con un toque; la franja también en «Mi día» (no se podía salir desde ahí) |
| 0.18.1 | Aplicación de Windows que no arranca por la base: dice la carpeta de datos, «Abrir la carpeta de datos» y, con una base de desarrollo antigua, «Empezar de cero» (la aparta en `apartadas`, no la borra) |
| 0.18.0 | Demostración con datos inventados dentro del programa: se entra y se sale desde la pantalla de entrada, base aparte y marcada, «Empezar de nuevo» (ADR-0028) |
| 0.17.0 | Plan de corte por serie, perfil y color (arreglo); mejor aprovechamiento de barras, medidas habituales e historial (ADR-0026); presupuestos que caducan a los 30 días, se renuevan y se archivan (ADR-0027, migración 0004); y lo de la segunda auditoría (abajo) |
| (0.16.1, sin publicar) | Segunda auditoría: factura anulada por rectificativa ya no sale vencida («Anulada»); avisos de cobro sin el límite de 500 facturas; todos los clientes al crear presupuesto o factura; fecha de la operación congelada e impresa; precio unitario de ventana con descuento que cuadra |
| 0.16.0 | Pantalla del operario para la tablet: cabecera en una línea y botones grandes (Nuevo apunte, Apuntes, Clientes y lo permitido) encima de sus tareas y ventanas. Termina el plan «dueño y operarios» |
| 0.15.0 | Apuntes (ADR-0025): notas y tareas sueltas; del operario al dueño (campana e Inicio) y del dueño a un operario («Mi día»); «Hacer presupuesto» desde el apunte. Migración 0003 |
| 0.14.0 | Permisos por operario (ADR-0024): busca clientes y, si el dueño lo marca, los da de alta, ve presupuestos (sin costes), facturas o todo el taller. Migración 0002 |
| 0.13.1 | Auditoría: el instalador de Windows solo se publica si pasan las pruebas en Windows; ejecutable de 11,5 a 9,3 MB; recursos guardados en la tablet hasta la versión siguiente; lector de la base libre |
| 0.13.0 | La calle sugiere las del callejero del INE (provincia de Badajoz) y rellena localidad y código postal; localidades con el artículo delante («Los Santos de Maimona») (ADR-0023) |
| 0.12.2 | La localidad sugiere mientras se escribe y, al elegirla, pone su código postal si es único; con varios deja elegir (ADR-0020, cambio del 2026-11-23) |
| 0.12.1 | Arreglo: las bases creadas por las versiones de Windows 0.8–0.10 (huella con CRLF) se reconocen; `.gitattributes` con LF para `.sql` |
| 0.12.0 | Campana de avisos en la barra superior; menú sin títulos de grupo |
| 0.11.1–0.11.2 | Notificaciones bajo la barra superior; menú lateral completo en pantallas bajas, plegar solo con icono, tarjeta de usuario y Salir en rojo |
| 0.11.0 | **H7:** copias automáticas (14:00, al cerrar, al arrancar si faltó) en el equipo y en el disco con la carpeta `AlumFrame-copias`, 7 diarias y 4 semanales, aviso en el Inicio, Ajustes > Copias con restaurar; esquema fijado como migración 0001: **los datos ya no se borran al actualizar** |
| 0.10.0 | **H6:** facturas desde el trabajo o libres, número sin huecos al emitir, hoja A4, rectificativas, cobros parciales (efectivo, Bizum, tarjeta, transferencia u otro), señales a cuenta, albarán de entrega; cobros vencidos y presupuestos sin respuesta en el Inicio. Sin cambio de esquema |
| 0.9.0 | **H5:** stock con entradas, ajustes, retales y mínimos; el plan de corte usa los retales del stock y la hoja dice cuál coger; repetir pieza; terminar orden descontando el plan con retales revisados; lista de la compra; stock bajo en el Inicio |
| 0.8.0 | **H4:** trabajos al aceptar, tablero, ficha, orden de taller A4 en mm con plan de corte del trabajo, Mi día del operario con pasos, modificar medidas con revisión; tablas de H5 y H6 ya creadas |
| 0.7.0 | Alta de cliente con nombre y teléfono obligatorios (sin prefijo, +34 por defecto); código postal que rellena localidad, provincia y país (ADR-0020); hoja del presupuesto con logotipo o nombre de empresa, datos completos, firma y pie; descripción de la ventana con serie, medidas, color, opciones y cristal |

| Hito | Estado |
|---|---|
| H0 Cimientos | **hecho** (2026-11-21) |
| H1 Motor de dominio | **hecho** (2026-11-21) |
| H2 Catálogo, clientes e importación | **hecho** (2026-11-21) |
| H3 Presupuestos | **hecho** (2026-11-21) |
| H4 Trabajos y orden de taller | **hecho** (2026-11-22) |
| H5 Stock | **hecho** (2026-11-22) |
| H6 Facturación | **hecho** (2026-11-22) |
| H7 Copias y despliegue | **hecho** (2026-11-22) |

## ADR pendientes
- Servicio de Windows: solo si algún día el servidor del taller es Windows (ADR-0022).
- PDF automático en el servidor (ADR-0003), si hace falta enviar documentos por correo desde el programa.

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

### H4. Trabajos y orden de taller (2026-11-22)
**Hecho**
- Al aceptar nace el trabajo (misma transacción) con una ventana por unidad, sus pasos (Ajustes > Taller) y la copia congelada de cada ventana (`aplicacion/trabajos/congelado.rs`).
- Dominio `dominio::taller`: pasos y sus acciones, estado del trabajo, columna del tablero y plan de corte de todo el trabajo por perfil y color.
- Pantallas: tablero del Inicio, Trabajos, ficha del trabajo, orden de taller A4 en mm (una hoja por ventana y el plan de corte con barras a escala), Mi día y ventana del operario, Modificar medidas con comparación.
- Permisos: `GestionarTrabajos` (propietario) y `MarcarPasos` (operario, solo sus ventanas: lo ajeno da 404). El operario no ve precios.
- **Criterios de aceptación:** cambiar una fórmula tras aceptar no altera el trabajo (`tests/trabajos.rs::cambiar_la_serie_despues_de_aceptar_no_altera_el_trabajo`); el operario no ve precios ni modifica medidas y el propietario sí, con revisión y motivo, y la orden impresa muestra la revisión (`tests/trabajos_pantallas.rs::modificar_medidas_deja_revision_con_motivo_y_la_orden_la_muestra`, E2E `h4-taller`): **cumplidos**.

**Decisiones:** las seis del plan (mm en la hoja, plan de corte por trabajo, una ventana por unidad, casilla para reiniciar pasos, pasos editables, entregado a mano). El aviso «falta algo» vive en la ventana. Modificar medidas no cambia el importe del trabajo (provisional). Las tablas de H5 y H6 se crean ya, para borrar los datos de prueba una sola vez.

**Pendiente**
- Terminar la orden y descontar stock (H5); albarán y factura desde el trabajo, y señales (H6).
- La medida del cristal de Aluprom 21 sigue sin dato: sus ventanas no se pueden aceptar todavía (las pruebas usan una serie ficticia «validada»).

### H5. Stock (2026-11-22)
**Hecho**
- Libro de stock de solo añadir: entradas (inventario inicial incluido), ajustes con motivo que no dejan negativo, barras enteras y retales con su largo, por artículo y color. Mínimos por artículo, «Bajo mínimo» en el stock y en el Inicio, y lista de la compra (lo que necesitan los trabajos con la orden abierta más el mínimo, menos lo que hay).
- `dominio::corte::optimizar_con_retales`: los retales del stock del mismo perfil y color se llenan antes de abrir barras nuevas (pruebas de propiedades). La hoja del plan de corte dice «Coger el retal de 3000 del stock».
- **Repetir pieza** (corte mal, rayón, golpe…) desde «Mi día» o desde el trabajo: entra en el plan y se descuenta.
- **Terminar orden y descontar stock:** muestra lo que sale y los avisos (artículos que no están en el catálogo), y los retales que propone el plan para revisarlos (quitar, corregir el largo, añadir). Una transacción, una sola vez; el plan cortado queda congelado en la orden.
- El enlace perfil o accesorio → artículo va por la tabla `referencias` de la versión de la serie con que se aceptó cada ventana.
- **Criterios de aceptación:** terminar una orden descuenta exactamente el material optimizado (`tests/stock.rs::terminar_la_orden_descuenta_el_plan_y_los_accesorios_una_sola_vez`); el stock no se edita sin un ajuste (`los_ajustes_llevan_motivo_no_dejan_negativo_y_el_libro_no_se_edita`); con retales del mismo perfil y color el plan los usa antes que barras nuevas y la hoja dice cuál coger (`el_plan_usa_el_retal_del_stock_y_al_terminar_entra_el_retal_confirmado`, `tests/stock_pantallas.rs`, E2E `h5-stock`): **cumplidos**.

**Decisiones:** el cristal no se lleva en stock (se pide a medida). Los accesorios se cuentan en unidades de uso (piezas o metros), no en cajas. Una orden puede dejar un artículo en negativo si no se hizo el inventario: se ve en el stock y se corrige con un ajuste. Las reservas de stock al aceptar (decisión 9 de DATOS) no entran en el MVP. Nueva tabla `pieza_repetida`: **hay que borrar los datos de prueba al actualizar**.

### H6. Facturación (2026-11-22)
**Hecho**
- Dominio `dominio::factura`: importe de línea, estado de cobro calculado (pendiente, parcial, pagada, vencida), avisos antes de emitir (sin NIF) y código `F-AAAA-NNNN` (rectificativas, serie `R`).
- **Factura del trabajo** (una ordinaria por trabajo) con las líneas del presupuesto aceptado, o **libre** para un cliente; en borrador se añaden y quitan líneas («Otro concepto») y se cambian fechas, vencimiento, notas y descuento.
- **Emitir:** el número se asigna en la misma transacción que pasa a emitida (sin huecos aunque se descarten borradores), con fotografía JSON y su huella SHA-256; la hoja impresa sale de esa fotografía. Las señales del trabajo pasan a la factura como cobradas a cuenta.
- **Rectificativa** desde una emitida, con motivo y las líneas en negativo, que referencia a la original.
- **Cobros** con fecha, importe, medio y nota; parciales; se anulan con motivo (no se borran). **Señales** en la ficha del trabajo antes de la factura.
- **Albarán** de entrega A-AAAA-NNNN imprimible, que marca el trabajo como entregado.
- Pantallas: Facturas (con filtro de pendientes), ficha de la factura, hojas A4 de factura y albarán, bloque «Cobrar» en el trabajo. El Inicio añade cobros vencidos y presupuestos sin respuesta. Permiso `GestionarFacturas` (solo el propietario).
- **Criterios de aceptación:** no se edita una emitida (`tests/facturas.rs::una_emitida_no_se_edita_y_se_corrige_con_una_rectificativa`, además del disparador del esquema); la numeración no tiene huecos (`la_numeracion_no_tiene_huecos`); la rectificativa referencia a la original (mismo test); E2E `h6-facturas`: **cumplidos**.

**Decisiones:** el año de la factura es el de emisión, no el de creación del borrador. Emitir sin NIF del cliente pide confirmación (factura simplificada a un particular). Una factura cobrada de más no se bloquea: se ve en la ficha. El PDF automático en el servidor sigue aplazado (ADR-0003): se imprime o guarda como PDF desde el navegador. Sin cambio de esquema: las tablas se crearon en la 0.8.0.

**Pendiente:** Verifactu y factura electrónica entre empresas (fuera del MVP); el IVA y las menciones legales, a revisar con el gestor.

### H7. Copias de seguridad y despliegue (2026-11-22)
**Hecho**
- `dominio::copias`: cuándo toca copia (14:00, al arrancar si faltó, reintento a los 30 min tras un fallo, al cerrar salvo si es reciente) y qué copias se conservan (7 diarias y 4 semanales).
- Copia automática (ADR-0022): `VACUUM INTO`, comprobación, copia en el equipo y en cada disco con la carpeta `AlumFrame-copias`, comprobada de nuevo allí, y rotación. Programador en marcha mientras el programa atiende; copia al cerrar la ventana, con Ctrl+C o al parar el servicio (SIGTERM). Registro en `copias/automaticas/registro.json`.
- **Ajustes > Copias:** estado, «Copiar ahora», preparar un disco conectado y «Restaurar…» de cualquier copia del equipo o de un disco (pasa por la revisión de la importación). Aviso en el Inicio.
- **Migraciones (ADR-0021):** el esquema queda congelado como migración 0001; las bases de 0.8–0.10 se adoptan sin tocar sus datos (comprobado con una base creada por el binario anterior); copia previa antes de migrar; una base o una copia más nueva se rechaza; las copias antiguas se actualizan al restaurarlas.
- Servicio de Linux: puede escribir en los discos de `/mnt`, `/media` y `/run/media`; instrucciones para montar un disco en el servidor en `despliegue/linux/LEEME.txt`.
- **Criterios de aceptación:** restaurar una copia en una máquina limpia reproduce los datos (`tests/copias.rs::una_copia_del_disco_se_restaura_en_un_equipo_limpio`, E2E `h7-copias`): **cumplido**. Reiniciar la máquina deja el servicio funcionando: el instalador hace `systemctl enable --now` (ya desde H0; no probado en un reinicio real en esta sesión). En Windows se usa la aplicación de escritorio.

**Decisiones:** retención 7 + 4 (sin las 12 mensuales de BACKEND 8); el registro de copias va en un archivo y no en la base; disco reconocido por la carpeta, no por la letra; `user_version` en lugar de una tabla de versión; servicio de Windows aplazado.

**Pendiente:** copia fuera del taller (el PC descarga la del servidor), después del MVP. Probar en el servidor del propietario el disco montado y el reinicio.

