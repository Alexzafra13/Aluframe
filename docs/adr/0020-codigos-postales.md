# ADR-0020: código postal que rellena localidad, provincia y país

**Estado:** aceptado (2026-11-22, petición del propietario).

## Contexto
El propietario quiere tardar menos al dar de alta clientes: al escribir el código postal deberían rellenarse la localidad, la provincia y el país, y que lo que se rellene sea correcto. El servidor puede no tener internet, así que no vale consultar un servicio externo.

## Decisión
1. **La provincia sale de las dos primeras cifras del código postal** (01 Álava … 52 Melilla), la regla de Correos. Es exacta y no necesita datos. El país pasa a ser «España» si está vacío.
2. **Las localidades salen de un listado incluido en el programa**, `data/codigos_postales.tsv` (unos 11 000 códigos, 700 kB), generado con `scripts/generar_codigos_postales.py` a partir del archivo `ES.zip` de GeoNames (licencia CC BY 4.0; se cita en el archivo y en Ajustes > Datos). Por cada código: primero el municipio y después los demás núcleos.
3. **Nunca se adivina:**
   - Si a un código le corresponde una sola localidad, se escribe. Si le corresponden varias (pasa en los pueblos, 04002 es Almería o El Palmer), el campo se queda vacío y las ofrece en su lista para elegir.
   - Al escribir la localidad salen las más parecidas (ver «Cambio del 2026-11-23»). Al elegir una se rellenan su provincia, su país y su código postal **si solo tiene uno**; con varios, el código queda vacío y se ofrecen los suyos para elegir.
   - Si se sale del campo sin elegir y el código está vacío, se busca el nombre exacto: la provincia se rellena solo si todos sus códigos son de la misma, y el código, solo si es único.
   - Si un nombre es municipio en un sitio y aldea en otro («Zafra» es el municipio de Badajoz y una aldea de Ourense), cuenta el municipio.
   - Solo se rellenan los campos vacíos o los que rellenó el programa; lo escrito a mano no se toca.
4. **GeoNames tiene errores** (46005 figura con el municipio «Madrid»; algunos códigos, con la provincia de al lado). El script descarta las filas cuya provincia no es la de las dos cifras y el municipio cuyo código INE es de otra provincia. Las pruebas comprueban el listado real.
5. **Dónde:** dirección fiscal del cliente, direcciones de montaje y datos de la empresa. Un script pequeño (`recursos/js/poblacion.js`) pide `/poblaciones?cp=…` o `?localidad=…`, que responde en JSON; la búsqueda vive en el dominio (`dominio::codigos_postales`).

## Requisitos del ADR-0002
- Sin dependencias nuevas: el listado se incluye con `include_str!` y se lee una vez.
- Pruebas: dominio (provincia por cifras, lectura del listado, preferencia del municipio), integración (el listado incluido se lee entero, Zafra, Madrid) y E2E (rellena, no elige entre varias y no pisa lo escrito).
- Sustitución: el formato es un texto simple (`código<TAB>localidad|…`); se puede generar desde otra fuente (por ejemplo, el callejero de CartoCiudad del IGN) sin tocar el código.

## Alternativas descartadas
- **Servicio en internet** (API de Correos o similares): el servidor del taller puede no tener conexión y no queremos depender de un tercero.
- **Listado del IGN/CartoCiudad** procesado por terceros: solo da el código INE del municipio (hace falta otra tabla para el nombre) y su licencia en el repositorio no está clara.
- **Solo la provincia:** no ahorra lo que pide el propietario.

## Cambio del 2026-11-23 (petición del propietario)
Antes, al escribir la localidad solo se rellenaba la provincia al salir del campo, y el código postal se ofrecía pero no se escribía, por si el listado no los tenía todos. El propietario prefiere que, al elegir un pueblo con un único código, se escriba: es lo habitual y, si no fuera el bueno, se corrige a mano. Lo que no cambia: con varios códigos nunca se elige uno (opción A del propietario frente a «poner el primero»).

- **Sugerencias mientras se escribe**, a partir de dos letras: `dominio::localidades::IndiceDeLocalidades` agrupa el mismo listado por nombre y provincia (así se distinguen la Zafra de Badajoz y la de Ourense) y ordena por parecido (nombre exacto, empieza por, una palabra empieza por, contiene) y, a igualdad, municipios antes. Se devuelven 8. Recorrer las ~36 000 localidades por pulsación tarda muy poco; la página espera 150 ms entre pulsaciones.
- **Lista propia** en `poblacion.js` en lugar de `<datalist>`: la del navegador filtra a su manera (sin ignorar tildes), no deja mostrar la provincia aparte y en el iPad funciona mal. Accesible como `combobox`/`listbox`, opciones de 44 px, flechas, Intro (sin enviar el formulario) y Esc.

## Pendiente
- Actualizar el listado de vez en cuando (`docs/MAINTENANCE.md`, apartado 6).
