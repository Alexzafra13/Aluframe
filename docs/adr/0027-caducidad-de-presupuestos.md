# ADR-0027: caducidad de los presupuestos, renovar y archivar sin borrar

**Estado:** aceptado (2026-11-23; el propietario pide 30 días porque los precios cambian y deja el resto a nuestro criterio).

## Contexto
El propietario quiere que los presupuestos tengan una validez de 30 días, porque los precios cambian, y propuso borrarlos a los 6 meses si no se aceptan. Ya existían la fecha «válido hasta» (impresa en la hoja) y el bloqueo de aceptar si la tarifa había cambiado, pero nada pasaba al vencer la validez. La lista de Presupuestos además solo mostraba los 500 últimos.

## Decisión
1. **30 días por defecto** (Ajustes > Presupuestos, se puede cambiar; vacío es «sin fecha»). La migración 0004 pone 30 solo si el ajuste estaba vacío, el valor que había antes por defecto: un número puesto a mano no se toca y ningún presupuesto cambia.
2. **«Caducado» y «Archivado» se calculan, no se guardan** (`dominio::presupuesto::situacion`), con la fecha de hoy en Madrid, igual que «Vencida» en las facturas. No hace falta ninguna tarea nocturna ni cambiar el esquema.
   - *Caducado:* borrador o enviado cuya validez (incluido su último día) ya pasó.
   - *Archivado:* sin aceptar y con la validez vencida hace más de 182 días (medio año). Un rechazado también se archiva. Un aceptado nunca caduca.
3. **Un caducado no se acepta**: hay que **renovarlo**. «Renovar» (también para archivados y rechazados) pone los precios de la tarifa vigente y una validez nueva desde hoy. Si se había enviado, lo enviado queda como versión anterior y se trabaja en una versión nueva en borrador, que se vuelve a enviar.
4. **Archivar, no borrar.** Un archivado sale de la lista principal y del filtro de caducados. Se ve en el filtro «Archivados» y en la ficha del cliente, y se puede renovar. Motivos: los clientes vuelven («me hiciste uno en primavera»); el historial de medidas para el corte (ADR-0026) sale de lo aceptado y se quiere conservar lo que se presupuestó; ocupan muy poco; borrar no tiene vuelta atrás. Los datos de un cliente que pide que se le borre siguen el camino del RGPD (H2).
5. **Avisos del Inicio:** «enviados sin respuesta» muestra solo los que siguen vigentes; uno caducado deja de esperar respuesta y se ve en el filtro «Caducados».
6. **Listas:** primero se lee de cada presupuesto solo su estado y su validez, el dominio decide su situación y después se cargan los que se muestran (hasta 500). Así la regla vive en un único sitio y no se cargan las copias congeladas de los que no salen.

## Consecuencias
- El propietario debe renovar antes de aceptar un presupuesto viejo; el programa se lo dice al lado del botón «Aceptar».
- Cambiar los días de validez afecta a los presupuestos nuevos y a los que se renueven; los ya hechos conservan su fecha.
- La migración 0004 hace que un ejecutable anterior a la 0.17.0 rechace la base actualizada (como con cada migración).

## Alternativas descartadas
- Borrar a los 6 meses (propuesta inicial del propietario): irreversible y sin ventaja real de espacio.
- Guardar el estado «caducado» con una tarea programada: más piezas que mantener y riesgo de quedarse sin pasar si el equipo estaba apagado.
- Renovar sin versión nueva: se perdería lo que se le había enviado al cliente.
