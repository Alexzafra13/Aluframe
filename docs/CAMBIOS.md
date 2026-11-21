# Registro de cambios (versión 1)

La documentación se mantiene en versión 1 hasta que el propietario declare la versión definitiva. Aquí se anotan los cambios relevantes, sin renumerar.

## Hasta 2026-11-19
- Stack: Rust completo (ADR-0001, aceptado).
- Criterios por requisito técnico y no por popularidad; interfaz generada en el servidor con HTMX, CSS propio y rusqlite (ADR-0002, aceptado).
- Diseño de referencia aprobado (`design/`).
- Nombres del código, variables de fórmulas, claves de datos y roles pasan al español sin tildes.
- `docs/BACKEND.md` (algoritmos, organización de archivos, reglas de estilo, concurrencia, copias y seguridad).
- Corrección de incoherencias: algoritmo de cortes unificado (Best Fit Decreasing) y estados de aprobación.
- Configuración de permisos de Claude Code y prompts de arranque.

## 2026-11-20
- Las series y la interfaz pasan a **cm** (un decimal de cm = 1 mm); por dentro se mantienen las décimas de mm. Redondeo provisional: a 0,1 cm hacia abajo (antes, 0,1 mm hacia arriba). El diseño de `design/` muestra las medidas en mm: hay que pasarlas a cm al escribir `docs/design.md`.
- `data/series/aluprom-21.yaml` convertido a cm. Fabricante: Aluval. Nueva tipología `corredera_4_hojas` en borrador (hoja de rodamiento `(ancho − 3) / 4`, el doble de piezas, 2 kits; ruedas y tornillos como supuesto).
- Nueva serie `data/series/sopena-v94.yaml`: ventana europea abatible de 2 hojas (Sopena V-94), cortes a 45°, en borrador.
- Fixtures: el de la corredera de 2 hojas pasa a cm (`aluprom21-corredera2-150x120.json`), con el kit de corredera que faltaba y `incompleto: true`; fixtures nuevos de la corredera de 4 hojas (303 × 120) y de la abatible (120 × 140). Todos `validado: false`.
- Decididos: las series se añaden como datos (no con código); recargo por color como porcentaje sobre el blanco; cada cliente tiene una o varias direcciones de montaje en su ficha.
- Correcciones de la revisión: `.gitignore` (excepción para las bases de prueba de migraciones, sin `web/dist/`, con salidas de Playwright), `MAINTENANCE.md` (Node solo para E2E), hook previo al push versionado y `just db-migrate` en H0, criterios de PDF alineados con el MVP (impresión A4; PDF automático tras ADR-0003 antes de H3), numeración sin huecos solo para facturas, ADR pendientes de zona horaria y servicio de Windows, y factura electrónica entre empresas junto a Verifactu.
- Nuevo: modos de ejecución local y servidor (`docs/BACKEND.md`, apartado 14); datos ficticios en `fixtures/demo/`; direcciones de montaje en la ficha del cliente y recargo por color en `SPEC.md`; lista ampliada de lo que queda fuera del MVP.
- Prototipo clicable de la pantalla de presupuesto con las tres tipologías (no forma parte del código final) y lista de funciones a decidir tras comparar con otros programas del sector (`OPEN_QUESTIONS.md`).
- Prioridad del MVP aclarada: clientes, presupuestos, trabajos asignados a operarios con marcado de pasos (en curso o hecho), orden de taller. Pedidos a proveedor, aplazados. Opciones de línea (lado de la cremona) en `SPEC.md`; el prototipo ya dibuja la cremona.
- Evaluador de fórmulas: se añade `si(condición, entonces, si_no)` para cantidades que dependen de la medida (bisagras de la abatible: 2 por hoja hasta 140 cm de alto, 3 por encima). El prototipo dibuja las bisagras y las suma a los accesorios.
- Nuevo `docs/DATOS.md`: modelo de datos en lenguaje llano y tabla de campos por hito. Pendiente `docs/FORMATO_SERIES.md` y `docs/PANTALLAS.md`.

## 2026-11-21
- Nuevo `docs/PANTALLAS.md`: organización de la interfaz por tareas (Vender, Taller, Cobrar), pantallas y rutas del propietario y del operario, el Inicio como tablero de trabajos con su panel de atención, permisos, reglas generales y pantallas por hito. Propone un menú de 8 entradas en lugar de 10.
- Nuevo `docs/FORMATO_SERIES.md`: formato de los archivos de series (cabecera, parámetros, referencias, piezas, vidrio, accesorios, opciones de línea, dibujo, límites), validaciones al importar y versiones.
- Las series (`aluprom-21.yaml`, `sopena-v94.yaml` y la demo) pasan al formato nuevo: `fabricante`, `proveedor`, tabla `referencias` (enlace con la tarifa), `dibujo` y, en la abatible, la opción `lado_cremona`. El vidrio de la demo usa el formato definitivo.
- `docs/DATOS.md`: `serie_referencia` sustituye a `serie_perfil`; `vidrio_pieza_serie`; `dibujo_json` y límites en `tipologia`; `cristal_recibido` en `ventana_trabajo`; `enviado_en` en `presupuesto`.
- Referencias cruzadas en `SPEC.md`, `CLAUDE.md`, `PROMPT_INICIAL.md`, `PROGRESS.md` y `OPEN_QUESTIONS.md`.
- Decisiones del propietario (2026-11-21): **ningún dato de cliente es obligatorio**; **las medidas de un trabajo aceptado solo las cambia el propietario** (revisión con motivo, el operario nunca); el operario ve una lista de tarjetas y abre una ventana a pantalla completa; los descuentos que faltan los dará más adelante. Nueva tabla `cambio_ventana_trabajo` y apartado de modificación de trabajos en `docs/BACKEND.md`.
- Paquete para empezar en Claude Code: `README.md` con los pasos de arranque y `design/prototipos/` con el prototipo de dibujo como referencia.

## 2026-11-21 (inicio de H0)
- Decisiones del propietario: el **servidor** del taller es **Linux** (servicio systemd) y el **PC de la oficina** es **Windows** (navegador, con acceso directo en modo aplicación). El ejecutable de Windows se usa en modo local para probar o como plan de emergencia.
- Puerto por defecto: **9090**.
- Base de datos: **SQLite** (se confirma ADR-0002; PostgreSQL descartado).
- Primer usuario: **asistente de primer arranque** (solo aparece si no hay usuarios) y comando `alumframe restablecer-contrasena` para recuperar el acceso.
- Plantillas: **Askama** (ADR-0004; el número 0003 queda reservado para el PDF).
- Ya no hace falta el ADR del envoltorio de servicio de Windows.
- H0 construido: ver `docs/PROGRESS.md`. Nuevos ADR-0004 a 0008, `docs/design.md` y `docs/MAINTENANCE.md` (secciones de compilación, publicación, instalación y dependencias).
- `docs/DATOS.md`: tabla `bloqueo_ip` (ip, intentos_fallidos, bloqueado_hasta) para el bloqueo por IP; `sesion.id` guarda la huella SHA-256 del token.
- Decisión del propietario: al entrar, el propietario puede marcar **«Mantener la sesión abierta hasta cerrar el navegador»** (desmarcada por defecto). Con ella la sesión no caduca por inactividad y dura hasta cerrar el navegador o apagar el PC, con un tope de 30 días. Sin ella: 2 h de inactividad y 12 h como máximo. El operario no tiene esta opción. Migración 0002 (`sesion.hasta_cerrar_navegador`).
- Bisagras de la abatible confirmadas por el propietario: 2 por hoja hasta 140 cm de alto (incluido) y 3 por encima. `fixtures/real/sopena-v94-abatible2-120x140.json` añade «BISAGRA: 4» a lo esperado, con su permiso.
- Lectura de series con `yaml-rust2` y series guardadas en la base de datos como el texto YAML de cada versión (decisión delegada en Claude; ADR en H1).
- H1 construido: ver `docs/PROGRESS.md`. ADR-0009 y 0010. `docs/DATOS.md` (tablas de series simplificadas) y `docs/FORMATO_SERIES.md` (funciones de redondeo y campos desconocidos) actualizados. Variable `ALUMFRAME_DATOS_DEMO=si` para aceptar series ficticias solo en pruebas.
- Decisión del propietario: **sin migraciones hasta el primer despliegue real**. Las migraciones 0001 a 0003 se sustituyen por un esquema único, `esquema.sql`, con su huella en la tabla `esquema` (ADR-0011). `CLAUDE.md`, `DATOS.md`, `BACKEND.md` y `MAINTENANCE.md` actualizados. La orden `migrar` pasa a ser `preparar-base`.
- Decisión del propietario: movimiento inspirado en iOS sin perder rendimiento ni el diseño aprobado (colores y estructura iguales). Transiciones entre pantallas, indicador del menú que se desliza, respuesta al pulsar, barra superior translúcida y PIN como el del iPhone con entrada automática. Detalle en `docs/design.md`.
- Datos del propietario: barras de 6,40 m en la V-94 y grosor de sierra de 2 mm (provisional). Nueva pantalla Ajustes > Taller y tabla `configuracion`.
- Documentación preparada para retomar en una sesión nueva: `docs/PROGRESS.md` («Cómo retomar») y `docs/PROMPT_INICIAL.md` (mensaje para H2).
- **H2 (plan aprobado):** importación de tarifas y clientes solo desde CSV (Excel, guardado como CSV), todo o nada, versiones completas de tarifa y deshacer la última de cada proveedor (ADR-0012). `proveedor` gana `descuento_habitual_bp` (DATOS 3.3 lo nombraba y la tabla no lo tenía). `kit_componente` pasa a H5 y `acabado_recargo_proveedor` espera a la decisión 4 de DATOS.
- **H2 terminado:** clientes (búsqueda, direcciones, RGPD, importación CSV con deshacer), catálogo (proveedores, acabados, artículos), tarifas versionadas (importación con vista previa, edición, ajuste, histórico y deshacer). `DATOS.md` 7.3 y 7.4 recogen las columnas añadidas (`busqueda`, `importacion_id`, tabla `importacion`). Rutas de edición del catálogo en `PANTALLAS.md`.
- El propietario confirma que las barras de Aluval vienen en 640 cm; coincide con `data/series/aluprom-21.yaml` y no cambia nada del código.
- Retales (decisión del propietario): se registran al instalar el programa y el plan de corte los usa antes que las barras nuevas (`SPEC.md` 9 y H5, `BACKEND.md` 5.3). Otras longitudes de barra se configurarán en su serie cuando aparezcan. Corregido `BACKEND.md`, que daba por hecho que el optimizador ya admitía barras de distinta longitud.
- Retal mínimo 0 (cualquier sobrante se guarda) y registro de retales al instalar opcional, por decisión del propietario.
- **H3 (plan aprobado):** ADR-0003 (PDF desde el navegador hasta H6) y ADR-0013 (`serde_json`). Medida de la ventana como entrada. Margen, modo de cobro de perfiles y mano de obra configurables en Ajustes > Presupuestos, sin valores inventados: sin margen ni modo elegido, las líneas no tienen precio final. IVA 21 % general y 10 % reducido configurables (cuándo aplica el reducido, con el gestor). Numeración `P-AAAA-NNNN`. Interfaz con hojas deslizantes, control segmentado y avisos efímeros al estilo iOS (`docs/design.md`).
- **H3 terminado:** presupuestos con versiones, editor de ventana con dibujo y cálculo en vivo, precio y totales en el dominio, aceptación con copia congelada y hoja A4. Interfaz con movimiento estilo iOS en todo el programa (avisos efímeros, hojas, control segmentado). `DATOS.md` 7.6 y `PANTALLAS.md` puestos al día.

