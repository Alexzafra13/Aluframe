# ADR-0032: persiana en la ventana; la medida escrita es la del conjunto

**Estado:** aceptado (2026-11-24, planteamiento aprobado por el propietario).

## Contexto
El propietario quiere ofrecer las correderas **con o sin persiana**. Las persianas no se fabrican en el taller: se piden. De momento solo interesa la persiana **montada en la ventana**, con el cajón encima (normalmente recibido en obra, aunque puede ir atornillado). Datos del propietario:
- La medida que se escribe es siempre **la de fuera, la del conjunto**, la llame el cliente «ventana» o «hueco». Sin persiana es la del marco, como hasta ahora.
- Con persiana, cada guía lateral ocupa **4 cm**: el marco tiene **8 cm menos de ancho**.
- El alto del marco es el del conjunto **menos el cajón**. El cajón es más o menos alto (más o menos lamas) según el alto: las alturas y a partir de qué medida va cada uno los dará el propietario.
- La **cinta** va a la derecha por defecto y se puede poner a la izquierda.

## Decisión
1. **Opción de línea** «Persiana: sí / no» en las correderas de Aluprom 21 (2 y 4 hojas) y, solo con persiana, «Cinta: derecha / izquierda».
2. **Bloque `persiana`** en la tipología (`solo_si`, `guia`, `cajon`, `cinta`). Con él, las fórmulas de las piezas, el cristal, las chapas y los accesorios ven en `ancho` y `alto` la medida **del marco** (conjunto − 2 guías, conjunto − cajón). `guia` y `cajon` se evalúan con la medida del conjunto, así el cajón podrá depender del alto con `si()`.
3. **Sin el alto del cajón** (`cajon: CONFIRMAR`), `alto` se queda sin valor: lo que dependa de él (laterales, hojas verticales, cristal) sale **pendiente** con su número, igual que el parteluces (ADR-0030), y la ventana queda incompleta, sin precio final. Lo que solo depende del ancho se calcula ya. Esto es general: cualquier fórmula con un nombre sin valor queda pendiente en vez de inventar la medida.
4. **La persiana se pide, no se corta:** va en `accesorios` (`PERSIANA`, uno por ventana, con referencia en `CONFIRMAR` hasta saber cómo se pide y cómo se cobra).
5. **Dibujo**, a partir de la foto del propietario: el cajón arriba a todo el ancho y una guía a cada lado, en el color de la ventana; la persiana medio bajada, con las lamas detrás del cristal; la cinta y el recogedor por fuera, en el lado elegido. Cotas: fuera, la medida del conjunto; dentro, «marco 112» (y el alto del marco a la derecha cuando se sepa). Sin el alto del cajón, el cajón se dibuja aproximado y no se acota.
6. La medida guardada en la línea sigue siendo la escrita (la del conjunto); el marco se deduce al calcular.

## Consecuencias
- Sin persiana nada cambia: los fixtures reales dan lo mismo.
- Cuando el propietario dé los cajones, se escribe en `cajon` (por ejemplo `si(alto <= 140, 17, 20)`) sin tocar código.
- **Más adelante, aparte:** la persiana suelta para montar en pared, con sus otras guías (producto propio, como la mosquitera), y la mosquitera dentro de la ventana con persiana, cuando el propietario dé su descuento (ADR-0031).

## Alternativas descartadas
- **Escribir la medida del marco y sumar la persiana:** el propietario se fija siempre en la medida de fuera.
- **Restar en cada fórmula de la serie** (`ancho - 8`): habría que duplicar cada pieza con y sin persiana.
- **Dibujar con un alto de cajón inventado y calcular con él:** daría medidas de corte falsas.
