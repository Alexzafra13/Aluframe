# ADR-0030: parteluces, opciones de medida, partes condicionales y chapas

**Estado:** aceptado (2026-11-24, petición y planteamiento aprobados por el propietario).

## Contexto
Una corredera puede llevar **parteluces**: un travesaño horizontal en cada hoja (Aluprom 21: 21564, o 21565 con Climalit) con chapa abajo y cristal arriba, o dos cristales. Quien presupuesta escribe la medida de la chapa (o del cristal) de abajo; 50 cm es lo habitual porque se aprovecha la plancha, pero puede ser cualquiera. La chapa es del color de la ventana: con cristal normal, una por hoja lacada por las dos caras; con Climalit, dos por hoja, de preferencia lacadas por una cara. Faltan datos del propietario: la medida de corte del parteluces, su altura real, el ancho de la chapa y cómo se cobra. El formato de series solo admitía opciones de lista, que no podían cambiar medidas ni qué piezas lleva la ventana.

## Decisión
Se amplía el formato de series de forma **general**, no como un caso especial del parteluces, porque servirá también para zócalos, puertas con chapa o fijos partidos:
1. **Opciones de medida** (`tipo: medida`, con `por_defecto`, `min` y `max`): se escriben en la línea y son variables de las fórmulas de su tipología. Se guardan con las demás opciones de la línea (texto), así que no hace falta migración.
2. **`solo_si: { opcion: valor }`** en opciones, piezas, accesorios, piezas de vidrio, chapas y parteluces: la parte solo existe, y la opción solo se pregunta, si se cumplen las condiciones. Una medida que no se pregunta usa el valor de la serie, aunque quede algo escrito en el campo oculto.
3. **Cristal partido:** cada pieza de vidrio puede llevar `nombre` («Arriba», «Abajo») y una medida en `CONFIRMAR` (esa pieza no sale y la ventana queda incompleta).
4. **`chapas`**: paneles opacos con artículo, cantidad, ancho y alto (estos dos admiten `CONFIRMAR` y se muestran como «?»). Se dibujan **a escala con sus cotas** (dibujo técnico) en el editor, en «Mi día» y en la hoja de taller. **Mientras el propietario no decida cómo se cobra, una ventana con chapa no tiene precio final.**
5. **`parteluces`** solo da el dibujo: una barra del color de la ventana en cada hoja, la chapa rayada en el color de la ventana (o el cristal) y una cota a la derecha con la altura real del parteluces. Sin la altura real (`CONFIRMAR`), la barra se coloca aproximada y la cota dice «chapa 50».
6. **En pantalla**, las opciones que dependen de otra se muestran u ocultan en el navegador (`recursos/js/opciones.js`) sin volver a pedir la página: así no se pierde el cursor mientras se escribe la medida.
7. El resumen guardado de cada línea añade las chapas y el nombre del cristal con valores por defecto, para que los resúmenes anteriores se sigan leyendo.

## Consecuencias
- Sin parteluces nada cambia: los fixtures reales dan lo mismo.
- Lo que falta queda en `CONFIRMAR` y en `docs/OPEN_QUESTIONS.md`. Cuando llegue cada dato, se rellena en `data/series/aluprom-21.yaml` sin tocar código.
- **Fase 2** (pendiente de plan): la chapa como material en el catálogo y el stock, con color (los mismos RAL que el aluminio), lacada por una o dos caras y planchas de 200 × 125 y 300 × 125 cm (por confirmar). Necesita una migración nueva que conserve los datos. **Fase 3:** aprovechar los recortes de chapa con un optimizador en dos dimensiones.

## Alternativas descartadas
- **Una tipología aparte «corredera con parteluces»:** duplicaría la serie y cada cambio habría que hacerlo dos veces.
- **Pedir la altura del parteluces en vez de la de la chapa:** el propietario corta la chapa a una medida (50 cm) y quiere ver, a partir de ella, dónde queda el parteluces.
- **Volver a pintar las opciones desde el servidor en cada cálculo:** se perdería el foco del campo mientras se escribe.
