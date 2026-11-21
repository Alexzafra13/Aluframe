# AlumFrame: especificación del MVP

## 1. Objetivo
Un programa de gestión para un taller pequeño de carpintería de aluminio y cerrajería: presupuestar ventanas con precio y dibujo, generar la hoja de corte para el operario, controlar el stock de material y facturar.

El taller corta **todo a mano** (sin tronzadora ni máquinas conectadas). El documento clave para el taller es una **hoja impresa** con el dibujo, las medidas de corte y las medidas de cristal.

**Fuera del MVP:** Verifactu (se deja fuera por decisión; ver 8), conexión con maquinaria, marcado CE, eficiencia energética, multiempresa, acceso público desde internet, compras a proveedores con pedidos y lectura automática de facturas (al principio se teclean o se importan), fotos y planos adjuntos, agenda de instalaciones, postventa y garantías.

## 2. Usuarios y roles
- `propietario` (dueño): ve precios, márgenes, facturas y configuración.
- `operario` (operario): lo da de alta el propietario (usuario y PIN). Ve solo los trabajos que se le asignan ("Mi día"), con la orden de taller (dibujo, medidas, material), y va marcando lo que hace y lo que falta. No ve precios ni márgenes.

## 3. Flujo principal
```
Cliente -> Presupuesto -> (aceptado) Trabajo -> Orden de taller -> Stock descontado -> Factura -> Cobro
```
- Un **presupuesto** tiene versiones y líneas. Una línea es una **ventana** (tipología + medidas + vidrio + acabado) o una **línea libre** (cerrajería, instalación, material suelto).
- Al **aceptar** un presupuesto se crea un **trabajo** con una copia congelada de todo lo calculado.
- La **orden de taller** se genera desde el trabajo y es lo que se imprime o se abre en la tablet.
- El propietario **asigna** el trabajo (o cada ventana) a un operario. Cada ventana lleva una lista de pasos marcable (la lista por defecto se define con el taller). Cada paso está `pendiente`, `en curso` o `hecho`, y puede llevar una nota de "falta algo". El operario puede marcar al empezar (en curso) o al terminar (hecho): ambas vías valen y el programa guarda quién y cuándo. Así puede dejar una ventana a medias, pasar a otra y retomarla.
- Al marcar la orden como **terminada** se descuenta el stock.
- **Las medidas de un trabajo aceptado solo las cambia el propietario**, ventana a ventana, mientras la orden no esté terminada: se recalcula con la misma versión de serie y de tarifa, se guarda una **revisión** con el motivo, el presupuesto original no cambia y la orden impresa lleva su número de revisión. El operario nunca modifica medidas.
- La **factura** se crea desde un trabajo (o libre), con numeración correlativa y estado de cobro.

## 4. Modelo de dominio (resumen)
- **Cliente**: datos fiscales y de contacto, tipo (particular o empresa) y las **direcciones de montaje** que haga falta (con persona de contacto y notas). **Ningún dato es obligatorio**; sin nombre se muestra "Cliente sin nombre (n.º N)". El presupuesto elige una dirección o añade otra al momento.
- **Proveedor** y **Articulo**: perfiles, vidrios, herrajes y accesorios. Cada artículo tiene referencia, descripción, unidad (barra, metro, m², unidad), longitud de barra, color/acabado, stock mínimo.
- **Acabado (color)**: recargo en % sobre el precio del blanco (el más barato); por confirmar si es igual para todas las series o depende del proveedor.
- **Precio**: tarifa del proveedor, descuento del proveedor (coste) y margen (venta). Con **histórico** (fecha, origen, importación).
- **SerieDePerfiles**: serie de perfiles definida como **datos** (ver 5).
- **Presupuesto** (+ `VersionPresupuesto`, `LineaPresupuesto`), **Trabajo**, **OrdenDeTaller**, **MovimientoDeStock** (entrada, salida, ajuste), **Factura** (+ líneas, cobros).
- Dinero en céntimos (enteros). Longitudes en décimas de milímetro (enteros; los archivos de series y la interfaz usan la unidad que declara la serie, por ahora cm con decimales como 4,85, y se convierten). Identificadores enteros autoincrementales y fechas en UTC.

## 5. Series de perfiles como datos
Una serie se describe con datos (YAML o JSON validado al cargarse), no con código. Las fórmulas son expresiones sobre `ancho`, `alto` y parámetros de la serie, evaluadas con un evaluador de expresiones **seguro** (sin `eval`).

```yaml
serie: ALUPROM_21        # formato de referencia: data/series/aluprom-21.yaml
estado: borrador         # borrador = no puede emitir precio final
unidades: cm             # un decimal de cm = 1 mm
redondeo: { precision: 0.1, sentido: abajo }   # provisional
longitud_barra: 640      # cm
parametros:
  descuento_marco_horizontal: 4.1
  descuento_hoja_vertical: 5.3
tipologias:
  corredera_2_hojas:
    piezas:
      - { perfil: MARCO_SUPERIOR, cantidad: 1, longitud: "ancho - descuento_marco_horizontal", corte: 90 }
      - { perfil: HOJA_CIERRE,    cantidad: 2, longitud: "alto - descuento_hoja_vertical",     corte: 90 }
    vidrio: { estado: CONFIRMAR }
    accesorios:
      - { articulo: RUEDA_CORREDERA, cantidad: 4 }
```

Reglas:
- **Nunca inventes datos reales.** Los datos ficticios para pruebas y demos viven en `fixtures/demo/` con `estado: demo`, y el programa no los carga en el uso real. Los datos de series vienen de `data/series/` y los casos esperados de `fixtures/real/`, aportados por el usuario. Lo marcado `borrador` se muestra como tal en la interfaz.
- Se define qué medida introduce el usuario: **hueco de obra**, **luz** o **medida de fabricación**, y el descuento a fabricación es un parámetro de la serie.
- El redondeo es configurable por serie (por ahora, a 0,1 cm hacia abajo; provisional, a validar con ventanas reales).
- **Kits**: un artículo puede ser un conjunto de otros (por ejemplo, el kit de corredera) con sus cantidades. En el presupuesto se ve como una línea y en stock se descuentan sus componentes.
- **Consumibles con presentación**: los tornillos y similares se compran por caja pero se consumen por unidad. El artículo guarda `unidades_por_caja` y el stock se lleva en unidades.
- Una tipología puede declarar **opciones de línea** que elige quien presupuesta o asigna el trabajo, por ejemplo el lado de la cremona en la abatible (hoja izquierda o derecha). Salen en el dibujo y en la hoja de corte, y el operario solo las ve.
- Las cantidades de accesorios pueden depender de la parte de la ventana (`aplica_a`: marco u hojas) para poder desglosarlas.
- **Pendiente de decidir (ADR):** dónde viven las series en ejecución. Propuesta: en la base de datos, versionadas e inmutables una vez usadas, con los archivos YAML como formato de importación y exportación.
- La primera serie real está en `data/series/aluprom-21.yaml` (borrador). Los campos `CONFIRMAR` no pueden usarse para precios hasta validarlos con una ventana real.
- Una **línea incompleta** (falta el cristal, un precio o un dato `CONFIRMAR`) se calcula igualmente, pero se marca como incompleta y **no permite emitir precio final ni aceptar el presupuesto**.
- El resultado del cálculo (despiece, vidrio, accesorios, precio) se **congela** al aceptar el presupuesto.

## 6. Optimización de barras
- Entrada: lista de cortes por perfil y color, longitud de barra y **grosor de sierra** (parámetro).
- Algoritmo inicial: Best Fit Decreasing (ver `docs/BACKEND.md`, apartado 5.3). La interfaz debe permitir cambiar de algoritmo.
- Salida: barras necesarias, esquema de cortes por barra, desperdicio y retales.
- En el presupuesto se estima por metros más porcentaje de merma; en la orden de taller se usa el resultado real de la optimización. Se guardan ambos para comparar.
- Propiedades a testear: ninguna barra supera su longitud, no se pierde ni duplica ninguna pieza, el resultado es determinista.

## 7. Precios y tarifas
- Importación desde **plantilla CSV/Excel** fija: `referencia, descripción, unidad, precio_tarifa, longitud_barra`.
- **Vista previa obligatoria** antes de guardar: artículos nuevos, cambios de precio con diferencia en %, filas con error. Hasta confirmar no se modifica nada.
- Validaciones: precio vacío o cero, salto superior al 30 %, referencias duplicadas, unidades desconocidas.
- Herramientas de ajuste: subir o bajar un porcentaje a una selección y edición en tabla.
- Cada importación queda registrada y se puede **deshacer**.
- Los trabajos aceptados conservan los precios con los que se hicieron.

## 8. Facturación (sin Verifactu por ahora)
- Factura desde trabajo o libre, imprimible en A4, numeración correlativa **sin huecos** (obligatoria solo para facturas y rectificativas) por serie y año, IVA, estados de cobro y vencimientos.
- **Una factura emitida no se edita**: se corrige con una rectificativa.
- Los datos del cliente no son obligatorios para presupuestar ni para crear el borrador. Al emitir, el programa avisa de lo que una factura completa suele exigir (nombre y NIF del cliente) y deja emitirla tras una confirmación explícita. **Confirmar con el gestor** qué casos admiten factura simplificada.
- Los datos del cliente y las líneas se copian a la factura al emitir.
- Se reservan en la tabla campos para la huella y el estado de envío (vacíos por ahora) para poder añadir Verifactu más adelante sin reescribir. Hacienda anunció en octubre de 2026 el aplazamiento de Verifactu a octubre de 2028, aún sin norma publicada, y la factura electrónica obligatoria entre empresas llega en la misma fecha: el taller factura a constructoras, así que hay que confirmar con el gestor qué formato y qué envío exigirá.
- **Cadena de documentos:** presupuesto, trabajo, albarán de entrega (opcional), factura, rectificativa. Cada documento referencia al anterior.
- **IVA por línea** (no solo por documento): tipo general y reducido, y marca para casos especiales (por ejemplo inversión del sujeto pasivo con constructoras). El taller debe confirmar con su gestor cuándo aplica cada uno.
- **Descuentos** por línea y por documento; condiciones y forma de pago (efectivo, transferencia, tarjeta, plazos); vencimientos y recibos.
- **Anticipos y facturas parciales** de un mismo trabajo.
- Validación de NIF/CIF, envío del PDF por correo (requiere el PDF automático de ADR-0003) y exportación de facturas para el gestor.
- **Referencia de estudio antes de H6:** analizar el modelo de documentos de FacturaScripts (software libre, licencia LGPL-3.0, escrito en PHP) para contrastar que no falta ningún campo ni estado. Se toman las ideas, no el código. Contabilidad completa, modelos fiscales y conciliación bancaria quedan fuera: los lleva el gestor.

## 9. Stock
- El stock actual es la **suma de movimientos** (entrada, salida, ajuste), nunca un número editable.
- Inventario inicial por artículo y color: barras enteras y **retales existentes** (cada uno con su longitud). Es **opcional**: el programa funciona sin inventario y los retales se pueden registrar cuando se quiera, al instalar o más tarde (decisión del propietario, 2026-11-21).
- **Los retales se usan antes que las barras nuevas:** el plan de corte propone qué retal coger (perfil, color y longitud) y, al terminar la orden, el retal sale del stock y el sobrante entra como retal nuevo. **Retal mínimo: 0** (se guarda cualquier sobrante por si encaja en algo; decisión del propietario). Un retal que se tira se da de baja con un ajuste.
- Entradas de material con una pantalla de pocos toques. Salidas automáticas al terminar la orden de taller.
- Stock mínimo por artículo, aviso y lista de compra sugerida a partir de trabajos pendientes.

## 9 bis. Interfaz
La organización de la interfaz (menú, pantallas, rutas, permisos y el Inicio como tablero de trabajos) está en `docs/PANTALLAS.md`; el formato de los archivos de series, en `docs/FORMATO_SERIES.md`; el modelo de datos, en `docs/DATOS.md`. El diseño visual aprobado está en `design/`.

## 10. Documentos imprimibles (A4)
En el MVP son páginas HTML con una hoja de estilos de impresión A4: se imprimen o se guardan como PDF desde el navegador. La generación automática de PDF en el servidor (para enviar por correo o archivar) se decide en ADR-0003, **antes de empezar H3**.

- Presupuesto (con dibujo de cada ventana, sin costes ni márgenes).
- **Hoja de corte / orden de taller**: dibujo, medidas de cada pieza con ángulo, medidas de cristal, accesorios, esquema de cortes por barra.
- Factura.

## 11. Requisitos no funcionales
- Funciona en red local sin internet. Usuarios simultáneos: hasta 5.
- **Servidor:** un PC pequeño dedicado, con cable de red, IP fija y SAI. El PC de la oficina y, más adelante, una tablet de operario se conectan por navegador. Al empezar puede alojarse en el PC principal, porque el programa es un ejecutable y un archivo de datos fáciles de mover. El mismo ejecutable funciona en **modo servidor** (red local) o **modo local** (solo ese PC, abre el navegador); ver `docs/BACKEND.md`, apartado 14. Nunca se comparte el archivo de datos por carpeta de red entre varios PCs.
- Copia de seguridad nocturna automática a un disco externo, más una copia fuera del taller. Botón de descarga manual. Test de restauración.
- Arranque automático y reinicio ante fallos (servicio del sistema). Si hay corte de luz no se corrompe la base de datos.
- **Mantenible con poco esfuerzo:** pruebas completas, decisiones en `docs/adr/` y una guía en `docs/MAINTENANCE.md`, de modo que otra sesión de Claude pueda retomar el proyecto sin contexto previo.
- Registro de actividad (quién cambió qué en documentos y precios).
- Punto de comprobación `/health`, registros de eventos y estado de la última copia de seguridad visible en la interfaz.
- Datos personales de clientes (RGPD): los mínimos necesarios, exportables y borrables. Acceso solo desde la red local.

## 12. Hitos y criterios de aceptación
Cada hito termina con tests en verde, `docs/PROGRESS.md` actualizado y una demo reproducible. Las pantallas de cada hito están en `docs/PANTALLAS.md`, apartado 16.

### H0. Cimientos
Workspace de Cargo, CI, `just check`, reglas de capas, base de datos con migraciones, autenticación y roles, esqueleto de interfaz con el diseño elegido.

Tareas de H0:
1. Workspace de Cargo (`crates/dominio`, `crates/servidor`), `rust-toolchain.toml`, `justfile` (dev, check, test, e2e, build, release, lint-size, db-migrate), `clippy.toml`, `deny.toml`.
2. CI que ejecuta `just check`, y script `lint-size` que falla si un `.rs` supera 300 líneas.
3. Servidor Axum mínimo: configuración, registros con `tracing`, `/health`, plantilla base, CSS con los tokens y página de inicio de sesión.
4. Base de datos: conexión con los ajustes de `docs/BACKEND.md`, migración 0001 (usuarios, sesiones, auditoría, versión del esquema) y prueba de migración desde cero.
5. Autenticación: propietario con contraseña, operario con PIN, sesiones, CSRF, bloqueo progresivo y permisos por rol.
6. Pruebas: unitarias, integración con SQLite en memoria y un E2E de inicio de sesión contra el ejecutable compilado.
7. Resolver en ADR la elección de plantillas (Askama o Maud), la zona horaria (`time` no la incluye; Europe/Madrid) y la herramienta de cobertura; documentar los tokens en `docs/design.md`.
8. Hook previo al push versionado en `.githooks/` (activado con `git config core.hooksPath .githooks`) y receta `just db-migrate`.
9. Datos ficticios en `fixtures/demo/` (serie y tarifa) para los E2E de H3 en adelante, mientras falten fórmulas reales de cristal.
- Aceptación: `just check` en verde, también en el hook previo al push y en CI; existe `/health`; un usuario `propietario` (contraseña) y otro `operario` (PIN) pueden entrar y ven pantallas distintas.

### H1. Motor de dominio
Dinero, series como datos, evaluador de fórmulas, cálculo de una ventana (despiece, vidrio, accesorios, precio), optimizador de barras.
- Aceptación: tests de propiedades del optimizador; la serie `ALUPROM_21` calcula la corredera de 2 hojas de 150 × 120 cm igual que `fixtures/real/aluprom21-corredera2-150x120.json`; el resultado sale marcado como incompleto mientras falte el cristal.

### H2. Catálogo, clientes e importación de tarifas
CRUD de clientes y artículos, importación de clientes desde CSV, precios con histórico, importación de tarifas con vista previa y deshacer.
- Aceptación: importar un CSV con errores muestra los avisos y no guarda nada hasta confirmar; deshacer restaura los precios.

### H3. Presupuestos
Versiones, líneas de ventana y libres, dibujo SVG, cálculo en vivo (la página lo pide al servidor mientras se escribe), presupuesto imprimible en A4.
- Aceptación: crear un presupuesto de 3 ventanas distintas con una línea libre, ver el dibujo, imprimirlo o guardarlo como PDF desde el navegador y que el total coincida con el cálculo del dominio.

**Punto de validación con el taller:** probar H1–H3 con 5 ventanas reales y comparar con lo que haría el dueño a mano. No se sigue hasta que cuadre. *(2026-11-21: aplazado por decisión del propietario hasta tener los datos; se sigue con H4 y la validación se hará en cuanto lleguen. Ver `docs/CAMBIOS.md`.)*

### H4. Trabajos y orden de taller
Aceptar presupuesto, congelar cálculo, orden de taller en pantalla y en hoja imprimible A4, estados del trabajo.
- Aceptación: cambiar una fórmula después de aceptar no altera un trabajo existente; el rol `operario` no ve precios y no puede modificar medidas; el propietario sí puede, con revisión y motivo, mientras la orden no esté terminada, y la orden impresa muestra su revisión.

### H5. Stock
Movimientos, inventario inicial, entradas, descuento al terminar la orden, mínimos y lista de compra.
- Aceptación: terminar una orden descuenta exactamente el material optimizado; el stock nunca se edita a mano sin crear un ajuste; con retales registrados del mismo perfil y color, el plan de corte los usa antes de abrir barras nuevas y la hoja de taller dice cuál coger.

### H6. Facturación
Facturas desde trabajo o libres, imprimibles en A4 (PDF automático según ADR-0003), numeración correlativa, cobros, rectificativas.
- Aceptación: no se puede editar una factura emitida; la numeración no tiene huecos; una rectificativa referencia a la original.

### H7. Copias de seguridad y despliegue
Ejecutable único instalado como servicio (ADR del envoltorio de servicio en Windows) y también utilizable en modo local, copia nocturna, copia remota opcional, restauración probada, guía de instalación en el taller y `docs/MAINTENANCE.md` completa.
- Aceptación: restaurar una copia en una máquina limpia reproduce los datos; reiniciar la máquina deja el servicio funcionando.

## 13. Lo que necesita aportar el usuario (Claude no puede inventarlo)
1. Serie real de perfiles (formato en `docs/FORMATO_SERIES.md`): **parcial**. Aluprom 21 (corredera de 2 y 4 hojas) y Sopena V-94 (abatible) están en borrador; faltan el cristal, las pletinas de falleba y las referencias reales.
2. 5 ventanas reales con hoja de corte y precio cobrado: **pendiente** (el fixture actual sale de las fórmulas, `validado: false`).
3. Una tarifa real de proveedor: **pendiente**.
4. Datos fiscales del taller y logotipo para los PDF: **pendiente**.
5. Cómo se mide (hueco, luz o fabricación) y cómo se cobra el material (barra, metro o kilo): **pendiente**.

## 14. Preguntas abiertas
La lista viva está en `docs/OPEN_QUESTIONS.md`; esto es un resumen.
- Fórmulas de ancho y alto del cristal y tipo de vidrio habitual.
- Las medidas `ancho` y `alto`, ¿son hueco de obra o medida de la ventana?
- ¿Marco superior, lateral e inferior salen del mismo perfil o de perfiles distintos?
- Proveedores y formato de sus tarifas.
- Tipos de IVA y casos especiales (confirmar con el gestor).
- Formato de numeración de documentos (por ejemplo P-2026-0001).
