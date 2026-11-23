# ADR-0028: demostración con datos inventados dentro del propio programa

**Estado:** aceptado (2026-11-23; idea y aprobación del propietario: «un modo en el propio programa como demo y poder salir»).

## Contexto
El propietario quiere probar el programa entero y aprender a usarlo sin rellenar datos a mano y sin tocar los de verdad. La regla del proyecto es que los datos inventados nunca se mezclan con los reales (`CLAUDE.md`, `fixtures/demo/`). Se valoró primero un programa aparte (otro icono y otro puerto); el propietario propuso un modo dentro del mismo programa, del que se entra y se sale, y es mejor: más fácil de encontrar, sin procesos extra y válido también en el servidor y en la tablet.

## Decisión
1. **Una base aparte**, `demostracion/alumframe.db` dentro de la carpeta de datos, marcada en `configuracion` (`demostracion = si`). Se crea al arrancar y se rellena la primera vez que alguien entra.
2. **Se entra y se sale por navegador.** «Probar la demostración» en la pantalla de entrada (y en Ajustes > Datos) pone una cookie `alumframe_demostracion` y guarda la sesión de verdad en `alumframe_volver`. Con esa cookie, cada petición de ese navegador va a un segundo enrutador con la base de la demostración; los demás equipos siguen con los datos reales. «Salir de la demostración» borra la cookie y devuelve la sesión de verdad (si caducó, se vuelve a la entrada).
3. **Imposible confundirlas:**
   - una base real con la marca no arranca; una carpeta de la demostración con datos sin la marca no se usa;
   - una copia con la marca no se importa en los datos de verdad;
   - en la demostración no hay copias automáticas ni Ajustes > Copias o Datos (exportar o importar);
   - todas sus pantallas llevan una franja «Demostración» con el botón de salir.
4. **Datos inventados** generados con los propios casos de uso (siempre cuadran con la versión instalada y de paso prueban el recorrido entero): empresa «Carpintería Demo», proveedor y tarifa de precios inventados para la serie real Aluprom 21 (`fixtures/demo/tarifa-demo-aluprom-21.csv`), stock con retales y un artículo bajo mínimo, 12 clientes «Demo», presupuestos en cada situación (borrador, enviado, caducado, rechazado, archivado y aceptados), trabajos sin empezar, a medias, terminados y entregado, facturas pagada, vencida a medias, pendiente y anulada con su rectificativa, una señal, apuntes y un operario. Usuarios: «Dueño (demo)» con la contraseña `demostracion` y «Juan (demo)» con el PIN `123456`, que se enseñan en su entrada.
5. **Precios:** la serie es la real (validada por el propietario) y los precios, inventados; dentro de la demostración dan precio final. No hace falta ninguna excepción en el cálculo y la instalación real sigue igual (sin tarifa real, provisional).
6. **«Empezar de nuevo»** (solo el dueño de la demostración): se rellena una base nueva en memoria y sustituye a la de la demostración con la misma restauración que la importación de datos.

## Dependencia: `tower` pasa de pruebas a normal
1. *Requisito:* reenviar una petición al enrutador de la demostración (`ServiceExt::oneshot`). Sin ella habría que reimplementar la llamada a un servicio de `tower-service` a mano.
2. *Estabilidad:* biblioteca base del ecosistema de Axum, versión 0.5 estable.
3. *Mantenimiento:* ya estaba compilada (Axum depende de ella) y fijada en `Cargo.lock` como dependencia de pruebas; no entra ningún crate nuevo.
4. *Licencia:* MIT. Sin coste de tamaño ni de compilación apreciable.
5. *Pruebas y sustitución:* `tests/demostracion.rs` cubre el desvío; se sustituye por la llamada al trait `Service` de Axum si hiciera falta.
6. *Alternativas:* un proceso aparte con otro puerto (primera idea, descartada por el propietario); un único enrutador que eligiera la base en cada caso de uso (cambiaría cientos de llamadas).

## Consecuencias
- El enrutador de la base de verdad ya resolvió la ruta antes del desvío: la petición se reenvía limpia, solo con quién se conecta (fallo encontrado al probarlo en el móvil y corregido con su prueba).
- Al entrar en la demostración se cierra la sesión del navegador en la de verdad (se recupera al salir).
- La demostración ocupa unos cientos de kB en la carpeta de datos y no entra en las copias.
