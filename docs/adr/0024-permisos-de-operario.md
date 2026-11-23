# ADR-0024: permisos por operario

**Estado:** aceptado (2026-11-23, decisión del propietario).

## Contexto
Hasta ahora había dos roles fijos: el dueño lo veía todo y el operario solo «Mi día», sin precios. El propietario quiere que cada operario tenga su cuenta y vea **lo justo y necesario**. Por ejemplo:
- uno que atiende a clientes debe poder buscarlos y darlos de alta;
- uno de confianza puede ver un presupuesto para explicárselo a un cliente;
- un encargado ve los trabajos de todo el taller.

Esto cambia la regla «el trabajador no ve precios ni márgenes» de `CLAUDE.md` y `SPEC.md`.

## Decisión
1. **Se mantienen los dos roles.** El dueño lo puede todo. El operario tiene **lo básico** más los **permisos** que el dueño le marca en Ajustes > Usuarios.
2. **Lo básico** (siempre): «Mi día», marcar pasos, «falta material» y «repetir pieza», y **buscar clientes** y ver sus datos de contacto y direcciones (no los archivados).
3. **Permisos** (casillas; un operario nuevo empieza sin ninguno):
   - *Dar de alta clientes y corregir sus datos* (también sus direcciones).
   - *Ver presupuestos, con su precio de venta*: en lectura y para imprimir.
   - *Ver facturas y cobros*: en lectura.
   - *Ver y marcar los trabajos de todo el taller*: en «Mi día», bajo «Resto del taller».
4. **Nunca para un operario**, tenga los permisos que tenga (lo comprueban las pruebas):
   - costes y márgenes (el desglose de perfiles, cristal, accesorios y mano de obra ni se calcula para él);
   - catálogo, tarifas y stock;
   - crear o cambiar presupuestos;
   - emitir facturas, cobrar o rectificar;
   - gestionar trabajos (asignar, cambiar medidas, entregar);
   - usuarios, ajustes, copias y datos;
   - archivar, borrar, exportar o importar clientes (protección de datos).
5. **Dónde se decide.** En `dominio::acceso::puede_realizar(rol, permisos, accion)`, en un único sitio. El servidor lo comprueba en **cada** caso de uso con `autorizar`: lo no permitido responde «no encontrado», como hasta ahora, para no revelar que existe. Las plantillas usan `UsuarioEnPantalla` solo para no enseñar enlaces y botones inútiles; ocultarlos no es la protección.
6. **Datos.** La migración 0002 añade `usuario_permiso (usuario_id, permiso)`; no toca nada de lo que había. Los permisos se leen en cada petición: un cambio vale desde la siguiente página que abra el operario, sin volver a entrar. Un permiso guardado que el programa no conoce (de una versión más nueva) no da acceso a nada.
7. **Regla nueva** en `CLAUDE.md` y `SPEC.md`: *el operario no ve precios salvo que el dueño se lo permita, y nunca costes ni márgenes.*

## Alternativas descartadas
- **Más roles fijos** («encargado», «comercial»…): obliga a adivinar combinaciones; las casillas son más claras para un taller pequeño.
- **Permisos totalmente libres**, como en un ERP grande: demasiadas opciones y demasiado riesgo de enseñar costes por error. Lo que es solo del dueño no se puede marcar.

## Pendiente (mismo plan aprobado)
- **Parte 2 (hecha, ADR-0025):** *apuntes*, notas y tareas sueltas para lo que no es una ventana: del operario al dueño (le llegan a la campana) y del dueño a un operario (le aparecen en «Mi día»).
- **Parte 3 (hecha, 0.16.0):** pantalla del operario pensada para la tablet (Mi día · Apuntes · Clientes y lo permitido).
- **Más adelante:** fotos en los apuntes.
