# Diseño: tokens y componentes (versión 1)

Referencia visual: `design/` (aprobado). Implementación: `crates/servidor/recursos/estilos.css`. Las medidas del diseño están en mm; la interfaz las muestra en **cm con un decimal**.

## Colores
| Token CSS | Valor | Uso |
|---|---|---|
| `--papel` | `#ECE9E0` | Fondo general |
| `--superficie` | `#FBFAF6` | Tarjetas, panel de atención |
| `--columna` | `#E3DFD4` | Columnas del tablero |
| `--tinta` | `#15191D` | Texto principal |
| `--apagado` | `#5C636B` | Texto secundario, antetítulos |
| `--linea` | `#D5D1C6` | Bordes y separadores |
| `--acero` / `--acero-tinte` | `#1F4E79` / `#E3EDF6` | Acento principal: botones, enlaces, confirmaciones |
| `--aviso` / `--aviso-fondo` | `#8A3207` / `#F8E6DA` | Avisos y errores ("Falta el cristal") |
| `--indicador` | `#E8702A` | Barra de la entrada activa del menú y foco visible |
| `--menu-fondo` / `--menu-activo` | `#15191D` / `#252A30` | Menú lateral |

Solo azul acero y naranja como acentos.

## Tipografía (incluida en el ejecutable)
- **IBM Plex Sans** 400, 500 y 600: texto.
- **IBM Plex Sans Condensed** 500 y 600: títulos, antetítulos y etiquetas.
- **IBM Plex Mono** 400 y 500: medidas, códigos y teclado del PIN.

Subconjunto Latin-1 en woff2 (licencia OFL en `recursos/fuentes/LICENCIA-OFL.txt`).

## Componentes
- **Menú lateral** de 280 px con grupos Vender, Taller y Cobrar; Ajustes y Salir al pie. Entrada activa con fondo `--menu-activo` y barra naranja a la izquierda.
- **Barra superior:** buscador (activo desde H2) y usuario.
- **Cabecera de página:** antetítulo en mayúsculas espaciadas y título en Condensed 36 px.
- **Tablero:** tres columnas y panel de atención.
- **Botones:** al menos 44 px de alto; primario relleno en acero y secundario con borde.
- **Pantallas de acceso y del operario:** tarjetas de nombre de 64 px y teclado numérico con teclas de 64 px.
- **Estados vacíos:** cada lista explica qué hacer o cuándo aparecerá algo.
- **Formularios (H2):** rejilla de campos de 240 px mínimo; error junto al campo en `--aviso` con el campo en `--aviso-fondo`; confirmación correcta (NIF) en acero. Ningún botón desactivado sin un texto al lado que diga por qué.
- **Asistente de importación (H2):** pasos 1-3 con el paso escrito; resumen en una línea (nuevos, cambios, sin cambios, ausentes); filas con salto de precio sobre `--aviso-fondo` y diferencia en naranja (sube) o acero (baja).
- **Tablas editables (H2):** importes en Plex Mono alineados a la derecha; casillas de 24 px dentro de objetivos de 44 px.
- **Zona de peligro:** acciones irreversibles dentro de un desplegable, con casilla de confirmación y botón con borde de aviso.

## Menú lateral (0.7.2, petición del propietario)
- Se queda fijo al desplazar la página (altura de la pantalla, con su propio desplazamiento si no cabe).
- Se pliega a solo iconos con «Plegar menú» (abajo): las columnas se animan (360 ms, curva suave), los textos se desvanecen y el icono gira. La preferencia se guarda en el navegador (`menu.js`, cargado antes de pintar para que no parpadee). Cada entrada lleva su nombre como `title`.

## Movimiento (inspirado en iOS, decisión del propietario del 2026-11-21)
Objetivo: que la interfaz se sienta fluida y responda al tacto, sin adornos ni efectos gratuitos y sin coste de rendimiento. Solo se animan `transform` y `opacity`, que el navegador resuelve en la tarjeta gráfica.

| Elemento | Comportamiento |
|---|---|
| Cambio de pantalla | Transiciones de vista nativas del navegador (`@view-transition`): el contenido entra con un fundido y 6 px de subida (260 ms); el menú y la barra superior no se mueven. Navegadores sin soporte cambian de página sin animación |
| Indicador del menú | La barra naranja se desliza de la entrada anterior a la nueva (`view-transition-name: indicador-menu`, 380 ms) |
| Botones y tarjetas táctiles | Se hunden al 96 % al pulsar y vuelven con un pequeño rebote (curva `--curva-muelle`) |
| Barra superior | Fija y translúcida, con desenfoque (`backdrop-filter`) |
| Avisos y confirmaciones | Aparecen bajando 6 px con un fundido |
| Resultados que cambian con HTMX | Se atenúan mientras se calculan y el nuevo resultado aparece con un fundido |
| PIN del operario | Teclado redondo de 76 px como el del iPhone, puntos que se rellenan, entrada automática al sexto dígito y sacudida si el PIN es incorrecto; no abre el teclado del sistema en la tablet |
| Lista de usuarios | Inicial en un círculo y flecha «›», como las listas de iOS |
| Hojas (paneles) | Formularios cortos y confirmaciones salen en una **hoja** con `<dialog>` nativo: sube desde abajo en tablet y entra desde la derecha en ordenador (320 ms, `--curva-suave`) sobre un velo que se oscurece; se cierra con Esc, con «Cancelar» o tocando el velo, y baja con la misma curva. El foco entra en la hoja y vuelve al botón que la abrió (lo hace el navegador). Sin JavaScript propio salvo abrir y cerrar |
| Control segmentado | Como el de iOS (Cotas, Piezas, Cristal): una «píldora» blanca se desliza bajo la opción elegida |
| Avisos efímeros | Las confirmaciones («Línea guardada») bajan desde la barra superior como en iOS, se quedan 4 s y se van; los errores se quedan hasta que se corrigen. Lectores de pantalla: `role="status"` |
| Totales | Al recalcular, el importe nuevo aparece con un fundido corto; mientras calcula, el pie se atenúa |
| Tarjetas de línea | Entran escalonadas (40 ms entre una y otra) al abrir el presupuesto; una línea nueva aparece desde abajo |

Curvas: `--curva-suave` `cubic-bezier(.2, .8, .2, 1)` y `--curva-muelle` `cubic-bezier(.34, 1.4, .64, 1)`. Duraciones: 160 ms (respuesta) y 260 ms (aparición).
**Accesibilidad:** con «reducir movimiento» activado en el sistema, se desactivan todas las animaciones.
**Lo que no se hace:** degradados decorativos, sombras grandes, cristal esmerilado por todas partes, emojis o animaciones que no comuniquen nada.

## Móvil y tablet (0.5.1, petición del propietario: «estética iPhone»)
Implementación: `crates/servidor/recursos/movil.css` e iconos en `plantillas/iconos.html`. Mismos tokens, tipografía y acentos; por debajo de 900 px solo cambia la disposición.

| Elemento | Comportamiento |
|---|---|
| Iconos | Juego propio de trazo fino (1,7 px, extremos redondeados) pensado para el taller: Trabajos es la ventana del logotipo, Stock son barras de perfil apiladas. También en el menú lateral del ordenador |
| Barra de pestañas | Fija abajo y translúcida (desenfoque), con Inicio, Clientes, Presupuestos, Trabajos y **Más**. Activa en acero con trazo más grueso; la barra naranja del indicador se desliza entre pestañas con la misma transición que en el menú lateral. Respeta la zona inferior del iPhone (`safe-area`) |
| «Más» | Hoja que sube desde abajo con el asa de iOS y una lista agrupada (Stock, Catálogo, Facturas, Ajustes) y Salir aparte, en el color de aviso |
| Barra superior | Logotipo, buscador con lupa como el campo de búsqueda de iOS y la inicial del usuario |
| Tablas | Por debajo de 700 px cada fila es una celda: el primer dato es el título con «›», el resto debajo en texto apagado y, si hace falta, con su etiqueta (`data-etiqueta`) |
| Filtros y pestañas | Si no caben, se deslizan de lado sin barra de desplazamiento |
| Pie del total | En el editor y el presupuesto ocupa el sitio de la barra de pestañas, de borde a borde |
| Hojas | Tan altas como su contenido y de borde a borde |

**Prueba:** `e2e/pruebas/movil.spec.js` recorre todas las pantallas en iPhone y iPad y falla si alguna se sale por los lados.

**Lista de piezas (0.5.4, petición del propietario):** dibujo del perfil (56 px) con el número de la pieza pequeño y gris debajo, nombre y corte, cantidad «×2» en negrita y medida. El número sigue siendo el mismo que el del dibujo de la ventana, pero discreto para que no se confunda con la cantidad.

## Menú lateral (0.11.2, petición del propietario)
- Cabecera: marca y, a la derecha, el botón de plegar **solo con icono** (44 × 44 px, nombre accesible «Plegar menú» / «Desplegar menú»). Plegado, ocupa el sitio de la marca.
- Pie: Ajustes y una tarjeta con avatar, nombre y rol; **Salir** es un botón con icono en rojo `--peligro: #F2555A` (fondo al pasar `rgb(242 85 90 / .14)`). Excepción pedida por el propietario a la regla de solo azul y naranja: el rojo se reserva para salir y acciones destructivas. En la hoja «Más» del móvil, Salir va en `#D93D42` sobre blanco.
- Con menú lateral, el nombre de quien ha entrado ya no se repite en la barra superior (sí en el móvil).
