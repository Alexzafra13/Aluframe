# ADR-0018: el perfil de una pieza puede cambiar con una opción de la línea

**Estado:** aceptado (2026-11-22, petición del propietario).

## Contexto
En Aluprom 21 las hojas existen en versión para cristal sencillo (21310, 21340, 21350) y para doble acristalamiento Climalit (21312, 21341, 21351), con las mismas medidas de corte. El propietario quiere elegir el acristalamiento al hacer la ventana y que salgan las hojas que tocan, y dejarlo preparado para concretar más adelante el tipo de Climalit y su cámara.

## Decisión
- Campo opcional `perfil_segun` en cada pieza: `{ opcion: { valor: CODIGO } }`. Con ese valor elegido, la pieza usa `CODIGO`; con cualquier otro, su `perfil`. Lo que no se elige en la línea toma el valor por defecto de la opción.
- Se resuelve antes de calcular (`Tipologia::con_opciones`), así que referencias, precio, plan de corte, silueta y pendientes usan ya el perfil elegido. La numeración y el dibujo no cambian.
- Al importar se comprueba que la opción y el valor existen en la tipología y que el código está en `referencias`.
- Aluprom 21 añade en sus dos correderas la opción `acristalamiento` (`sencillo` por defecto, o `climalit`) y las hojas `*_CLIMALIT`.

## Por qué así
- **No toca el evaluador de fórmulas:** las opciones siguen sin entrar en las cuentas (`FORMATO_SERIES.md`, apartado 4). Elegir un perfil es un dato, no una fórmula.
- **Compatible:** las series sin `perfil_segun` y las líneas guardadas antes calculan igual (valor por defecto).
- **Crece sin código:** más valores (tipos de Climalit, cámaras) o más opciones se añaden en el YAML.

## Descartado
- **Listar las dos versiones de cada pieza con una condición de «incluir si»:** duplica piezas y complica la numeración.
- **Usar la opción dentro de las fórmulas:** abre el evaluador a textos para resolver algo que es una elección de perfil.

## Pendiente
- Si el Climalit cambia los descuentos de las hojas o del cristal (hoy se suponen iguales) y cómo se piden su tipo y su cámara: `docs/OPEN_QUESTIONS.md`.
