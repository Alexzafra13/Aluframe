# Preguntas abiertas

Marca cada una cuando se resuelva e indica dónde queda recogida la respuesta.

## Ventanas y fabricación
- [ ] Fórmulas de ancho y alto del cristal (descuento) y tipo de vidrio habitual, en cada serie.
- [x] `ancho` y `alto` son la **medida de la ventana** (plan de H3 aprobado, 2026-11-21); se valida con las 5 ventanas reales tras H3.
- [ ] ¿Marco superior, lateral e inferior salen del mismo perfil o de perfiles distintos?
- [ ] ¿Se optimizan los cortes por ventana o por trabajo? ¿Qué se hace con los retales?
- [ ] Grosor de sierra: **2 mm provisional** (el propietario lo comprobará); recorte de extremos sin dato (se usa 0); retal mínimo 0 por decisión del propietario. Se cambian en Ajustes > Taller.
- [ ] Aluprom 21 de 4 hojas: confirmar ruedas (8) y tornillos (9 + 16), que son supuestos. Otras tipologías (fija, puertas). ¿Lleva el perfil «cierre 4 hojas» 21344?
- [x] Nombres de las hojas de Aluprom 21: los del catálogo de Aluval, HOJA LATERAL y HOJA CIERRE (propietario, 2026-11-22; ADR-0017).
- [x] Hojas de Aluprom 21 según el cristal (propietario, 2026-11-22): sencillo → 21310, 21340, 21350; doble (Climalit) → 21312, 21341, 21351. **Pendiente de modelar** el tipo de cristal en la línea para elegir la referencia de precio.
- [ ] Aluprom 21, marcos: el propietario cree que el lateral es siempre 21210 y el superior 21220 (confirmar en el taller). El inferior es siempre el mismo, pero falta saber si es 21230 (con cámara) o 21231 (sin cámara).
- [x] **Fixture de la abatible y bisagras:** el propietario confirma (2026-11-21) 2 bisagras por hoja hasta 140 cm de alto de la ventana, incluido, y 3 por encima. El fixture incluye ahora «BISAGRA: 4» para 120 × 140.
- [ ] V-94 abatible: cantidad de inversores y su ángulo de corte, cálculo de las pletinas de falleba, tipos de escuadra (exterior e interior de hoja) y si los marcos superior, lateral e inferior son el mismo perfil.
- [ ] Redondeo: ¿a 0,1 cm hacia abajo es lo correcto? Validarlo con ventanas reales.
- [ ] Optimizador con cortes a 45°: de momento cada pieza cuenta por su lado más largo (conservador). Revisarlo con el taller.
- [x] Cómo se modifica un trabajo ya aceptado: lo hace el propietario con "Modificar medidas" (`docs/PANTALLAS.md`, apartado 8). El operario no.
- [ ] Al modificar medidas con pasos ya hechos: ¿se reinician siempre o decide el propietario (propuesto: decide él)?

- [x] Longitud de barra de la V-94: 6,40 m (confirmado 2026-11-21).

## Precios y documentos
- [ ] Proveedores (Aluval y Sopena de momento) y formato de sus tarifas. El usuario reunirá precios por barra y tarifas.
- [ ] Recargo por color: el blanco es el más barato y el resto cuesta un porcentaje más. ¿Es el mismo para todas las series o depende del proveedor?
- [ ] Facturas de compra: ¿llegan en papel, PDF digital o Excel? Decide si se importan (y cómo) o se teclean.
- [ ] Cómo se cobra el material (barra, metro o kilo) y si el proveedor factura una superficie mínima de cristal. **Desde H3, perfiles por barras enteras o por metros + merma, a elegir en Ajustes > Presupuestos; el presupuesto muestra los dos importes.**
- [ ] Mano de obra: ¿importe por ventana, por m² o a mano? **Desde H3, importe por ventana en cada línea con un valor por defecto en Ajustes (vacío = 0). Provisional: se suma después del margen (es lo que se cobra, no lleva margen); el propietario aún no lo ha decidido (2026-11-21).**
- [ ] Margen: ¿sobre coste o sobre venta? Descuentos habituales. **Desde H3 es configurable en Ajustes > Presupuestos; mientras no se elija, ninguna línea tiene precio final. El propietario aún no lo ha decidido (2026-11-21).**
- [ ] Tipos de IVA, casos especiales, y cómo se redondea la cuota (confirmar con el gestor). **Confirmado por el propietario (2026-11-21):** 21 % por defecto; las líneas y la base se muestran sin IVA y el IVA se suma al final (total con IVA).
- [ ] Documentos que realmente se emiten (presupuesto, albarán, factura) y formato de numeración.
- [ ] Datos fiscales del taller y logotipo.

## Formato de datos
- [x] Enlace entre el perfil de una serie y la referencia de la tarifa: resuelto con la tabla `referencias` (`docs/FORMATO_SERIES.md`, apartado 5).
- [x] Formato de `vidrio` en las series: `docs/FORMATO_SERIES.md`, apartado 6.
- [ ] Pletinas de falleba: fórmula y unidad (¿se venden por metro o por unidad cortada?).
- [ ] Límites de medida por serie (`limites`): mínimos y máximos reales de cada tipología.

## Implantación y fiscalidad
- [ ] ¿Con qué se gestionan hoy clientes, presupuestos y facturas (papel, hoja de cálculo, otro programa)? ¿Qué datos hay que importar?
- [ ] Verifactu y factura electrónica entre empresas: Hacienda anunció en octubre de 2026 el aplazamiento a octubre de 2028 (sin norma publicada). Confirmar con el gestor cuándo aplica al taller, qué formato y envío se exigirá al facturar a constructoras, y si el programa se ofrecerá a terceros (podría implicar obligaciones como productor de software).

## Infraestructura
- [x] Servidor: mini PC con **Linux**; la oficina usa **Windows** por navegador (`docs/CAMBIOS.md`, 2026-11-21). Pendiente: cuándo se incorpora la tablet del operario.
- [x] Caducidad de sesión: 2 h sin actividad y 12 h como máximo; el propietario puede elegir al entrar mantenerla hasta cerrar el navegador (`docs/CAMBIOS.md`, 2026-11-21).

## Funciones de otros programas del sector (revisión 2026-11-20)
Comparado con programas como Productor (Gaia), Cortizo Software, GesFácil, Klaes o ePref. Decidir cuáles entran en el MVP:
- [ ] **Pedido de material a proveedor**: hoja de pedido por proveedor y color generada desde la lista de compra (hoy solo hay lista sugerida).
- [ ] **Pedido de vidrio a medida**: lista de cristales con medidas y cantidades para el proveedor (no la vi en esos programas; es habitual en el oficio).
- [ ] **Presupuesto con opciones alternativas** (por ejemplo, europea o corredera, o dos acabados) dentro del mismo presupuesto.
- [ ] **Informe de margen por trabajo**: coste real frente a lo vendido.
- [ ] **Límites por serie**: avisar si el ancho o el alto superan lo que admite el perfil (a añadir en `docs/FORMATO_SERIES.md`).
- [ ] **Condiciones y validez del presupuesto**: plantilla de texto (plazo, forma de pago, garantía).
- [ ] **Contabilidad mínima**: registrar facturas de proveedores (lo que se debe) y exportar para el gestor; la contabilidad completa queda fuera.
- [ ] Después del MVP: etiquetas de pieza, hoja de montaje, conexión con tronzadora, optimización multibarra, código de barras.

## Interfaz y pantallas (ver `docs/PANTALLAS.md`)
- [ ] Menú de 8 entradas (se construye así salvo que el propietario diga otra cosa) o las 10 del diseño aprobado.
- [x] Operario: lista de tarjetas y una ventana a pantalla completa al abrirla.
- [ ] Cristal recibido: se marca a mano en el trabajo (propuesto).
- [ ] Qué más debe aparecer en el panel de atención del Inicio.
- [x] Ningún dato de cliente es obligatorio. Pendiente con el gestor: qué casos admiten factura simplificada o sin NIF.
- [ ] El prototipo de flujo (`alumframe-flujo.html`) tiene una organización distinta a la aprobada y se rehará con `docs/PANTALLAS.md`.

## Trabajo diario del operario
- [ ] Lista de pasos por ventana: propuesta por defecto cortar, montar marco, montar hojas, acristalar, accesorios y cierre, revisar. Ajustarla al taller; ¿cambia según la tipología?
- [ ] Cuándo se descuenta el stock: al terminar la orden (como está) o al marcar el paso de corte. Con el trabajo por pasos, descontar al cortar sería más fiel.
- [ ] ¿Se quiere medir el tiempo por paso o por ventana? De momento no.
- [ ] Cremona: ¿el lado se mira desde dentro o desde fuera? ¿Cambia alguna pieza (por ejemplo, qué hoja lleva el inversor) según el lado?
- [x] Barras de longitud distinta de 640 cm (2026-11-21): si alguna serie o marca las vende de otra longitud, se configura en esa serie cuando aparezca. Los retales existentes se registran en el stock al instalar el programa (con su longitud, perfil y color) y el plan de corte los propone antes de abrir barras nuevas (`SPEC.md`, apartado 9 y H5).
- [x] Retal mínimo aprovechable: **0**, se guarda cualquier sobrante por si encaja en algo (propietario, 2026-11-21). Registrar retales al instalar es opcional.
- [x] Retales: no se marcan; basta con perfil, color y longitud para buscarlos (propietario, 2026-11-21, de momento).

## Bisagras y dibujo de la abatible
- [x] Bisagras: 2 por hoja hasta 140 cm de alto de la ventana (140 incluido) y 3 por encima (confirmado 2026-11-21).
- [ ] ¿Las bisagras van dentro del kit de 2 hojas o se piden aparte?
- [ ] Posición de las bisagras en el dibujo (desde H3, al 15 % y al 85 % del alto de la hoja, y en medio si hay tres): ¿a qué distancia de los extremos van?
