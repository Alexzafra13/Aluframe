# ADR-0023: callejero que sugiere la calle y rellena el resto de la dirección

**Estado:** aceptado (2026-11-23, petición del propietario).

## Contexto
Tras ADR-0020, el código postal y la localidad ya se rellenan entre sí. El propietario pide que también se pueda empezar por la calle. El taller trabaja de momento en Zafra y alrededores, y el servidor puede no tener internet.

## Decisión
1. **Fuente:** el callejero del censo electoral del INE (`https://www.ine.es/prodyser/callejero/`, archivo nacional `caj_esp_MMAAAA.zip`, unos 48 MB). Trae cada vía con su tipo y su nombre, los tramos con su código postal y las unidades de población (entidades y núcleos). Licencia **CC BY 4.0** según el aviso legal del INE. La cita obligatoria va en Ajustes > Datos y en la cabecera del archivo: «Elaboración propia con datos extraídos del sitio web del INE: www.ine.es», con la fecha del callejero (julio de 2025).
2. **Solo la provincia de Badajoz** (`data/callejero/06.tsv`): 165 municipios, unas 19 000 calles y 520 kB. Otra provincia se añade generando su archivo e incluyéndolo en `aplicacion/calles.rs`.
3. **Formato propio**, generado con `scripts/generar_callejero.py`. Por municipio va un renglón `=Municipio|Localidad|…`. Por calle va `Tipo<TAB>Nombre<TAB>cp|cp<TAB>índices de localidad`.
   - Los nombres se escriben bien: «AVDA DUQUES DE FERIA» pasa a «Avenida Duques de Feria» y «SANTOS DE MAIMONA (LOS)» a «Los Santos de Maimona». Se conservan «XIII» y «N-430».
   - **No se añaden tildes** que el ayuntamiento no puso. Si una calle o localidad aparece escrita con y sin tildes, se queda la que las tiene.
   - Se descartan las vías llamadas «DISEMINADO», que no son calles. Las vías de un diseminado cuentan también como del municipio, para que «Zafra» encuentre sus fincas.
4. **Dónde se busca** (`dominio::callejero`):
   - Si la localidad escrita está en el callejero, solo en ella, sea municipio o pedanía.
   - Si no está (nombre antiguo o escrito distinto, como «Torremegia»), por el código postal del formulario.
   - Sin localidad, en toda la provincia, y la lista muestra el pueblo de cada calle.
   - Con una localidad de otra provincia y sin código, no se sugiere nada: sería equivocarse.
5. **Orden:** el mismo que las localidades (exacto, empieza por, una palabra empieza por, contiene), sobre el nombre y sobre «tipo + nombre». A igualdad, primero las calles con código postal (las que no lo tienen no tienen viviendas censadas) y después las más cortas. Al principio se aceptan abreviaturas: «c/», «avda», «pza», «ctra»…
6. **Al elegir una calle** se escribe «Calle X, » y el cursor se queda para el número. Se rellenan la localidad, la provincia, el país y el código postal **si la calle tiene uno solo**. Con varios (unas 30 avenidas de Badajoz capital), el código queda vacío y ofrece los suyos, como con las ciudades en ADR-0020. Lo escrito a mano no se toca. Tras la coma ya no se sugiere nada.
7. **Nombres de localidad con el artículo delante:** GeoNames escribía «Santos de Maimona, los» y no casaba con el INE. `generar_codigos_postales.py` ahora pone el artículo delante («Los Santos de Maimona», «L'Alcora», «O Carballiño») y el listado se ha regenerado con esa regla (776 nombres).

## No incluido
- **Elegir el código según el número del portal.** Los tramos del INE lo permiten, pero solo serviría para unas 30 avenidas y obligaría a separar calle y número.
- **Toda España:** unos 15 MB más. Se valorará si el taller trabaja fuera.

## Requisitos del ADR-0002
1. **Requisito:** sugerir calles sin internet. Basta con un listado y unas pocas líneas propias; no hay dependencias nuevas.
2. **Estabilidad:** el INE publica el callejero dos veces al año con un diseño de registro fijado por el Consejo de Empadronamiento. Si cambia, solo hay que tocar el script.
3. **Mantenimiento:** regenerar el archivo de vez en cuando (`docs/MAINTENANCE.md`).
4. **Licencia y tamaño:** CC BY 4.0. Añade unos 0,6 MB al ejecutable y unos pocos MB de memoria. Se lee la primera vez que se usa.
5. **Pruebas y sustitución:** pruebas del dominio con datos de ejemplo, de integración con el archivo real (Zafra, Badajoz y sus pedanías, Valuengo, Los Santos de Maimona) y E2E. El formato es texto simple: se puede generar desde otra fuente sin tocar el código.
6. **Alternativas descartadas:**
   - **OpenStreetMap:** licencia ODbL con obligación de compartir igual y menos completo en pueblos pequeños.
   - **Catastro:** pensado para parcelas, no para direcciones postales, y sin código postal.
   - **CartoCiudad (IGN):** un servicio en línea, y su descarga no trae el código postal por tramo de forma directa.
   - **Una API en internet:** el servidor puede no tener conexión.
