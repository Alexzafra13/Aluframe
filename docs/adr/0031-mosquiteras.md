# ADR-0031: mosquitera enrollable como serie; piezas según la medida y tipologías sin cristal

**Estado:** aceptado (2026-11-24, planteamiento aprobado por el propietario).

## Contexto
El propietario quiere presupuestar **mosquiteras enrollables de aluminio** como un producto más: al escribir la medida sale el dibujo, con los mismos colores que las ventanas. Ahora se venden sueltas; más adelante irán dentro de las ventanas con persiana. Lleva un cajón de 45 mm hasta 140 cm de alto y de 50 mm por encima, dos guías laterales, un terminal con tirador y un kit. Datos del propietario: guía con cajón de 50 = alto − 5; ancho − 2 cm; falta el descuento de la guía con cajón de 45, las referencias, el proveedor y la longitud de barra.

## Decisión
1. **La mosquitera es una serie** (`data/series/mosquitera-enrollable.yaml`), con su tipología. Se presupuesta, se acepta, se corta y se descuenta del stock como una ventana, sin código propio fuera del dibujo. Va **en borrador**: se dibuja y se calcula, pero no tiene precio final hasta confirmar los datos.
2. **Cantidad 0 = la pieza no va.** Con `si()` una serie elige entre dos perfiles según la medida: `cantidad: "si(alto <= 140, 1, 0)"` para el cajón de 45 y la contraria para el de 50. Las piezas se numeran entre las que lleva esa medida (también las pendientes), así no hay saltos.
3. **`vidrio` es opcional:** sin él la tipología no lleva cristal y no se pide.
4. **Dibujo `tipo: mosquitera`:** cajón arriba (más alto si sale el perfil del cajón de 50), guías, malla con trama, terminal con tirador y tapas, en el color del acabado. Los números de pieza van con `posicion` (`marco_superior`, `marco_lateral`, `marco_inferior`). Dibujo propio a partir de la imagen que aportó el propietario.
5. **Al importar** se calcula también a 200 cm de alto, para probar las piezas que solo lleva una medida alta.
6. **Interfaz:** «+ Ventana o mosquitera»; en el editor, «Nueva», «+ Otra» y «Guardar».
7. Los nombres de pantalla recuperan la tilde de las palabras del taller que la llevan (`CAJON` → «Cajón», `GUIA` → «Guía»), porque los códigos van sin tildes.

## Consecuencias
- Cuando el propietario dé el descuento de la guía con cajón de 45, las referencias, el proveedor y la longitud de barra, se rellenan en la serie y se pasa a `validada`.
- **Más adelante:** la ventana con persiana podrá llevar una mosquitera; se decidirá entonces si como pieza de la tipología (con opción «Mosquitera: sí/no») o como línea enlazada.

## Alternativas descartadas
- **Un producto aparte con su propio formulario:** duplicaría cálculo, precio, corte y stock.
- **Que se elija el cajón a mano:** el propietario dio la regla (hasta 140, de 45), así que se aplica sola.
