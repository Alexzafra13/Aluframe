# Preguntas abiertas

Marca cada una cuando se resuelva e indica dónde queda recogida la respuesta.

## Ventanas y fabricación
- [ ] Fórmulas de ancho y alto del cristal (descuento) y tipo de vidrio habitual, en cada serie.
- [ ] `ancho` y `alto`: ¿hueco de obra o medida de la ventana? (las fórmulas dadas sugieren medida de ventana; confirmarlo)
- [ ] ¿Marco superior, lateral e inferior salen del mismo perfil o de perfiles distintos?
- [ ] ¿Se optimizan los cortes por ventana o por trabajo? ¿Qué se hace con los retales?
- [ ] Grosor de sierra, recorte de extremos y retal mínimo aprovechable.
- [ ] Aluprom 21 de 4 hojas: confirmar ruedas (8) y tornillos (9 + 16), que son supuestos. Otras tipologías (fija, puertas).
- [ ] V-94 abatible: cantidad de inversores y su ángulo de corte, cálculo de las pletinas de falleba, tipos de escuadra (exterior e interior de hoja), longitud de barra y si los marcos superior, lateral e inferior son el mismo perfil.
- [ ] Redondeo: ¿a 0,1 cm hacia abajo es lo correcto? Validarlo con ventanas reales.
- [ ] Optimizador con cortes a 45°: de momento cada pieza cuenta por su lado más largo (conservador). Revisarlo con el taller.
- [x] Cómo se modifica un trabajo ya aceptado: lo hace el propietario con "Modificar medidas" (`docs/PANTALLAS.md`, apartado 8). El operario no.
- [ ] Al modificar medidas con pasos ya hechos: ¿se reinician siempre o decide el propietario (propuesto: decide él)?

## Precios y documentos
- [ ] Proveedores (Aluval y Sopena de momento) y formato de sus tarifas. El usuario reunirá precios por barra y tarifas.
- [ ] Recargo por color: el blanco es el más barato y el resto cuesta un porcentaje más. ¿Es el mismo para todas las series o depende del proveedor?
- [ ] Facturas de compra: ¿llegan en papel, PDF digital o Excel? Decide si se importan (y cómo) o se teclean.
- [ ] Cómo se cobra el material (barra, metro o kilo) y si el proveedor factura una superficie mínima de cristal.
- [ ] Margen: ¿sobre coste o sobre venta? Descuentos habituales.
- [ ] Tipos de IVA, casos especiales, y cómo se redondea la cuota (confirmar con el gestor).
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
- [ ] Caducidad de sesión deseada para la oficina y para el operario.

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
- [ ] Barras de longitud distinta de 640 cm: ¿se compran así o son retales? Cómo se registran en stock y cuándo las usa el optimizador.

## Bisagras y dibujo de la abatible
- [ ] Bisagras: 2 por hoja hasta 140 cm de alto y 3 por encima. ¿A los 140 cm exactos son 2? ¿El límite es el alto de la ventana o el de la hoja? ¿Van dentro del kit de 2 hojas o se piden aparte?
- [ ] Posición de las bisagras en el dibujo (hoy repartidas a ojo): ¿a qué distancia de los extremos van?
