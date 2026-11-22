# ADR-0019: colores de los acabados y ventana dibujada en su color

**Estado:** aceptado (2026-11-22, petición del propietario).

## Contexto
La ventana se dibujaba siempre en gris. El propietario quiere verla en el color elegido (blanco por defecto), con los colores que más se usan (lisos con su RAL e imitaciones de madera), y que los cierres salgan blancos si la ventana es blanca y negros si es de color, como se montan en el taller. El precio de cada color respecto al blanco se trabajará después.

## Decisión
1. **Los colores vienen con el programa**, en `data/acabados.yaml` (nombre, RAL, color del dibujo, si es madera, color de los cierres y cuál es la base), y se cargan al arrancar como las series (ADR-0014). Si un acabado no existe se crea; si existe se le ponen el RAL, el color, la veta y los cierres del archivo. **El recargo y el archivado son del propietario** y no se tocan. Las correcciones las hacemos en el repositorio.
2. **La tabla `acabado` guarda `ral`, `color_dibujo`, `madera` y `cierres`.** Cambia el esquema: hasta el primer despliegue real se borra la base de pruebas al actualizar (ADR-0011), con permiso del propietario.
3. **El dibujo lleva el color en atributos `fill` y `stroke`**, no en `style`: la política de seguridad de la página (CSP) no deja aplicar estilos escritos en el HTML. Las hojas salen un punto más claras que el marco y los bordes son el mismo tono oscurecido (se calcula en el dominio). La imitación madera usa un patrón de vetas. Los cierres, manillas y bisagras van en un grupo con el color de los herrajes.
4. **Sin color elegido, la ventana es blanca (RAL 9016) con cierres blancos.** Los dibujos guardados antes, que no llevan colores, siguen viéndose en gris.

## Requisitos del ADR-0002
- Sin dependencias nuevas.
- Pruebas:
  - dominio: color por defecto, color liso con cierres negros, veta en marco y hojas, mezcla de tonos y lectura del archivo;
  - integración: el archivo del programa es válido, se carga una vez sin tocar el recargo y la ventana se dibuja en el color elegido.

## Pendiente
- Confirmar los RAL (se pusieron los estándar del sector: 9016, 9005, 6005, 7016, 8014) y el nombre «Embero». Recargo de cada color. Editar el color de un acabado creado a mano desde la pantalla.
