# ADR-0025: apuntes, notas y tareas sueltas

**Estado:** aceptado (2026-11-23, plan aprobado por el propietario; parte 2 del ADR-0024).

## Contexto
No todo el trabajo del taller es una ventana que se calcula con una serie. Llaman pidiendo presupuesto de mosquiteras, hay que cambiar una cerradura o falta un remate en una obra. El propietario quiere que el operario pueda **tomar apuntes que le lleguen**, y poder **mandar tareas sueltas** a un operario. Cuando el programa tenga más tipos de trabajo, parte de esto pasará a ser trabajo definido; los apuntes seguirán para lo que no encaje.

## Decisión
1. **Un apunte** lleva un texto obligatorio y, si se quiere, medidas tomadas (texto libre, una por línea), un cliente con una de sus direcciones de montaje o un contacto suelto (nombre o teléfono de quien llama sin ser cliente). Validación en `dominio::apuntes`.
2. **Dirección:**
   - Sin destinatario, el apunte es **para el dueño**: le llega a la campana y al Inicio.
   - Con un operario como destinatario es una **tarea**: le sale en «Mi día», encima de sus ventanas, hasta que la marca como hecha.
   - Solo el dueño manda tareas.
3. **Estados:** *pendiente*, *hecho* o *presupuestado*.
   - *Hecho* se puede reabrir, por si se marcó por error.
   - *Presupuestado* ya no cambia: el presupuesto sigue su camino.
4. **Quién ve y hace qué** (`Accion::TomarApuntes`, todos; `Accion::GestionarApuntes`, solo el dueño):
   - **El operario** escribe apuntes y ve los suyos y las tareas que le mandan. Solo marca como hechas esas tareas; lo que apunta para el dueño lo cierra el dueño.
   - **El dueño** ve todos, manda tareas, marca y reabre, y **«Hacer presupuesto»**. Este botón crea el borrador con el cliente y la dirección del apunte y lo deja *presupuestado* en la **misma transacción**: o se hacen las dos cosas o ninguna.
5. **Datos.** La migración 0003 añade `apunte`; no toca lo que había.
   - **Con el cliente:** si se borra un cliente (protección de datos), se borran sus apuntes (`ON DELETE CASCADE`), y su exportación los incluye.
   - **Con las direcciones:** quitar una dirección deja el apunte sin ella.
   - **Cambios a la vez:** se detectan por versión, igual que en el resto del programa.
6. **Pantallas.**
   - *Apuntes* en el menú, con pestañas *Pendientes* y *Hechos* (los 200 más recientes).
   - *Nuevo apunte* desde la lista o desde la ficha del cliente, que ofrece sus direcciones.
   - *Tareas* en «Mi día».
   - Grupo *Apuntes* en la campana y en el panel del Inicio.

## Alternativas descartadas
- **Usar las «notas» del cliente:** no tienen estado ni destinatario, y no sirven para quien llama sin ser cliente.
- **Medidas con campos (hueco, ancho, alto):** el operario apunta deprisa y no siempre es una ventana. Un texto libre de varias líneas basta para pasarlas a mano al presupuesto.
- **Tareas entre operarios:** de momento las reparte el dueño; se puede abrir más adelante con un permiso.

## Pendiente
- Fotos en los apuntes (hueco, papel escrito a mano).
- Parte 3 del plan: la pantalla del operario para la tablet.
