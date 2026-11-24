# ADR-0029: dibujos de herrajes, modelo en el catálogo y logotipo de la marca

**Estado:** aceptado (2026-11-24, petición del propietario).

## Contexto
El propietario quiere ver los herrajes (cierres embutidos, kit de plásticos, ruedas, tornillos) aparte de los perfiles a cortar y con un dibujo, como las siluetas de los perfiles (ADR-0017). El cierre embutido que usa hoy es el **Alualpha 267 i**, pero puede cambiar de marca, así que la serie no debe atarse a ese modelo. También quiere que la ventana diga qué es («Ventana corredera») y de qué serie y marca, con el logotipo de Aluval.

## Decisión
1. **Dibujos propios y genéricos** de herrajes en `data/perfiles/herrajes/` (SVG hechos a mano en el estilo de las siluetas, no copiados de un fabricante), citados en la serie con el mismo campo `silueta` de `referencias`. No hace falta código nuevo para enlazarlos.
2. **La serie dice el herraje genérico; el catálogo, el modelo.** La serie usa referencias del taller (`CIERRE-EMBUTIDO`, `KIT-CORREDERA`…). El artículo del catálogo con esa referencia es lo que se compra y se descuenta del stock, y su descripción («Cierre de embutir Alualpha 267 i») se muestra bajo el nombre del herraje. Cambiar de modelo es editar ese artículo, sin tocar la serie.
3. **Logotipos de los fabricantes** en `data/marcas/<fabricante>.png`, dentro del ejecutable y servidos en `/marcas/…` (solo `.png` de esa carpeta). El de Aluval sale de la imagen que aportó el propietario, recortada y con el fondo transparente. Si un fabricante no tiene logotipo, se escribe su nombre.
4. **Ficha de la ventana en el editor:** en la cabecera del dibujo, el tipo («Ventana corredera 4 hojas») como título y debajo el logotipo con la serie («Aluprom 21»). En el desplegable, la serie por su nombre y su fabricante («Aluprom 21 (Aluval) · Ventana corredera 2 hojas») en lugar del código interno.

## Consecuencias
- Para que el cierre tenga precio y se descuente del stock hay que dar de alta el artículo `CIERRE-EMBUTIDO` (y `KIT-CORREDERA`) en Catálogo > Artículos, con la descripción del modelo.
- La hoja de taller impresa conserva los nombres genéricos: los documentos emitidos no releen el catálogo.
- El logotipo es una imagen de baja resolución (191 × 41 px); se muestra a 16 px de alto, donde se ve nítido. Si el fabricante da un logotipo vectorial, se sustituye el archivo.

## Alternativas descartadas
- **Poner el modelo (Alualpha 267 i) en la serie:** obligaría a editar la serie cada vez que se compre otra marca, y la serie es del fabricante de perfiles, no del de herrajes.
- **Copiar el dibujo del catálogo de Alualpha:** es de una marca concreta; un dibujo genérico vale para cualquier cierre de embutir.
- **Un logotipo dibujado en CSS o con texto:** no se reconocería como la marca, que es lo que pide el propietario.
