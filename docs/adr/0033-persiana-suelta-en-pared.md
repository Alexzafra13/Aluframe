# ADR-0033: persiana suelta en pared con guías de L; producto que se pide hecho

**Estado:** aceptado (2026-11-24, planteamiento aprobado por el propietario).

## Contexto
Además de la persiana en la ventana (ADR-0032), el taller vende **persianas sueltas** para montar en un hueco de la pared, con **guías de L** (perfil plano) atornilladas a la pared. No se fabrican: **se piden hechas**. El propietario quiere añadirlas al presupuesto con su dibujo y su precio, solas o con la ventana, para llevar pedidos y precios. Datos del propietario:
- La medida que se escribe es **la del hueco**.
- La guía de L tiene un canal de 53 mm y un ala que no es la de 60 mm del plano: **80 o 100 mm, por confirmar**.
- Plano de referencia del propietario: cajón por fuera encima del hueco, guías de L a los lados con tornillos, lamas y lama final con asa.

## Decisión
1. **Serie nueva `persiana-pared.yaml`** con la tipología «Persiana en pared», en borrador. Se elige en «Serie y tipología» como una ventana; el botón del presupuesto pasa a «+ Ventana, mosquitera o persiana».
2. **Sin perfiles a cortar** (`piezas: []`): no entra en el plan de corte ni en el stock de barras. Una serie sin piezas no necesita longitud de barra. La persiana va en `accesorios` (`PERSIANA`, uno por línea) con la referencia en `CONFIRMAR` hasta saber cómo se pide y se cobra.
3. **El bloque `persiana` gana `montaje: ventana | pared`.** En pared la medida es el hueco y no se le quita nada; la guía (el ala de la L) y el cajón pueden quedar en `CONFIRMAR`, y entonces la persiana sale incompleta, sin precio final.
4. **Dibujo `tipo: persiana`**, como el plano del propietario: guías y cajón por fuera del hueco, en el color elegido; lamas con relieve en todo el hueco; lama final con el asa; dos tornillos por guía; la cinta por la guía del lado elegido con su recogedor. Mientras falten la guía o el cajón se dibujan aproximados (80 mm y el 15 % del alto) y **no se acotan**: solo se acota el hueco («hueco 120»). Con los dos datos se acota además el total.

## Consecuencias
- Cuando el propietario dé el ala de la guía, el cajón y el precio, se rellenan en la serie sin tocar código.
- Los pedidos a proveedor siguen fuera del MVP; la persiana queda en el presupuesto y en el trabajo como artículo a pedir.

## Alternativas descartadas
- **«Otro concepto» con precio a mano:** sin dibujo ni medida, y no deja calcular el precio cuando llegue la tarifa.
- **Un producto con formulario propio:** duplicaría presupuesto, precio y trabajo.
