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

Curvas: `--curva-suave` `cubic-bezier(.2, .8, .2, 1)` y `--curva-muelle` `cubic-bezier(.34, 1.4, .64, 1)`. Duraciones: 160 ms (respuesta) y 260 ms (aparición).
**Accesibilidad:** con «reducir movimiento» activado en el sistema, se desactivan todas las animaciones.
**Lo que no se hace:** degradados decorativos, sombras grandes, cristal esmerilado por todas partes, emojis o animaciones que no comuniquen nada.
