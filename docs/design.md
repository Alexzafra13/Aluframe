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
