# ADR-0035: material a pedir por trabajo; el presupuesto con dibujo limpio

**Estado:** aceptado (2026-11-25, planteamiento aprobado por el propietario).

## Contexto
El cristal y la persiana no se fabrican ni salen del stock: se encargan a medida para cada trabajo. Hasta ahora el cristal solo se veía sobre el dibujo y había una casilla «cristal recibido» por ventana; la persiana salía entre los herrajes. El propietario quiere:
- En el presupuesto, el dibujo de la ventana entera con su medida total, sin flechas, sin números de pieza, sin la cota del marco y sin los botones «Cotas · Piezas · Cristal». Debajo, la lista: perfiles a cortar, lo que hay que pedir (cristal con sus medidas, persiana) y herrajes.
- Al aceptar, el trabajo lleva lo que hay que pedir; el dueño o el operario lo marcan **pendiente → pedido**; **recibido** existe por si se quiere apuntar, pero no hace falta.
- Que crezca bien: más adelante puede haber otras cosas que se piden.

## Decisión
1. **Dos dibujos por ventana:** el del presupuesto (limpio, medida total) y el del taller (números de pieza, flechas y cotas del marco), que usan la hoja de taller, «Mi día» y la ficha del trabajo. El resumen guarda los dos (`dibujo`, `dibujo_taller`); los trabajos anteriores conservan el suyo.
2. **Qué se pide es un dato de la serie:** el cristal siempre, y los artículos con `se_pide: true` en `referencias` (hoy, `PERSIANA`). `dominio::series::material_a_pedir` saca la lista; el resumen de la línea la guarda (`a_pedir`) y el editor la muestra en «Para pedir». Lo que se marque `se_pide` deja de salir en los herrajes.
3. **Tabla `material_a_pedir`** (migración 0005, solo añade): una fila por cosa y ventana del trabajo, con clase (cristal o artículo), código, nombre («Arriba»), modelo del catálogo, cantidad, medida, estado (`pendiente`, `pedido`, `recibido`), nota, quién y cuándo, y versión. Nace al aceptar. La migración pasa a ella los cristales de los trabajos en marcha (lo marcado «cristal recibido» queda recibido); la casilla antigua deja de usarse y la columna se conserva.
4. **Lo marcan** el dueño desde la ficha del trabajo y el operario desde «Mi día», en las ventanas que puede ver (como los pasos). Con la versión vieja se avisa en lugar de pisar.
5. **Al modificar medidas**, la lista se rehace: lo pendiente toma la medida nueva; lo pedido o recibido con otra medida vuelve a pendiente con la nota «La medida cambió después de pedirlo» (`dominio::taller::estado_tras_cambiar_la_medida`).
6. **Aviso en el Inicio** (y en la campana) por trabajo: «T-2026-0001: falta pedir cristal y persiana».

## Consecuencias
- Para pedir otra cosa a medida, se marca `se_pide: true` en su referencia de la serie: sale en «Para pedir» y en el trabajo sin tocar código.
- Pendiente de decidir con el propietario: una hoja para pedir (todos los cristales de uno o varios trabajos) y la medida que se da al proveedor de persianas.

## Alternativas descartadas
- **Una lista fija en el código de lo que se pide:** obligaría a recompilar para cada cosa nueva.
- **Una casilla por clase en la ventana (como «cristal recibido»):** no guarda medida, cantidad, nota ni quién, y no admite más de una cosa.
