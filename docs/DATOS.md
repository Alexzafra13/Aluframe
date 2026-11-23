# Modelo de datos (versión 1, borrador)

Este documento explica qué guarda AlumFrame y cómo se relaciona cada cosa. Las secciones 1 a 6 están escritas para entenderlas sin ser desarrollador. La sección 7 es la tabla de campos que usará Claude Code. Si algo choca con `SPEC.md` o `docs/BACKEND.md`, se corrige aquí o allí en el mismo cambio.

## 1. La idea en una imagen
Imagina un archivador con cajones. Cada cajón es una lista (clientes, presupuestos, artículos…) y cada ficha es una línea de esa lista. Lo importante no es cuántos cajones hay, sino **qué se puede cambiar y qué no**. Hay seis grupos:

| Grupo | Qué contiene | ¿Se puede editar? |
|---|---|---|
| **Personas y acceso** | Usuarios (propietario y operarios), sesiones | Sí |
| **Clientes** | Datos fiscales y varias direcciones de montaje | Sí, y se archivan en vez de borrarse |
| **Catálogo** | Proveedores, artículos, acabados con su recargo, tarifas | Sí, pero cada tarifa nueva es una **versión** y las usadas no se tocan |
| **Recetas** | Series de perfiles: cómo se calcula cada ventana | Igual: se crea una versión nueva, la antigua se conserva |
| **Documentos y trabajo** | Presupuestos, trabajos, órdenes de taller, albaranes, facturas, cobros | Los borradores sí; lo emitido o aceptado queda **congelado** |
| **Libro de stock y sistema** | Movimientos de stock, numeración, auditoría, copias | Solo se **añade**, nunca se reescribe |

## 2. Seis reglas que lo hacen fiable
1. **Números enteros.** El dinero va en céntimos y las medidas en décimas de milímetro, para que no haya errores de redondeo.
2. **No se borra lo que se ha usado.** Se archiva. Una factura mal hecha se corrige con otra rectificativa.
3. **Lo aceptado o emitido es una fotografía.** Al aceptar un presupuesto, al terminar una orden de taller y al emitir una factura se guarda una copia de todo lo que contaba entonces (medidas, fórmulas, precios, versión de la serie y de la tarifa). En 2030 se puede reimprimir igual aunque hayan cambiado precios o fórmulas.
4. **El stock es un extracto, no un número editable.** No existe "quedan 12 barras" escrito a mano: hay apuntes (entró, salió, ajuste) y el saldo es su suma.
5. **Todo cambio importante deja quién y cuándo**, en un registro que solo admite añadir.
6. **Si dos personas editan a la vez**, la segunda recibe un aviso y recarga; nadie pisa el trabajo de otro sin enterarse.

## 3. Cada grupo en palabras

### 3.1 Personas y acceso
Hay dos tipos de usuario. El **propietario** lo ve y lo cambia todo. El **operario** lo da de alta el propietario con un usuario y un PIN: ve los trabajos que se le asignan y busca clientes, y el propietario puede darle permisos extra (ADR-0024). Nunca ve costes ni márgenes.

### 3.2 Clientes
Una ficha de cliente puede guardar los datos fiscales (nombre o razón social, NIF o CIF, dirección fiscal), el contacto, si es **particular o empresa** (cambia el IVA) y la forma de pago habitual. Al darlo de alta a mano son **obligatorios el nombre y un teléfono** (en España, 9 cifras sin prefijo; el +34 se da por puesto); el resto se completa después y el NIF, si se escribe, se valida. La importación por CSV no exige nada. Cada cliente puede tener **una o varias direcciones de montaje**, con persona de contacto y notas ("sin ascensor"). El presupuesto elige una o añade otra al momento. Cuando se emite un documento, los datos del cliente se **copian** dentro de él, así que cambiar la ficha después no altera facturas antiguas.

### 3.3 Catálogo
- **Proveedor:** quién vende, condiciones de pago y descuento habitual.
- **Artículo:** una referencia de un proveedor (un perfil, un vidrio, un herraje, un accesorio, un kit). Guarda la unidad en que se vende (barra, metro, m², unidad, caja), la longitud de barra (640 cm por defecto; las barras más cortas se registran en el stock, no creando artículos nuevos) y el stock mínimo.
- **Acabado (color):** blanco, lacados, etc. El **blanco es el precio base** y los demás llevan un **recargo en %** sobre él. El color no crea artículos distintos: el mismo perfil se compra en varios colores y el stock se lleva por perfil y color.
- **Kit:** un artículo formado por otros con sus cantidades (por ejemplo, el kit de corredera). En el presupuesto se ve como una línea; en el stock se descuentan sus componentes.
- **Tarifa:** la lista de precios de un proveedor con su fecha, su descuento de compra y el archivo de origen. Importar una tarifa nueva crea una versión nueva y conserva las anteriores, así que siempre se sabe a qué precio se calculó cada presupuesto.
- **Consumibles con presentación:** los tornillos se compran por caja y se gastan por unidad; el artículo guarda cuántas unidades trae la caja y el stock se lleva en unidades.

### 3.4 Recetas (series de perfiles)
Una **serie** (por ejemplo Aluprom 21 o Sopena V-94) tiene **versiones**. Cada versión tiene **tipologías** (corredera de 2 hojas, de 4 hojas, abatible…) y cada tipología lista:
- las **piezas** con su cantidad, la fórmula de su longitud y el ángulo de corte;
- el **vidrio** (tipo, fórmulas de ancho y alto, cantidad);
- los **accesorios** con su cantidad, que puede depender de la medida (las bisagras);
- las **opciones de línea** que se eligen al presupuestar (el lado de la cremona);
- los **límites** de medida que admite el perfil (para avisar si se pasa).

Cada versión se guarda como **borrador**, **validada** o **demo**. Un borrador se puede calcular, pero el resultado sale **incompleto** y no permite precio final. Cuando una versión se usa en un presupuesto ya no se edita: se crea la siguiente.

### 3.5 Presupuestos
Un **presupuesto** pertenece a un cliente (y a una dirección de montaje) y tiene **versiones**. Cada versión tiene **líneas**: una **ventana** (serie, tipología, ancho, alto, acabado, tipo de vidrio y opciones) o una **línea libre** (cerrajería, instalación, material suelto). Al calcular una línea de ventana se guarda también el **resultado** (piezas, vidrio, accesorios, fórmulas usadas) y si está incompleta. Un presupuesto con líneas incompletas no se puede aceptar. Los presupuestos se numeran al crearse; si se descarta un borrador, queda un hueco, y no pasa nada.

### 3.6 Trabajos y orden de taller
Al **aceptar** un presupuesto nace un **trabajo** con una **copia congelada** de todo lo calculado. Cada ventana del trabajo lleva su lista de **pasos** y el propietario puede **asignar** el trabajo o cada ventana a un operario. Un paso está `pendiente`, `en curso` o `hecho`, puede llevar una nota de "falta algo" y cada cambio guarda quién y cuándo. El operario marca al empezar o al terminar, como prefiera. **Las medidas solo las cambia el propietario**: cada cambio crea una **revisión** de la ventana (con motivo, quién y cuándo) y el trabajo conserva todas; el presupuesto aceptado no cambia. La **orden de taller** reúne el dibujo, las medidas, el material y el **plan de corte** (qué piezas salen de cada barra). Al terminar la orden se registran las salidas de stock.

### 3.7 Libro de stock
Un **movimiento** dice: qué artículo, en qué color, cuánto, si es entrada, salida o ajuste, y por qué (una orden de taller, un ajuste, una compra). El saldo es la suma. Una barra más corta que la estándar se anota con su longitud, para que el optimizador la use antes de abrir una nueva.

### 3.8 Facturas y cobros
Una **factura** se crea desde un trabajo o libre. Es **borrador** hasta que se emite; en ese momento recibe su número correlativo **sin huecos** (por serie y año), se guarda su fotografía con una huella de integridad y ya no se puede cambiar. Una corrección es una **rectificativa** que apunta a la original. Los **cobros** viven aparte: cada uno es un apunte con fecha e importe, y el estado "pendiente, parcial o pagada" se calcula sumándolos. El **albarán** es el justificante de entrega de un trabajo. La factura reserva campos vacíos para Verifactu y la factura electrónica, para poder añadirlos sin rehacer nada.

### 3.9 Sistema
Configuración del taller (datos fiscales, logo, IVA por defecto, formato de numeración, grosor de sierra, merma), contadores de numeración, **auditoría** (quién hizo qué y cuándo), estado de las copias de seguridad y versión del esquema de la base de datos.

## 4. Qué se congela y cuándo
| Momento | Qué se guarda como fotografía |
|---|---|
| Aceptar un presupuesto | Líneas, medidas, resultado del cálculo, precios, versión de serie y de tarifa |
| Terminar una orden de taller | El plan de corte real y las salidas de stock |
| Emitir un albarán o una factura | Datos del cliente, líneas, tipos de IVA y huella SHA-256 |

## 5. Ejemplo de principio a fin
Se da de alta un **cliente** con una dirección de montaje. Se crea el **presupuesto P-2026-0001** con una línea: corredera de 2 hojas de Aluprom 21, 150 × 120 cm, blanco. El cálculo da las piezas, pero el cristal sigue en `CONFIRMAR`, así que la línea sale **incompleta** y no se puede aceptar. Cuando el cristal esté definido, el propietario **acepta** y nace el **trabajo T-2026-0001** con la copia congelada. Lo **asigna** a un operario, que ve sus ventanas, marca "en curso" al empezar a cortar y deja una nota "falta cristal" en la que no puede avanzar. El propietario **termina la orden**: se generan las salidas de stock de barras, kit, ruedas y tornillos en una sola operación. Se emite la **factura F-2026-0001**, que recibe su número, se congela y queda a la espera de **cobros**.

## 6. Decisiones pendientes que afectan a los datos
1. Dónde viven las series en ejecución. Propuesta: en la base de datos, versionadas, con el YAML como formato de importación y exportación (formato en `docs/FORMATO_SERIES.md`).
2. Qué medida introduce el usuario: hueco de obra, luz o medida de ventana. Las fórmulas dadas sugieren medida de ventana.
3. ~~Cómo se enlaza el perfil con la tarifa~~: resuelto con la tabla `referencias` de cada serie (`serie_referencia`).
4. Si el recargo de color es el mismo para todos los proveedores o varía por proveedor.
5. Margen sobre coste o sobre venta; coste de perfiles por barras tras optimizar o por metros más merma.
6. Cuándo se descuenta el stock: al terminar la orden o al marcar el paso de corte.
7. Lista de pasos por defecto de una ventana y si cambia según la tipología.
8. Formato visible de la numeración y letras de cada serie (`P-`, `T-`, `F-`).
9. Reservas de stock al aceptar un presupuesto (`BACKEND` 5.5): ¿entran en el MVP?
10. Facturas de proveedores y archivos adjuntos: ¿entran en el MVP o quedan reservadas?

## 7. Tablas y campos (para Claude Code)
El esquema de H0 a H6 (usuarios, sesiones, auditoría y esquema; series; clientes y catálogo; presupuestos; trabajos; stock y facturas) es la migración `crates/servidor/src/bd/migraciones/0001_inicial.sql`; la 0002 añade los permisos de operario (ADR-0024). Desde la 0.11.0 cada cambio es una migración nueva y la versión de la base es `PRAGMA user_version` (ADR-0021).

### 7.1 Convenciones
| Convención | Regla |
|---|---|
| Tablas | `STRICT`; clave `id INTEGER PRIMARY KEY`; claves foráneas `ON DELETE RESTRICT`, con `foreign_keys=ON` |
| Fechas | `TEXT` ISO 8601 en UTC (`*_en`); se muestran en Europe/Madrid |
| Dinero | `*_cent` `INTEGER` en céntimos |
| Precio unitario | `*_pu` `INTEGER` en 1/10 000 € |
| Cantidad | `*_mil` `INTEGER` en milésimas de unidad |
| Longitud | `*_dmm` `INTEGER` en décimas de milímetro (640 cm = 64 000) |
| Porcentajes | `*_bp` `INTEGER` en puntos básicos (1 % = 100) |
| Booleanos | `INTEGER` 0/1 con `CHECK` |
| Estados | `TEXT` con `CHECK (... IN (...))` |
| JSON | `*_json` `TEXT` canónico (claves ordenadas) |
| Editables | columna `version INTEGER` para bloqueo optimista; `archivado` en catálogo y clientes |

### 7.2 Personas, acceso y sistema
| Tabla | Campos |
|---|---|
| `usuario` | id, nombre, rol (`propietario`/`operario`), clave_hash (argon2id: contraseña o PIN), activo, intentos_fallidos, bloqueado_hasta, creado_en, version |
| `bloqueo_ip` | ip (PK), intentos_fallidos, bloqueado_hasta (bloqueo progresivo por dirección) |
| `usuario_permiso` | usuario_id, permiso (`editar_clientes`/`ver_presupuestos`/`ver_facturas`/`ver_todos_los_trabajos`). Migración 0002, ADR-0024 |
| `sesion` | id (hash del identificador de 256 bits), usuario_id, creada_en, ultima_actividad_en, caduca_en, token_csrf, hasta_cerrar_navegador (0/1, opción del propietario al entrar) |
| `configuracion` | clave (PK), valor, actualizado_en, usuario_id |
| `secuencia_documentos` | serie, anio, ultimo; PK (serie, anio) |
| `auditoria` | id, cuando_en, usuario_id, accion, entidad, entidad_id, detalle_json (solo añadir) |
| `copia_seguridad` | id, cuando_en, resultado, ruta, tamano_bytes |
| `esquema` | huella (SHA-256 de la migración 0001; desde la 2, la de «migracion-N» para que las versiones antiguas no abran la base), creado_en (ADR-0021) |

### 7.3 Clientes
| Tabla | Campos |
|---|---|
| `cliente` | id, tipo (`particular`/`empresa`, por defecto `particular`), **todos los demás campos admiten NULL**: nombre_o_razon_social, nif, email, telefono, fiscal_calle, fiscal_cp, fiscal_localidad, fiscal_provincia, fiscal_pais, forma_de_pago, notas, busqueda (nombre, NIF, teléfono, correo y localidad sin tildes), importacion_id, archivado, creado_en, version |
| `direccion_montaje` | id, cliente_id → cliente (puede no haber ninguna), etiqueta, calle, cp, localidad, provincia, contacto_nombre, contacto_telefono, notas, archivado |

### 7.4 Catálogo
| Tabla | Campos |
|---|---|
| `importacion` | id, tipo (`tarifa`/`clientes`), sha256, filas, estado (`confirmada`/`deshecha`), usuario_id, importada_en, deshecha_en (ADR-0012) |
| `proveedor` | id, nombre, nif, contacto, condiciones_pago, descuento_habitual_bp, notas, archivado, version |
| `acabado` | id, nombre, es_base (blanco), recargo_bp, archivado |
| `acabado_recargo_proveedor` | acabado_id, proveedor_id, recargo_bp (solo si el recargo varía por proveedor; ver decisión 4) |
| `articulo` | id, proveedor_id, referencia, descripcion, tipo (`perfil`/`vidrio`/`herraje`/`accesorio`/`kit`/`otro`), unidad (`barra`/`metro`/`m2`/`unidad`/`caja`), longitud_barra_dmm (NULL si no es barra), unidades_por_caja, con_acabado, stock_minimo_mil, importacion_id, archivado, creado_en, version; único (proveedor_id, referencia) |
| `kit_componente` | kit_articulo_id → articulo, componente_articulo_id → articulo, cantidad_mil |
| `tarifa` | id, proveedor_id, nombre, fecha_tarifa, vigente_desde, descuento_compra_bp, formato_origen (`csv`/`ajuste`/`edicion`), archivo_origen, estado (`vigente`/`sustituida`; una vigente por proveedor), importacion_id, importada_en, usuario_id. Cada versión guarda todos los precios del proveedor |
| `tarifa_precio` | tarifa_id, articulo_id, precio_pu (del acabado base, por unidad de venta), minimo_pedido_mil; PK (tarifa_id, articulo_id) |

### 7.5 Series de perfiles (las recetas)
| Tabla | Campos |
|---|---|
| `serie` | id, codigo (`ALUPROM_21`), fabricante, creada_en |
| `serie_version` | id, serie_id, numero, estado (`borrador`/`validada`/`demo`), origen_yaml (texto original), sha256, avisos, creada_en (quién la importó consta en la auditoría); único (serie_id, numero); no se modifica ni se borra (ADR-0009) |
| `serie_referencia` | serie_version_id, codigo (`MARCO_LATERAL`, `BISAGRA`), referencia_texto (`CONFIRMAR` si falta), proveedor_texto; PK (serie_version_id, codigo). En H2 se añade el enlace con el artículo de la tarifa |

Las tipologías, piezas, vidrios, accesorios y opciones no tienen tablas propias: se leen del YAML de la versión (ADR-0009).

### 7.6 Presupuestos
| Tabla | Campos |
|---|---|
| `presupuesto` | id, anio, numero (`P-AAAA-NNNN`), cliente_id, direccion_montaje_id, estado (`borrador`/`enviado`/`aceptado`/`rechazado`/`caducado`), validez_hasta, condiciones, descuento_bp, enviado_en, aceptado_en, copia_json y copia_sha256 (solo al aceptar; un trigger impide modificar lo aceptado), creado_por, creado_en, version; único (anio, numero) |
| `version_presupuesto` | id, presupuesto_id, numero_version, motivo, vigente (una por presupuesto), base_cent, iva_cent, total_cent, completo, creada_en |
| `linea_presupuesto` | id, version_id, orden, tipo (`ventana`/`libre`), descripcion, cantidad_mil, serie_version_id, tipologia, ancho_dmm, alto_dmm, acabado_id, vidrio_articulo_id, opciones_json, resultado_json (piezas, vidrios, accesorios, barras, costes y dibujo), faltan_json, incompleta, revisar_limites, tarifas_json, coste_cent, mano_de_obra_cent, descuento_bp, tipo_iva_bp, precio_unitario_pu (libres), precio_cent |
| `presupuesto_tarifa` | presupuesto_id, tarifa_id: tarifas usadas por un presupuesto aceptado (no se pueden deshacer) |
| `secuencia_documentos` | serie (`P`, `T`, `F`, `R`, `A`), anio, ultimo |

Ajustes de empresa y precio en `configuracion`: `empresa.*`, `precio.margen_modo`, `precio.margen`, `precio.perfiles`, `precio.merma`, `precio.mano_de_obra`, `iva.general`, `iva.reducido`, `presupuesto.validez_dias`, `presupuesto.condiciones`.

### 7.7 Trabajos y orden de taller
| Tabla | Campos |
|---|---|
| `trabajo` | id, anio, numero, presupuesto_id (único), cliente_id, cliente_nombre y direccion_montaje (copiados al aceptar), importe_cent (el aceptado), estado (`pendiente`/`en_curso`/`terminado`/`entregado`/`anulado`), asignado_a (usuario_id), fecha_prevista, creado_en, version. Lo congelado va en cada ventana |
| `ventana_trabajo` | id, trabajo_id, orden, linea_orden, descripcion, asignado_a (usuario_id), cristal_recibido (0/1, se marca a mano), revision (1 al aceptar), revisada_en, medida_cambiada (aviso al operario hasta que la abre), falta_algo (la nota «falta algo» vive en la ventana, no en cada paso), congelado_json (versión de la serie, tipología, medidas, opciones, color, resultado y longitud de barra; la revisión vigente), version |
| `paso_ventana` | id, ventana_id, orden, nombre, estado (`pendiente`/`en_curso`/`hecho`), usuario_id, cambiado_en, version (si dos tocan el mismo paso, el segundo recibe aviso) |
| `cambio_ventana_trabajo` | id, ventana_trabajo_id, revision, congelado_anterior_json, congelado_nuevo_json, motivo, pasos_reiniciados (0/1), usuario_id, cuando_en (solo añadir). Cada **Modificar medidas** del propietario crea una revisión; `ventana_trabajo.congelado_json` guarda la vigente |
| `paso_historial` | id, paso_id, estado_anterior, estado_nuevo, nota, usuario_id, cuando_en (solo añadir) |
| `orden_taller` | id, trabajo_id, generada_en, plan_de_corte_json, estado (`abierta`/`terminada`), terminada_en, terminada_por |
| `albaran` | id, serie_numeracion, anio, numero, trabajo_id, fecha, snapshot_json, snapshot_sha256 |

### 7.8 Stock
| Tabla | Campos |
|---|---|
| `movimiento_stock` | id, articulo_id, acabado_id (NULL si no tiene color), longitud_dmm (NULL = barra estándar del artículo), tipo (`entrada`/`salida`/`ajuste`), cantidad_mil (con signo), motivo, orden_taller_id, usuario_id, cuando_en; único (orden_taller_id, articulo_id, acabado_id, longitud_dmm) para que terminar una orden sea idempotente |
| `pieza_repetida` | id, ventana_id, numero (de la pieza en el dibujo), perfil, longitud_dmm, cantidad, motivo (corte mal, rayón…), usuario_id, cuando_en. Entra en el plan de corte y se descuenta al terminar la orden (H5) |
| `reserva_stock` | id, trabajo_id, articulo_id, acabado_id, cantidad_mil, estado (`activa`/`liberada`/`consumida`) (ver decisión 9) |

El saldo de un artículo es `SUM(cantidad_mil)` por artículo y color, con índice en `articulo_id`. No hay saldo duplicado.

### 7.9 Facturas y cobros
| Tabla | Campos |
|---|---|
| `factura` | id, serie_numeracion, anio, numero (NULL hasta emitir), tipo (`ordinaria`/`rectificativa`), rectifica_factura_id, cliente_id, trabajo_id, estado (`borrador`/`emitida`), fecha_emision, fecha_operacion, vencimiento, forma_de_pago, base_cent, iva_cent, total_cent, snapshot_json, snapshot_sha256, huella_anterior, huella, estado_envio (las tres últimas, reservadas y vacías), version |
| `linea_factura` | id, factura_id, orden, descripcion, cantidad_mil, precio_unitario_pu, descuento_bp, tipo_iva_bp, base_cent |
| `cobro` | id, factura_id (NULL mientras sea una **señal** a cuenta), trabajo_id (la señal se cobra al aceptar, antes de la factura; al emitirla, sus señales cuentan como ya pagado), fecha, importe_cent, medio (`efectivo`/`bizum`/`tarjeta`/`transferencia`/`otro`), nota, anulado, motivo_anulacion, usuario_id, creado_en |

Triggers: `BEFORE UPDATE` y `BEFORE DELETE` en `factura`, `linea_factura`, `albaran`, `serie_version` y `tarifa_precio` con `RAISE(ABORT)` cuando el documento ya está emitido o usado. Al emitir, el número se asigna en la misma transacción que el cambio de estado.

### 7.10 Reservado (no entra salvo decisión)
`factura_proveedor` (proveedor_id, número, fecha, importes, pagada, adjunto_id), `pedido_proveedor` y sus líneas, `adjunto` (id, ruta, tipo, tamaño, sha256; archivos en disco, no dentro de la base), `agenda_instalacion` e `incidencia_postventa`.
