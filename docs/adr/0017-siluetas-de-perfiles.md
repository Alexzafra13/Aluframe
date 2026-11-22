# ADR-0017: siluetas de los perfiles junto a cada pieza

**Estado:** aceptado (2026-11-22, petición del propietario).

## Contexto
En la lista de piezas solo había un número y un nombre («Hoja cierre»). El propietario quiere ver junto al número el dibujo de la sección del perfil, como en el catálogo del fabricante, para saber de un vistazo qué pieza es. Aportó el catálogo de Aluprom 21 de Aluval en PDF.

## Decisión
1. **Las siluetas son datos**, en `data/perfiles/<fabricante>/<referencia>.svg`, con un `indice.txt` de nombres. Van dentro del ejecutable como las series (ADR-0014) y se sirven en `/perfiles/…` (solo `.svg` de esa carpeta, con caché de un día).
2. **Se sacan del PDF del fabricante con `scripts/extraer_siluetas.py`** (herramienta de desarrollo, Python y poppler). El catálogo es vectorial: se toman los rellenos grises del perfil (sin cotas ni textos), se agrupan por dibujo y se nombran con el texto de debajo. La referencia de 5 cifras de Aluval está dibujada con trazos, no es texto: se asigna a mano comprobándola en el catálogo. Se guardan como un único trazo en el color de tinta del diseño.
3. **La serie enlaza cada código con su silueta** con el campo opcional `silueta` de `referencias`, separado de `referencia`: el dibujo ayuda a reconocer la pieza pero **no fija el precio**, así que puede ponerse aunque la referencia de precio siga en `CONFIRMAR`.
4. **Posición en el dibujo declarable** (`posicion` en cada pieza). Al usar los nombres del catálogo, «HOJA_CIERRE» es la hoja del centro, y la regla por nombre la habría puesto en el lateral.

## Consecuencias
- Aluprom 21 usa los nombres del catálogo (decisión del propietario): `HOJA_LATERAL` (antes «hoja de cierre») y `HOJA_CIERRE` (antes «hoja central»). Las medidas no cambian. En los fixtures solo se cambió el nombre del perfil, con permiso del propietario.
- Las 31 secciones de las págs. 4 a 6 del catálogo quedan disponibles; las que aún no se usan esperan a nuevas tipologías.
- Lo pendiente queda en `docs/OPEN_QUESTIONS.md`: cuál es el marco inferior (21230 o 21231) y cómo modelar las hojas según el cristal (sencillo o Climalit).

## Alternativas descartadas
- **Recortar imágenes del PDF:** pesan más, se ven borrosas al ampliar y no siguen los colores del diseño.
- **Dibujar los perfiles a mano en el código:** inexacto y no escala a otras series.
- **Guardar la silueta dentro del YAML de la serie:** varias series comparten perfiles, y los dibujos inflarían un archivo que se edita a mano.
