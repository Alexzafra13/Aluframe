# ADR-0029: dibujos de herrajes, modelo en el catálogo y logotipo de la marca

**Estado:** aceptado (2026-11-24, petición del propietario).

## Contexto
El propietario quiere ver los herrajes (cierres embutidos, kit de plásticos, ruedas, tornillos) aparte de los perfiles a cortar y con un dibujo, como las siluetas de los perfiles (ADR-0017). El cierre embutido que usa hoy es el **Alualpha 267 i**, pero puede cambiar de marca, así que la serie no debe atarse a ese modelo. También quiere que la ventana diga qué es («Ventana corredera») y de qué serie y marca, con el logotipo de Aluval.

## Decisión
1. **Dibujos técnicos de herrajes** en `data/perfiles/herrajes/`, sacados de los PDF que aportó el propietario con la máxima calidad que traen. El cierre embutido es la vista de frente del Alualpha 267 i, **en vector** (la ficha de Alualpha es vectorial), extraída con `scripts/extraer_herraje.py`, sin cotas y en los colores del diseño; va tumbado para aprovechar el hueco apaisado de la lista. El kit de plásticos (9426065) y la rueda fija (9101027) son las **imágenes originales** del catálogo de Aluprom 21, que no los trae en vector, extraídas sin pérdida (`pdfimages`) y puestas en tinta sobre fondo transparente dentro de un SVG con `scripts/imagen_de_herraje.py`, que refuerza las líneas finas. Las líneas del cierre se engordan (`--grosor-minimo`) para que no desaparezcan a 56 × 42 px. Se citan en la serie con el mismo campo `silueta` de `referencias`, así que no hace falta código nuevo para enlazarlos. Al principio se dibujaron a mano, pero el propietario prefirió sus dibujos técnicos (2026-11-24). El tornillo (4,8 × 25, cabeza redonda de estrella) usa un dibujo técnico que pasó el propietario aparte. Un herraje sin dibujo técnico se queda sin dibujo, para no mezclar estilos.
2. **La serie dice el herraje genérico; el catálogo, el modelo.** La serie usa referencias del taller (`CIERRE-EMBUTIDO`, `KIT-CORREDERA`…). El artículo del catálogo con esa referencia es lo que se compra y se descuenta del stock, y su descripción («Cierre de embutir Alualpha 267 i») se muestra bajo el nombre del herraje. Cambiar de modelo es editar ese artículo, sin tocar la serie.
3. **Logotipos de los fabricantes** en `data/marcas/<fabricante>.png`, dentro del ejecutable y servidos en `/marcas/…` (solo `.png` de esa carpeta). El de Aluval sale de la imagen que aportó el propietario, recortada y con el fondo transparente. Si un fabricante no tiene logotipo, se escribe su nombre.
4. **Ficha de la ventana en el editor:** en la cabecera del dibujo, el tipo («Ventana corredera 4 hojas») como título y debajo el logotipo con la serie («Aluprom 21»). En el desplegable, la serie por su nombre y su fabricante («Aluprom 21 (Aluval) · Ventana corredera 2 hojas») en lugar del código interno.

## Consecuencias
- Para que el cierre tenga precio y se descuente del stock hay que dar de alta el artículo `CIERRE-EMBUTIDO` (y `KIT-CORREDERA`) en Catálogo > Artículos, con la descripción del modelo.
- El kit y la rueda son imágenes de poca resolución (unos 150 px), porque así vienen en el catálogo de Aluval; se ven bien al tamaño de la lista. Si el fabricante da una ficha en vector, se sacan con `scripts/extraer_herraje.py` sin tocar la serie.
- La hoja de taller impresa conserva los nombres genéricos: los documentos emitidos no releen el catálogo.
- El logotipo es una imagen de baja resolución (191 × 41 px); se muestra a 16 px de alto, donde se ve nítido. Si el fabricante da un logotipo vectorial, se sustituye el archivo.

## Alternativas descartadas
- **Poner el modelo (Alualpha 267 i) en la serie:** obligaría a editar la serie cada vez que se compre otra marca, y la serie es del fabricante de perfiles, no del de herrajes.
- **Dibujos hechos a mano:** el propietario prefiere los técnicos del catálogo, que se reconocen mejor.
- **Un logotipo dibujado en CSS o con texto:** no se reconocería como la marca, que es lo que pide el propietario.
