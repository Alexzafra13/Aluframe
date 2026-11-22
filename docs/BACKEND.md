# Backend: diseño técnico (versión 1, borrador)

Notas de trabajo antes de construir. Concretan algoritmos, organización del código y reglas de estilo del servidor. Lo marcado **(confirmar)** depende de datos del taller o del gestor. Reglas generales en `CLAUDE.md`; alcance en `SPEC.md`.

## 1. Principios
- **Núcleo puro, bordes con efectos.** `dominio` calcula; `servidor` guarda, sirve y escribe archivos.
- **Dependencias hacia dentro:** `http → aplicacion → dominio`. `bd`, `copias` y el PDF dependen del dominio, nunca al revés.
- **Determinismo:** misma entrada, mismo resultado. Sin `f64`, sin depender del orden de un `HashMap`, sin reloj ni azar dentro del dominio.
- **Estados inválidos irrepresentables:** enums y tipos propios en lugar de cadenas y números sueltos.
- **Errores como valores** (`Result`); nada de `panic`.

## 2. Organización de archivos
```
crates/dominio/src/
  lib.rs
  unidades/     dinero.rs  tasa.rs  longitud.rs  cantidad.rs  redondeo.rs
  formula/      lexico.rs  sintaxis.rs  arbol.rs  evaluacion.rs  error.rs
  series/       modelo.rs  validacion.rs  expansion.rs   # tipología + medidas -> piezas
  corte/        problema.rs  mejor_ajuste.rs  exacto.rs  plan.rs   # optimizador de barras
  precio/       coste.rs  margen.rs  impuestos.rs  totales.rs
  dibujo/       geometria.rs  svg.rs                     # geometría y SVG
  stock/        movimiento.rs  saldo.rs
  documentos/   estado.rs  numeracion.rs  instantanea.rs
crates/servidor/src/
  main.rs                          # solo arranque
  config.rs  estado.rs  errores.rs  auditoria.rs
  http/          router.rs  middleware/  rutas/presupuestos.rs trabajos.rs stock.rs ...
  vistas/        mod.rs  plantillas/...                  # plantillas HTML
  aplicacion/    presupuestos/  trabajos/  stock/  facturas/   # un archivo por caso de uso
  bd/            conexion.rs  transaccion.rs  migraciones/*.sql  consultas/presupuestos.rs ...
  autenticacion/ sesiones.rs  pin.rs  csrf.rs  permisos.rs
  copias/        instantanea.rs  retencion.rs
```
Reglas:
- **Un concepto por archivo.** Objetivo de 200 líneas; máximo 300 (se comprueba en CI, salvo datos generados). Funciones de hasta 40 líneas y hasta 4 parámetros; si hacen falta más, una estructura.
- `mod.rs` solo declara y reexporta. Visibilidad `pub(crate)` por defecto; `pub` solo para lo que cruza el crate.
- Un **caso de uso** es una función pública en su archivo (por ejemplo `aceptar_presupuesto.rs`): comprueba permisos, abre la transacción, llama al dominio, guarda y registra la auditoría.
- Tests unitarios cortos en un módulo `tests` junto al código; integración en `tests/`.

## 3. Dependencias entre módulos
- `dominio` solo usa `serde` y `thiserror`; no conoce tokio, axum, rusqlite ni el reloj. Se comprueba con `cargo deny` y con un test que lee los metadatos de Cargo.
- `aplicacion` no importa `http` ni `vistas`. `http` no llama a `bd`: pasa siempre por `aplicacion`.
- El reloj, la fuente de números y el PDF entran como parámetros o puertos, para poder probarlos.

## 4. Tipos del dominio
```rust
pub struct Dinero(i64);            // céntimos
pub struct PrecioUnitario(i64);    // 1/10 000 €
pub struct Cantidad(i64);          // milésimas (m², metros, unidades)
pub struct Tasa(u32);              // puntos básicos: 21 % = 2100
pub struct Longitud(i64);          // décimas de milímetro
pub enum EstadoPresupuesto { Borrador, Enviado, Aceptado, Rechazado }
// las transiciones son funciones que devuelven Result: aceptar(), rechazar()...
```
- Cada identificador tiene su tipo (`IdPresupuesto(i64)`, `IdArticulo(i64)`) para que no se mezclen.
- Operaciones con comprobación de desbordamiento; los cálculos intermedios usan `i128`.

## 5. Algoritmos

### 5.1 Evaluador de fórmulas
1. **Léxico** (`lexico.rs`): números, identificadores, `+ - * /`, paréntesis, comas.
2. **Sintaxis** (`sintaxis.rs`): análisis por precedencia (Pratt) a un árbol `Expresion`. Se analiza **una vez al cargar la serie** y se guarda el árbol.
3. **Validación** al cargar: variables permitidas (`ancho`, `alto` y parámetros de la serie), funciones permitidas (`minimo`, `maximo`, `redondear`, `techo`, `suelo`, `si`) con su número de argumentos (en `si`, el primer argumento es una comparación y solo ahí se admiten comparaciones), longitud ≤ 200 caracteres, profundidad ≤ 16.
4. **Evaluación** (`evaluacion.rs`): decimales de punto fijo en `i128` con 4 decimales; `ancho` y `alto` entran en la unidad que declare la serie (por ahora cm) y se convierten a décimas de mm. Una división redondea a 4 decimales (mitad hacia fuera). El redondeo a la precisión de la serie (por ahora 0,1 cm, hacia abajo) se aplica **una sola vez**, al resultado, según la regla de la serie.
5. **Errores tipados:** división por cero, variable o función desconocida, aridad incorrecta, desbordamiento, resultado no positivo para una longitud.
- Complejidad lineal en el tamaño de la fórmula. Resultados deterministas.
- Pruebas: propiedades con `proptest` (cualquier texto devuelve resultado o error, nunca `panic`; imprimir y volver a analizar da el mismo árbol; ningún desbordamiento silencioso) y casos con las fórmulas reales.
- Ampliar la gramática solo cuando una serie real lo pida.

### 5.2 Expansión de una ventana
Entrada: serie (versión), tipología, `ancho`, `alto`, parámetros. Pasos: evaluar parámetros → evaluar cada fórmula en orden → comprobar longitudes positivas → devolver piezas (perfil, longitud, cantidad, ángulo), cristal y accesorios. Si falta un dato `CONFIRMAR`, el resultado sale **incompleto** y no permite precio final. El resultado guarda el texto de cada fórmula y la versión de la serie.

### 5.3 Optimizador de barras
Problema: por cada grupo (perfil, acabado), colocar piezas en barras de longitud `L` minimizando barras. Es un problema de empaquetado en una dimensión: difícil en general, pero con unas decenas de piezas por grupo es tratable.
- **Datos de entrada:** longitud de barra, grosor de sierra `s`, recorte de extremos, retal mínimo reutilizable, lista de piezas con identificador. Longitud útil = `L − recortes`. Regla conservadora: cada pieza consume `longitud + s`.
- **Agrupación:** las piezas del mismo perfil y acabado se optimizan juntas por trabajo; no se mezclan colores. El mismo código sirve para una sola ventana (confirmar cuál usa el taller).
- **Algoritmo base: Best Fit Decreasing.** Ordenar de mayor a menor (desempate: perfil, número de línea, índice) y colocar cada pieza en la barra con menos espacio sobrante que aún la admita. Coste O(n·b). Garantiza usar como mucho en torno a 11/9 del óptimo, y en instancias pequeñas suele coincidir con él.
- **Objetivos, por orden:** 1) menos barras nuevas; 2) mayor retal aprovechable (concentrar el sobrante en una sola barra); 3) desempate determinista.
- **Modo exacto (opcional, después del MVP):** búsqueda con ramificación y poda sobre piezas ordenadas, con cota inferior `⌈suma / útil⌉` y poda por simetría. Se limita por **número de nodos** (por ejemplo 200 000), no por tiempo, para seguir siendo determinista. Solo se usa si mejora el resultado base. Se activa únicamente si las pruebas con trabajos reales muestran desperdicio evitable.
- **Retales en stock (H5):** hoy el optimizador trabaja con una sola longitud de barra. En H5 se amplía para recibir los retales disponibles del mismo perfil y color (cada uno con su longitud) y llenarlos antes de abrir barras nuevas, con el mismo Best Fit Decreasing; las pruebas de propiedades se extienden (ninguna pieza supera su retal, ninguna se pierde). Los retales se registran al instalar (inventario inicial) y al terminar cada orden.
- **Vidrio:** se pide a medida; no se optimiza.
- **Salida:** `PlanDeCorte` con barras, piezas colocadas (con su identificador), retal y desperdicio. Error si una pieza es más larga que la longitud útil.
- **Pruebas:** cada pieza aparece exactamente una vez; ninguna barra supera su longitud útil; mismo resultado si cambia el orden de entrada; el número de barras nunca baja de la cota inferior; en instancias pequeñas se compara con una búsqueda exhaustiva.

### 5.4 Precio e impuestos
Orden de cálculo de cada línea: **coste → margen → descuento de línea**. Después, a nivel de documento: **descuento del documento → base por tipo de IVA → cuota → total**.
- Coste de perfiles: en el presupuesto, por barras tras optimizar o por metros más merma **(confirmar cuál se cobra)**; se guarda también el otro valor para comparar.
- Cristal: `m² × precio/m²`, con superficie mínima facturable si el proveedor la aplica **(confirmar)**.
- Margen sobre coste o sobre venta **(confirmar)**.
- Aritmética: precio unitario en 1/10 000 €, cantidad en milésimas, producto en `i128` y **un único redondeo** a céntimos (mitad hacia fuera).
- IVA: cuota por tipo sobre la suma de bases, redondeada una vez **(confirmar con el gestor)**. Estrategia seleccionable (`PorLinea` o `PorDocumento`) para no reescribir si cambia.
- Pruebas: la suma de las partes siempre coincide con el total; valores límite; casos reales de facturas del taller.
- **Implementado en H3** (`crates/dominio/src/precio/`): coste = tarifa − descuento de compra (+ recargo del color si el artículo lo lleva); perfiles por barras del plan de corte o por metros cortados + merma, convertidos a la unidad de venta en milésimas; cristal por m² de cada pieza; accesorios por unidad, metro o caja (unidades ÷ unidades por caja). Mano de obra por ventana después del margen (provisional). Precio = (venta + mano de obra) × cantidad − descuento de línea. Cada importe se redondea una vez a céntimos.

### 5.5 Stock
- **Libro de movimientos** (entrada, salida, ajuste); el saldo es la suma por artículo. Con unos miles de movimientos, una consulta con índice por artículo es suficiente; no hay saldo duplicado que pueda desincronizarse.
- Terminar una orden de taller crea todas las salidas en **una transacción** y es idempotente (clave única por orden y artículo).
- Reservas al aceptar un presupuesto; aviso cuando `saldo − reservas < mínimo`.

### 5.6 Numeración
Tabla `secuencia_documentos(serie, anio, ultimo)`. Al emitir se hace, en **la misma transacción** que cambia el documento a emitido, `UPDATE ... SET ultimo = ultimo + 1 RETURNING ultimo`. Índice único `(serie, anio, numero)`. La regla de **no tener huecos** es obligatoria solo para facturas y rectificativas: un borrador de factura no tiene número hasta emitirse, y una corrección es una rectificativa, nunca un borrado. Presupuestos y trabajos se numeran al crearse (el diseño muestra `P-0001` en un borrador) y pueden tener huecos si se descarta uno. Formato visible configurable **(confirmar)**.

## 6. Base de datos y concurrencia
- **Ajustes:** `journal_mode=WAL`, `synchronous=FULL` (pocas escrituras; no se quiere perder la última), `foreign_keys=ON`, `busy_timeout=5000`. Tablas `STRICT`, `CHECK` en estados e importes, claves foráneas `ON DELETE RESTRICT`.
- **Un único escritor:** una conexión tras un `Mutex`, accesible solo mediante `bd.escribir(|tx| …)` (`BEGIN IMMEDIATE`, transacción corta, sin llamadas de red dentro). Lecturas con un grupo pequeño de conexiones de solo lectura. Para unos pocos usuarios no hace falta un actor aparte.
- **Modificaciones perdidas:** columna `version` en los agregados editables; `UPDATE … WHERE id = ? AND version = ?`. Si afecta a 0 filas, error `Conflicto` y la pantalla pide recargar.
- Límite real: las escrituras se serializan, pero a esta escala (decenas por minuto) cada una dura milisegundos. SQLite no es un cuello de botella aquí.
- Convenciones: `id INTEGER PRIMARY KEY`, fechas en UTC (texto ISO 8601), importes como enteros, sin borrado lógico salvo un indicador `archivado` en el catálogo.

## 7. Documentos emitidos
- Al emitir se guarda un **snapshot** con todo lo necesario para reproducir el documento: datos fiscales y del cliente copiados, líneas, medidas, precios unitarios, descuentos, tipos de IVA, referencias del catálogo, **versión de la serie y de la tarifa**. Se serializa en JSON canónico (claves ordenadas) y se guarda su **SHA-256** (sirve de comprobación de integridad y prepara la cadena de Verifactu).
- **Triggers** que impiden modificar un documento emitido (`BEFORE UPDATE`/`DELETE` con `RAISE(ABORT)`). Los cobros viven en otra tabla.
- Series y tarifas **versionadas e inmutables una vez usadas**: editar crea una versión nueva.
- **Modificar medidas de un trabajo** (solo `propietario`, orden no terminada): en una transacción se recalcula con la versión de serie y de tarifa congeladas, se escribe una fila en `cambio_ventana_trabajo` con el estado anterior y el nuevo, se incrementa `ventana_trabajo.revision`, se regenera el plan de corte, se reinician los pasos solo si se pidió y se anota la auditoría. Un operario recibe 403/404 en esa acción aunque conozca la ruta.
- Cadena: presupuesto → trabajo → orden de taller → albarán → factura; rectificativa referencia a la original. Exigencias legales (numeración, inalterabilidad, conservación) frente a decisiones técnicas: las primeras se confirman con el gestor.

## 8. Migraciones y copias de seguridad
- **Migraciones** (ADR-0021, desde 0.11.0): archivos SQL numerados e incrustados; se aplican en una transacción; la versión es `PRAGMA user_version`. Antes de migrar se hace una copia automática. Si la base de datos es más nueva que el programa, **no arranca**.
- **Pruebas de migración:** instalación desde cero y desde bases de datos de versiones anteriores guardadas en `tests/bases_de_datos/`; después `integrity_check` y `foreign_key_check`.
- **Copias** (ADR-0022): a las 14:00, al cerrar y al arrancar si faltó; `VACUUM INTO` a un archivo temporal, `integrity_check` sobre la copia y traslado al disco con la carpeta `AlumFrame-copias`, donde se vuelve a comprobar. Retención: 7 diarias y 4 semanales (decisión del propietario). El estado de la última copia se guarda y se avisa en el Inicio. La restauración está probada en CI.

## 9. Seguridad
- Sesión: identificador aleatorio de 256 bits, cookie `HttpOnly`, `SameSite=Lax`, regenerado al iniciar sesión; caducidad por inactividad (2 h) y absoluta (12 h) **(confirmar)**.
- PIN del operario de 6 dígitos; bloqueo progresivo tras 5 fallos (1, 5 y 15 minutos) por usuario y por IP. Contraseña del dueño con `argon2id`.
- CSRF: token por sesión enviado en una cabecera de HTMX y verificado en toda petición que cambie datos.
- Permisos comprobados en `application` (`autorizar(actor, accion)`), no solo en las rutas.
- Con HTTP en una wifi compartida el PIN puede interceptarse: tablets en una red privada del taller; HTTPS cuando se use acceso remoto.

## 10. Errores y diagnóstico
- Tres familias: **de dominio** (corregibles por el usuario, 422, mensaje en español), **de conflicto** (409) y **internos** (500 con un identificador de correlación que se muestra al usuario y se registra).
- Registros con `tracing` en archivo rotado; sin datos personales ni PIN. Auditoría mínima: tabla solo de añadir con usuario, fecha UTC, acción, entidad e identificador, para sesiones, emisión de documentos y cambios de series, tarifas y configuración.

## 11. Estilo de código
- **Nombres en español que dicen qué hace cada cosa:** `evaluar_formula`, `expandir_ventana`, `optimizar_cortes`, `calcular_cuota_iva`, `emitir_factura`, `registrar_salida_de_stock`, `asignar_numero_de_documento`, `crear_copia_de_seguridad`, `restaurar_copia`.
- **Nombres que explican la intención;** si hace falta un comentario para entender qué hace una línea, se renombra o se divide.
- **Comentarios solo para el porqué:** una decisión no evidente, un invariante o una regla del taller. Sin comentarios que repitan el código, sin bloques largos, sin código comentado. `///` de una línea en tipos y funciones públicas del dominio. Un `TODO` debe apuntar a una pregunta de `docs/OPEN_QUESTIONS.md`.
- Sin estado global mutable: el estado compartido se pasa explícitamente (`EstadoApp`).
- `?` y errores tipados; `unwrap`, `expect` y `panic!` prohibidos fuera de tests.
- Límites de tamaño comprobados en CI (`just lint-size`) y `clippy` con umbral de complejidad cognitiva.

## 12. Rendimiento
Objetivos realistas, no optimización prematura: cálculo de una ventana en pocos milisegundos, optimización de 200 piezas por grupo en menos de 50 ms, respuestas de página por debajo de 100 ms en la red local. Se mide solo si algo falla.

## 13. Decisiones que dependen del taller
Ver `docs/OPEN_QUESTIONS.md`.

## 14. Modos de ejecución (local y servidor)
Hay un solo código y un solo ejecutable `alumframe`. La diferencia es de configuración, no de programa:

| | Modo servidor (principal) | Modo local |
|---|---|---|
| Escucha en | la red local (`0.0.0.0`, puerto configurable) | solo `127.0.0.1` (ese PC) |
| Arranque | servicio del sistema, siempre encendido | doble clic; abre el navegador solo |
| Datos | carpeta del servidor, copia nocturna | carpeta del usuario |
| Uso | oficina, tablet, varios usuarios | empezar, probar o un único puesto |

- Las pantallas, la base de datos y las reglas son idénticas. Los datos pasan de un modo a otro con Ajustes → Datos, exportando e importando (ADR-0016).
- En Windows el modo local lo usa la **aplicación de escritorio** (`alumframe-escritorio`, ADR-0015): ventana propia sin consola, que también puede conectarse al servidor del taller en lugar de arrancar el modo local.
- La autenticación está siempre activa. HTTPS solo hace falta si hay acceso remoto.
- **Un solo proceso dueño del archivo de datos.** Nunca se ejecuta el programa en varios PCs sobre un archivo compartido por carpeta de red: SQLite en modo WAL no es seguro por red y puede corromperse.
- Configuración: dirección, puerto, carpeta de datos y si abre el navegador. El formato del archivo se decide en el ADR de H0.
- No se incluye ventana propia (webview): añade una dependencia y el navegador es suficiente.
- Los E2E arrancan el ejecutable real en modo local.
