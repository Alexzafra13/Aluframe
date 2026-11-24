# Mapa de pantallas, rutas y permisos (versión 1, borrador)

Este documento dice qué pantallas tiene AlumFrame, cómo se organizan, qué se ve en cada una y quién puede entrar. Es la base de la interfaz del MVP. El aspecto (colores, tipografía, tamaños) está en `design/` y se documentará en `docs/design.md` en H0; aquí se decide **la organización**, no el estilo.

Los diseños aprobados (`design/01-presupuesto.png`, `02-inicio.png`, `03-hoja-taller.png`) muestran las medidas en mm: al construir se pasan a **cm con un decimal**, como las series.

## 1. Cómo está pensada la organización
1. **Por tareas, no por tablas.** El menú se agrupa por lo que hace el dueño: **Vender**, **Taller** y **Cobrar**, tal como en el diseño aprobado. Nadie tiene que saber qué es una "entidad".
2. **El Inicio responde a "¿qué hay que hacer hoy?"**: un tablero de trabajos (por cortar, en montaje, listos para entregar) y un panel con lo que requiere atención (avisos del taller, stock bajo, presupuestos sin respuesta, cobros vencidos).
3. **Cada documento enseña el siguiente paso.** Presupuesto → Trabajo → Orden de taller → Factura, siempre con la misma barra de pasos arriba y un único botón principal.
4. **El operario tiene otra pantalla, no una versión recortada.** Empieza en "Mi día": cabecera de una línea, botones grandes (Nuevo apunte, Apuntes con su contador, Clientes y, con permiso, Presupuestos y Facturas), sus tareas y sus ventanas, grandes y táctiles, sin precios. Además toma apuntes, ve las tareas que le manda el dueño (ADR-0025), busca clientes y, con permiso del dueño, ve presupuestos, facturas o todo el taller (ADR-0024), nunca costes.
5. **Un botón desactivado siempre dice por qué** ("No se puede aceptar: la línea 2 no tiene medida de cristal").
6. **Lo irreversible pide confirmación con su consecuencia** (aceptar un presupuesto, terminar una orden, emitir una factura). Lo reversible no pregunta: se deshace.

### Cambio propuesto respecto al menú del diseño
El diseño tiene 10 entradas. Propongo **8 para el MVP** y se decide al construir H0:
- **Órdenes de taller** deja de ser una entrada: la orden es una pestaña dentro del Trabajo (son lo mismo para el dueño).
- **Cobros** pasa a ser una pestaña de Facturas.
- Se añade **Ajustes** al pie del menú.

| Grupo | Entradas |
|---|---|
| Vender | Inicio, Clientes, Presupuestos, Trabajos |
| Taller | Stock, Catálogo |
| Cobrar | Facturas |
| (pie) | Ajustes, Salir |

## 2. Barra superior (siempre visible)
- **Búsqueda global** (`Ctrl K`): cliente, presupuesto, trabajo, factura o referencia. Salta a la ficha.
- **+ Nuevo**: Cliente, Presupuesto, Entrada de stock.
- Usuario y estado de la última copia de seguridad (se pone en rojo si falla).

## 3. Pantallas del propietario
| Pantalla | Ruta | Hito | Qué hace |
|---|---|---|---|
| Inicio | `/` | H4 | Tablero y panel de atención (apartado 4) |
| Clientes | `/clientes` | H2 | Lista con búsqueda; botón Nuevo cliente |
| Cliente (alta y edición) | `/clientes/nuevo`, `/clientes/{id}/editar` | H2 | Formulario (apartado 5) |
| Ficha de cliente | `/clientes/{id}` | H2 | Datos, direcciones de montaje, pestañas Presupuestos, Trabajos y Facturas; botón **Nuevo presupuesto** |
| Presupuestos | `/presupuestos` | H3 | Lista con filtros por estado (borrador, enviado, aceptado, rechazado, caducado y archivado; ADR-0027). Sin filtro, todos menos los archivados |
| Presupuesto | `/presupuestos/{id}` | H3 | Cabecera, líneas, totales y acciones (apartado 6) |
| Línea de ventana | `/presupuestos/{id}/lineas/{n}` | H3 | Editor con dibujo y cálculo en vivo (apartado 7) |
| Línea libre | hoja en `/presupuestos/{id}` | H3 | Descripción, cantidad, precio unitario, descuento e IVA |
| Nueva línea de ventana | `/presupuestos/{id}/lineas/nueva` | H3 | El mismo editor, vacío |
| Ajustes de empresa y precio | `/ajustes/presupuestos` | H3 | Datos de la empresa, margen, cobro de perfiles, mano de obra, IVA, validez y condiciones |
| Presupuesto imprimible | `/presupuestos/{id}/imprimir` | H3 | A4 con dibujos y condiciones; sin costes ni márgenes |
| Trabajos | `/trabajos` | H4 | El mismo tablero del Inicio con vista de lista y filtro por operario |
| Trabajo | `/trabajos/{id}` | H4 | Pestañas Resumen, Orden de taller, Material y Entrega (apartado 8) |
| Orden de taller | `/trabajos/{id}/orden` y `/trabajos/{id}/orden/imprimir` | H4 | Una hoja A4 por ventana, como `design/03-hoja-taller.png` |
| Stock | `/stock` | H5 | Saldos, mínimos, lista de compra sugerida (apartado 9) |
| Artículo de stock | `/stock/{articulo}` | H5 | Movimientos y retales |
| Entrada o ajuste de stock | `/stock/movimientos/nuevo` | H5 | Artículo, color, cantidad, longitud si es un retal y motivo. Sirve también para el inventario inicial de barras y retales al instalar |
| Catálogo | `/catalogo` | H2 | Pestañas Artículos, Proveedores, Acabados, Tarifas y Series. Fichas en `/catalogo/articulos/{id}`, `/catalogo/proveedores/{id}`, `/catalogo/acabados/{id}`; versiones en `/catalogo/tarifas/{id}` |
| Importar clientes | `/clientes/importar` | H2 | Asistente en 3 pasos con deshacer |
| Importar tarifa | `/catalogo/tarifas/importar` | H2 | Asistente en 3 pasos (apartado 10) |
| Importar serie | `/catalogo/series/importar` | H1 | El mismo asistente, con las comprobaciones de `docs/FORMATO_SERIES.md` |
| Facturas | `/facturas` | H6 | Lista; pestañas Facturas y Cobros pendientes |
| Factura | `/facturas/{id}` | H6 | Borrador editable, emisión, cobros, rectificativa (apartado 11) |
| Factura imprimible | `/facturas/{id}/imprimir` | H6 | A4 |
| Albarán imprimible | `/albaranes/{id}/imprimir` | H6 | A4; se crea desde el trabajo y lo marca entregado |
| Ajustes | `/ajustes` | H0 a H7 | Empresa, usuarios, taller, copias, registro de actividad (apartado 12) |
| Demostración | `POST /demostracion/entrar`, `/demostracion/salir`, `/demostracion/empezar-de-nuevo` | 0.18.0 | Entrar desde la pantalla de entrada o Ajustes > Datos; franja con «Salir» y «Empezar de nuevo» en todas sus pantallas; su entrada enseña los usuarios y claves de prueba (ADR-0028) |
| Estado | `/health` | H0 | Comprobación técnica |

## 4. Inicio
Sigue `design/02-inicio.png`.
- **Cabecera:** "Hoy en el taller", botones **Nuevo cliente** y **Nuevo presupuesto**.
- **Tablero** con tres columnas: **Por cortar**, **En montaje** y **Listos para entregar**. Cada tarjeta es un trabajo: número, cliente, miniatura, tipología y medida (si tiene varias ventanas, "3 ventanas" y una barra de progreso), operario asignado y una etiqueta de estado.
- **Panel derecho:** *Avisos del taller* (las notas "falta algo" que dejan los operarios), *Stock bajo*, *Presupuestos sin respuesta* (con los días desde que se enviaron) y *Cobros vencidos*. Cada línea abre su pantalla.
- **Vacío:** cada columna y cada bloque explican qué falta ("No hay trabajos por cortar. Cuando aceptes un presupuesto aparecerá aquí").

**En qué columna cae un trabajo** (se calcula, no se mueve a mano):
| Columna | Condición |
|---|---|
| Por cortar | Ningún paso de la primera ventana empezado |
| En montaje | Algún paso empezado y la orden sin terminar |
| Listos para entregar | Orden terminada y entrega pendiente |

**Etiquetas** (se muestra la primera que aplique, con un contador si hay más):
| Etiqueta | Cuándo |
|---|---|
| Sin asignar | El trabajo no tiene operario |
| Falta algo | Un paso tiene una nota de aviso |
| Falta el cristal | Alguna ventana no tiene marcado "cristal recibido" |
| Falta material | El saldo menos las reservas no cubre lo que pide la orden |
| Material listo | Hay stock para todo |
| Sin albarán | Terminado y sin albarán |

## 5. Alta de cliente
Un formulario corto, en este orden:
1. **Tipo:** particular o empresa (cambia las etiquetas y el IVA).
2. **Nombre o razón social**, **teléfono**, **NIF o CIF** (si se escribe, se valida mientras se teclea), y **correo**. La forma de pago se elige al registrar cada cobro (apartado 11).
3. **Dirección fiscal** y una casilla "la dirección de montaje es la misma".
4. **Direcciones de montaje:** las que haga falta; **+ Añadir otra** con etiqueta ("Casa", "Obra Calle X"), persona de contacto y notas ("sin ascensor").
5. Guardar y elegir: **Guardar** o **Guardar y crear presupuesto**.

**Obligatorios: nombre y teléfono** (decisión del propietario, 2026-11-22), para no tener fichas sin forma de contacto. El teléfono español se escribe con sus 9 cifras y se guarda como «612 345 678»; el +34 se da por puesto si el país es España, que viene escrito por defecto. Un número extranjero empieza por su prefijo. Los clientes importados sin nombre se muestran como **"Cliente sin nombre (n.º 12)"**. Solo se avisa (sin bloquear) cuando se emite una factura (apartado 11).

## 6. Presupuesto
- **Cabecera:** número (`P-2026-0001`), estado, cliente y **dirección de montaje** (selector con "+ Añadir otra"), validez y condiciones.
- **Barra de pasos:** Presupuesto, Trabajo, Orden de taller, Factura; el paso actual resaltado.
- **Líneas:** cada una es una tarjeta con miniatura del dibujo, título corto con el tipo y las medidas ("Ventana corredera 2 hojas · 150 × 120 cm") y **«Ver detalles»** desplegable con la serie, el color y las opciones (desde la 0.20.1; igual en la ficha del trabajo), cantidad, precio y una etiqueta **Completa** o **Incompleta** con el motivo en palabras ("Parteluces: falta la medida de corte"). Botones **+ Ventana o mosquitera** y **+ Otro concepto**.
- **Pie fijo:** perfiles, cristal, accesorios, mano de obra, descuento, base, IVA y **total**.
- **Acciones:** *Guardar*, *Imprimir*, *Marcar como enviado*, *Nueva versión*, *Rechazar* y el botón principal **Aceptar y crear trabajo**. Este último está desactivado con su motivo mientras haya líneas incompletas o límites sin revisar. Al pulsarlo se pide confirmación: "Se creará el trabajo T-2026-0001 con estos datos congelados. Cambios posteriores en las series o en las tarifas no lo alteran."
- **Versiones:** una nueva versión copia las líneas; la anterior queda visible y solo lectura.

## 7. Editor de línea de ventana
Sigue `design/01-presupuesto.png`.
- **Izquierda:** el dibujo con pestañas **Cotas**, **Piezas** y **Cristal**. El dibujo se genera con los datos de la serie (`docs/FORMATO_SERIES.md`, apartado 9) y cambia al instante con la medida.
- **Derecha, arriba:** *Serie* y luego *Tipología* (dos selectores en cascada), **Ancho**, **Alto**, **Vidrio**, **Color** y las **opciones de la tipología** (por ejemplo, "Hoja de la cremona: izquierda o derecha").
- **Cabecera del dibujo (0.19.0, ADR-0029):** el tipo ("Ventana corredera 4 hojas") y debajo la serie con el logotipo de su fabricante. El desplegable dice "Aluprom 21 (Aluval) · Ventana corredera 2 hojas".
- **Opciones dependientes (0.20.0, ADR-0030):** una opción con `solo_si` aparece y desaparece en el navegador (`recursos/js/opciones.js`) sin recargar: "Parteluces: Sí" muestra "Abajo" y "Medida de abajo (cm)". Al cambiar de serie, el servidor vuelve a pintar las opciones (0.21.3). Una tipología sin cristal (mosquitera) oculta "Vidrio", la capa "Cristal" y su importe.
- **Derecha, abajo, por apartados:** **Perfiles a cortar** numerados (como en la hoja de corte), con la silueta del perfil, cantidad, longitud y ángulo; una pieza sin medida sale con "pendiente". **Chapas**, con su dibujo técnico a escala y cotas. **Herrajes** (cierres, kits, ruedas, tornillos), con su dibujo y, debajo, el modelo del artículo del catálogo. La hoja de taller y "Mi día" siguen los mismos apartados.
- **Pie:** perfiles, cristal, accesorios, mano de obra y total, más una etiqueta que explica lo que falta ("Falta el cristal").
- **Validaciones visibles:** medida fuera de límites ("Este perfil no admite más de X cm: revísalo"), pieza más larga que la barra, longitud negativa. Los avisos no borran lo escrito.
- **Cálculo en vivo:** el servidor recalcula al escribir (objetivo: menos de 100 ms). Con `CONFIRMAR` el cálculo sigue, pero la línea sale incompleta.
- **Guardar línea** y **Guardar y añadir otra**.

## 8. Trabajo
- **Cabecera:** número, cliente, dirección de montaje, estado y la barra de pasos.
- **Resumen:** las ventanas con su miniatura, el **operario** (selector, para todo el trabajo o por ventana), la **fecha prevista**, una barra de progreso y las notas de aviso.
- **Orden de taller:** vista en pantalla de las hojas (una por ventana) y botón **Imprimir**.
- **Material:** lo que necesita el trabajo, lo que hay en stock y el plan de corte por barra; indica si falta algo.
- **Entrega:** casilla **cristal recibido** por ventana, albarán y factura.
- **Acciones:** *Asignar*, *Imprimir orden*, **Terminar orden y descontar stock** (confirmación: "Se descontarán X barras, 1 kit… Esto no se puede deshacer, se corrige con un ajuste"), *Crear albarán* y *Crear factura*.
- **Cambiar las medidas lo hace quien lleva el trabajo (el propietario), nunca el operario.** En cada ventana hay un botón **Modificar medidas**, que abre un formulario con ancho, alto y las opciones de la tipología:
  - Recalcula con la **misma versión de la serie y de la tarifa** con la que se aceptó, y muestra lo que cambia: piezas antes y después, plan de corte y diferencia de precio.
  - Si la ventana ya tiene pasos hechos, avisa y ofrece una casilla **Reiniciar los pasos**.
  - Pide un **motivo** (una línea).
  - Si la **orden está terminada** (el stock ya se descontó) no se puede modificar: se anula el trabajo y se crea otro, o se registra un ajuste de stock.
  - El presupuesto original **no cambia**: queda como se aceptó. El trabajo conserva todas las revisiones, y la factura se crea con el importe vigente.
  - La orden de taller lleva **"Revisión n" y la fecha**, para no fabricar con una hoja antigua. El operario ve el aviso **"Medida cambiada"** en esa ventana hasta que la abre.

## 9. Stock
- **Lista:** artículo, color, saldo, reservado, mínimo y etiqueta (**Bajo mínimo**). Filtro "solo bajo mínimo" y búsqueda.
- **Lista de compra sugerida:** lo que está bajo mínimo más lo que necesitan los trabajos pendientes. Es solo una lista para leer o imprimir; **no genera pedidos** en el MVP.
- **Artículo:** sus movimientos (entrada, salida, ajuste, con quién y cuándo) y los **retales** (barras más cortas con su longitud).
- **Entrada o ajuste:** nunca se edita el saldo directamente: siempre se crea un movimiento con motivo.

## 10. Catálogo e importaciones
Pestañas: **Artículos**, **Proveedores**, **Acabados** (color y recargo en % sobre el blanco), **Tarifas** y **Series**.

Tanto las tarifas como las series se importan con el mismo asistente de **3 pasos**:
1. **Elegir** proveedor (o serie) y archivo.
2. **Vista previa**: qué es nuevo, qué cambia (con el precio anterior y el nuevo), cuántos avisos y errores, cada uno explicado en español. Nada se guarda.
3. **Confirmar**. Después aparece **Deshacer esta importación** durante todo el tiempo que no se haya usado la versión nueva en un presupuesto.

La lista de **Series** muestra su estado (borrador, validada), sus versiones y cuántos `CONFIRMAR` quedan. **Validar** una serie es una acción explícita del propietario.

## 11. Facturas
- **Lista:** número, cliente, total, estado (borrador, emitida, parcial, pagada, vencida). La pestaña **Cobros pendientes** agrupa por cliente.
- **Borrador:** editable y sin número.
- **Emitir:** confirmación con la consecuencia: "Recibirá el número F-2026-0001 y ya no se podrá modificar. Una corrección se hará con una rectificativa." Si faltan datos que una factura completa suele exigir (nombre y NIF del cliente), **avisa y lista lo que falta, pero deja emitir** tras una confirmación explícita. Hay que confirmar con el gestor qué casos admiten factura simplificada.
- **Emitida:** solo lectura. **Registrar cobro** (fecha, importe —por defecto lo pendiente— y medio: efectivo, Bizum, tarjeta, transferencia u otro; se admiten cobros parciales) y **Crear rectificativa**. El estado de pago se calcula con los cobros.

## 12. Ajustes
- **Empresa:** datos fiscales, logotipo, IVA por defecto, formato de numeración, condiciones y validez por defecto del presupuesto.
- **Usuarios:** altas de operarios (nombre y PIN de 6 dígitos), restablecer PIN, desactivar.
- **Taller:** grosor de sierra, recorte, retal mínimo, cómo se reparten las piezas (mejor aprovechamiento o rápido), medidas habituales por serie y tipología (ADR-0026) y lista de pasos por defecto de una ventana.
- **Datos** (desde 0.5.0, ADR-0016): exportar todos los datos a un archivo `.alumframe` e importar uno (sustituye los datos tras revisar qué contiene y guardar una copia de lo anterior). Base de la restauración guiada de H7.
- **Copias de seguridad:** estado de la última, descarga manual, restauración guiada.
- **Registro de actividad:** quién hizo qué y cuándo (solo lectura).

## 13. Pantallas del operario
El operario entra con su nombre y su **PIN en un teclado numérico grande**. Está pensado para **tablet** y se usa también en móvil. Texto de 16 px o más y botones de al menos 44 px.

| Pantalla | Ruta | Hito | Qué hace |
|---|---|---|---|
| Entrar | `/entrar` | H0 | Tarjetas con los nombres de los operarios y teclado de PIN |
| Mi día | `/mi-dia` | H4 | Tarjetas de las ventanas que tiene asignadas |
| Ventana | `/mi-dia/{ventana}` | H4 | El dibujo grande, la lista de pasos y la hoja de corte |

**Mi día:** una tarjeta por ventana asignada (miniatura, cliente, tipología y medida, progreso "3 de 6", etiquetas **Falta algo** o **Medida cambiada** si las hay), ordenadas por fecha prevista, con varias a la vista a la vez. Al tocar una se abre **esa ventana a pantalla completa**, con botones **Anterior** y **Siguiente** para pasar a otra ventana del mismo trabajo sin volver a la lista. Sin precios ni totales.

**Ventana:**
- **Arriba:** el dibujo con las piezas numeradas y las opciones elegidas (por ejemplo, "Cremona en la hoja derecha").
- **Pasos:** lista de pasos (propuesta: cortar, montar marco, montar hojas, acristalar, accesorios y cierre, revisar). Cada paso tiene un botón claro: **Empezar** (pasa a *en curso*) y **Terminar** (pasa a *hecho*). Se puede pulsar **Terminar** sin haber pulsado **Empezar**. **Pausar** y **Deshacer** devuelven el paso a su estado anterior.
- **Falta algo:** abre una nota con motivos rápidos ("falta cristal", "falta material", "medida mal", "otro" con texto). El aviso llega al Inicio del propietario y la ventana queda marcada.
- **Hoja de corte** (pestañas): Piezas (medida, cantidad y ángulo), Cristal, Accesorios y Barras (plan de corte).
- Cada cambio guarda quién y cuándo. Si dos personas tocan el mismo paso, la segunda ve un aviso y recarga.
- El operario **no ve** precios, márgenes, facturas ni otros operarios.

## 14. Permisos
| Pantalla o acción | Propietario | Operario |
|---|---|---|
| Inicio, Clientes, Presupuestos, Catálogo, Facturas, Ajustes | Sí | No |
| Trabajos (ver, asignar, terminar la orden) | Sí | No |
| Orden de taller (ver e imprimir) | Sí | Solo las suyas, sin precios |
| Mi día y Ventana | Sí (lectura) | Sí |
| Marcar pasos y avisos | Sí | Solo en sus ventanas |
| Modificar medidas de una ventana de un trabajo | Sí | Nunca |
| Stock (ver) | Sí | No |
| Stock (movimientos y ajustes) | Sí | No |
| Costes, márgenes y precios | Sí | Nunca |

Los permisos se comprueban en la capa de aplicación (`docs/BACKEND.md`, apartado 9), no solo ocultando botones; las rutas del operario devuelven 404 para lo que no es suyo.

## 15. Reglas generales de interfaz
- **Idioma y formato:** español claro y sin siglas; fechas `dd/mm/aaaa`; importes `1.234,56 €`; medidas en cm con un decimal.
- **Errores y avisos:** dicen qué pasó y qué hacer, junto al campo, sin borrar lo escrito. Un error interno muestra su código de correlación.
- **Conflictos de edición:** "Otra persona ha cambiado esto. Recarga para ver sus cambios", sin perder lo que se estaba escribiendo.
- **Estados vacíos:** cada lista vacía explica el primer paso.
- **Teclado y accesibilidad:** todo se puede hacer con teclado, contraste suficiente, nada que dependa solo del color (las etiquetas llevan texto).
- **Impresión:** presupuesto, orden de taller y factura son páginas HTML con CSS de impresión A4; la hoja del taller es el documento clave.
- **Pantallas:** el propietario trabaja en ordenador (1280 px o más) y el operario en tablet (10") o móvil; el propietario también puede usar la tablet.
- **Sin ventanas emergentes enormes:** las confirmaciones son pequeñas y explican la consecuencia.

## 16. Pantallas por hito
| Hito | Pantallas |
|---|---|
| H0 | Entrar, esqueleto con menú y barra superior, Ajustes (usuarios) y `/health` |
| H1 | Importar serie (sin editor todavía) |
| H2 | Clientes (lista, alta, ficha), Catálogo (artículos, proveedores, acabados, tarifas) e importar tarifa |
| H3 | Presupuestos (lista, presupuesto, editor de línea, imprimible) |
| H4 | Inicio con el tablero, Trabajos, Trabajo, Orden de taller, Mi día y Ventana |
| H5 | Stock y movimientos |
| H6 | Facturas, factura e imprimible |
| H7 | Copias de seguridad en Ajustes |

## 17. Objetivos de usabilidad (para probar con el dueño)
- Crear un cliente (con o sin todos sus datos), sin ayuda, en menos de 2 minutos.
- Presupuestar una ventana de una serie ya cargada en menos de 2 minutos.
- Un operario que nunca lo ha visto marca un paso y deja un aviso sin explicación.
- Todo estado desactivado o incompleto explica la causa en la misma pantalla.

## 18. Pendiente de decidir
1. Aprobar el menú de 8 entradas o mantener las 10 del diseño.
2. Lista de pasos por defecto de una ventana y si cambia según la tipología.
3. ~~Una ventana por pantalla o todas en una lista~~: resuelto, lista de tarjetas y una ventana a pantalla completa al abrirla.
4. ~~Cómo se modifica un trabajo ya aceptado~~: resuelto, lo hace el propietario con **Modificar medidas** (apartado 8).
5. Cuándo se descuenta el stock: al terminar la orden (como está) o al marcar el paso de corte.
6. Formato visible de la numeración (`P-2026-0001`) y las letras de cada serie.
7. Si el cristal recibido se marca a mano (propuesto) o con un pedido de vidrio, que queda fuera del MVP.
8. Qué más debe aparecer en el panel de atención del Inicio.
9. ~~Pantallas del propietario en el móvil~~: hecho en 0.5.1 (barra de pestañas abajo con «Más», tablas como celdas, editor a una columna; `docs/design.md`, apartado «Móvil y tablet»).
