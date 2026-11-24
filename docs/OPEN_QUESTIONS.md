# Preguntas abiertas

Marca cada una cuando se resuelva e indica dónde queda recogida la respuesta.

## Ventanas y fabricación
- [ ] Fórmulas de ancho y alto del cristal (descuento) y tipo de vidrio habitual, en cada serie. Aluprom 21, corredera de 2 hojas: **dos cristales de (ancho / 2 − 9,8) × (alto − 13,1) cm**. El ancho lo da el propietario y coincide con el plano de Aluval ((L/2) − 98 mm); el alto es **provisional** (alto de la hoja menos los mismos 7,8 cm que en el ancho), pendiente de comprobar con un cristal real. Falta el cristal de la de 4 hojas. El plano de Aluval (con Climalit y cierre 21343) acota la hoja en H − 55 y (L/2) − 13,5 mm, distinto de los descuentos del taller (5,3 y 2 cm): el propietario decide mantener los suyos, que calculan bien.
- [x] `ancho` y `alto` son la **medida de la ventana** (plan de H3 aprobado, 2026-11-21); se valida con las 5 ventanas reales tras H3. Aluprom 21: medidas de corte de la 2 y la 4 hojas validadas por el propietario (2026-11-22).
- [ ] ¿Marco superior, lateral e inferior salen del mismo perfil o de perfiles distintos?
- [x] Los cortes se optimizan **por trabajo** (decisión 2 de H4) y los retales del stock se usan antes que barras nuevas; al terminar la orden se revisan los que entran (H5).
- [ ] Grosor de sierra: **2 mm provisional** (el propietario lo comprobará); recorte de extremos sin dato (se usa 0); retal mínimo 0 por decisión del propietario. Se cambian en Ajustes > Taller.
- [ ] Aluprom 21 de 4 hojas: confirmar ruedas (8) y tornillos (9 + 16), que son supuestos. Otras tipologías (fija, puertas). Resuelto (propietario, 2026-11-24): lleva el «cierre 4 hojas» 21344, una pieza con el alto de la hoja, y 4 cierres embutidos (la de 2 hojas, 2); ver `docs/CAMBIOS.md`.
- [x] Nombres de las hojas de Aluprom 21: los del catálogo de Aluval, HOJA LATERAL y HOJA CIERRE (propietario, 2026-11-22; ADR-0017).
- [x] Hojas de Aluprom 21 según el cristal (propietario, 2026-11-22): sencillo → 21310, 21340, 21350; doble (Climalit) → 21312, 21341, 21351. Se elige en la línea con la opción «Acristalamiento» (ADR-0018).
- [ ] Climalit: ¿cambian los descuentos de las hojas o del cristal respecto al sencillo? (hoy se suponen iguales). Tipo de Climalit y cámara: cómo se piden y si cambian algo más (lo concretará el propietario).
- [x] Aluprom 21, marcos (propietario, 2026-11-22): lateral 21210, superior 21220 e inferior 21230 (con cámara), con cualquier cristal. Recogido en `data/series/aluprom-21.yaml`.
- [x] **Fixture de la abatible y bisagras:** el propietario confirma (2026-11-21) 2 bisagras por hoja hasta 140 cm de alto de la ventana, incluido, y 3 por encima. El fixture incluye ahora «BISAGRA: 4» para 120 × 140.
- [ ] **Parteluces de Aluprom 21** (ADR-0030; el propietario los pasará cuando los tenga): medida de corte y ángulo del parteluces (21564 / 21565); su altura real en la ventana a partir de la medida de la chapa; ancho de la chapa; alto del cristal de arriba; confirmar que el cristal partido tiene el mismo ancho que sin parteluces; si las dos chapas de una hoja Climalit miden lo mismo; si lleva herrajes o tornillos propios.
- [ ] **Persiana suelta en pared** (ADR-0033, serie `persiana-pared.yaml` en borrador). Resuelto (propietario, 2026-11-24): se pide hecha; se escribe la medida del hueco; guías de L atornilladas a la pared; va al presupuesto con su dibujo, sola o con la ventana. Falta: **ala de la guía de L, 80 o 100 mm** (el plano dice 60, no es esa; canal de 53); **alturas del cajón** y a partir de qué alto va cada uno; qué medida se da al proveedor y **cómo se cobra**; si también hay manivela o motor (hoy, cinta).
- [ ] **Persiana en la ventana** (ADR-0032, correderas de Aluprom 21). Resuelto (propietario, 2026-11-24): la medida escrita es la del conjunto; 4 cm por guía (8 de ancho); cinta a la derecha por defecto, se puede cambiar. Falta: **alturas del cajón y a partir de qué alto va cada uno** (hasta entonces lo que depende del alto sale pendiente); qué medida se da al proveedor al pedirla y **cómo se cobra** (referencia `PERSIANA` en `CONFIRMAR`). Para más adelante: persiana suelta con guías de pared y la mosquitera dentro de la ventana con persiana (su descuento).
- [ ] **Mosquitera enrollable** (ADR-0031, serie en borrador). Resuelto (propietario, 2026-11-24): guía = alto − 4 con cajón de 45 y alto − 5 con cajón de 50; cajón = ancho − 2; el terminal no se corta (viene dentro del cabezal); dos tiradores negros. La malla viene dentro del cajón al cortarlo: no se cuenta aparte. El kit va aparte, uno por mosquitera (con su dibujo desde la 0.21.4). Falta: si todos los cortes son a 90°; referencias, proveedor y longitud de barra de cajón 45, cajón 50, guía y terminal; altos y anchos máximos.
- [ ] **Chapa:** cómo se cobra (sin decidir, una ventana con chapa no tiene precio final); formatos de plancha (200 × 125 y 300 × 125 cm, por confirmar); lacada por una cara o por las dos y cuándo se aprovecha un recorte de dos caras en una Climalit. Para la fase 2 (stock de chapas por color) y la 3 (recortes).
- [ ] V-94 abatible: cantidad de inversores y su ángulo de corte, cálculo de las pletinas de falleba, tipos de escuadra (exterior e interior de hoja) y si los marcos superior, lateral e inferior son el mismo perfil.
- [ ] Redondeo: ¿a 0,1 cm hacia abajo es lo correcto? Validarlo con ventanas reales.
- [ ] Optimizador con cortes a 45°: de momento cada pieza cuenta por su lado más largo (conservador). Revisarlo con el taller.
- [x] Mejor aprovechamiento de las barras con medidas habituales e historial (propietario, 2026-11-23; ADR-0026). Medida de partida: Aluprom 21, corredera de 2 hojas, 120 × 120, editable en Ajustes > Taller.
- [x] Validez de los presupuestos: 30 días; caducado se renueva con los precios de hoy y se archiva a los 6 meses sin borrar (propietario, 2026-11-23; ADR-0027).
- [x] Cómo se modifica un trabajo ya aceptado: lo hace el propietario con "Modificar medidas" (`docs/PANTALLAS.md`, apartado 8). El operario no.
- [x] Al modificar medidas con pasos ya hechos decide el propietario con una casilla (decisión 4 de H4).

- [x] Longitud de barra de la V-94: 6,40 m (confirmado 2026-11-21).

## Precios y documentos
- [ ] Proveedores (Aluval y Sopena de momento) y formato de sus tarifas. No se sabe cómo las enviarán: el propietario pide admitir lo máximo posible. Hoy: CSV, pegar desde Excel y precios a mano en Catálogo > Artículos. **Siguiente tarea:** Excel (.xlsx, .xls, .ods) y PDF asistido. **Pregunta al propietario (2026-11-23, sin respuesta):** ¿cómo manda Aluval los precios (Excel, PDF, papel o web)?
- [ ] Recargo por color: el blanco es el más barato y el resto cuesta un porcentaje más. ¿Es el mismo para todas las series o depende del proveedor?
- [ ] Colores de serie (ADR-0019, `data/acabados.yaml`): confirmar los RAL puestos (blanco 9016, negro 9005, verde 6005, gris antracita 7016, marrón 8014), el nombre «Embero», qué imitaciones de madera se usan (pino nudo, nogal, embero, roble) y la superficie de cada uno (brillo, mate, texturado; anodizado plata y bronce).
- [ ] Facturas de compra: ¿llegan en papel, PDF digital o Excel? Decide si se importan (y cómo) o se teclean.
- [ ] Cómo se cobra el material (barra, metro o kilo) y si el proveedor factura una superficie mínima de cristal. **Desde H3, perfiles por barras enteras o por metros + merma, a elegir en Ajustes > Presupuestos; el presupuesto muestra los dos importes.**
- [ ] Mano de obra: ¿importe por ventana, por m² o a mano? **Desde H3, importe por ventana en cada línea con un valor por defecto en Ajustes (vacío = 0). Provisional: se suma después del margen (es lo que se cobra, no lleva margen); el propietario aún no lo ha decidido (2026-11-21).**
- [ ] Margen: ¿sobre coste o sobre venta? Descuentos habituales. **Desde H3 es configurable en Ajustes > Presupuestos; mientras no se elija, ninguna línea tiene precio final. El propietario aún no lo ha decidido (2026-11-21).**
- [ ] Tipos de IVA, casos especiales, y cómo se redondea la cuota (confirmar con el gestor). **Confirmado por el propietario (2026-11-21):** 21 % por defecto; las líneas y la base se muestran sin IVA y el IVA se suma al final (total con IVA).
- [x] Documentos: presupuesto P-AAAA-NNNN, trabajo T-AAAA-NNNN, albarán A-AAAA-NNNN, factura F-AAAA-NNNN y rectificativa R-AAAA-NNNN, sin huecos en las facturas (H6). Pendiente que el gestor revise las menciones legales.
- [ ] **Volver a facturar un trabajo tras anular su factura** (auditoría, 2026-11-23). Hoy un trabajo tiene una sola factura ordinaria: si se anula entera con una rectificativa, «Facturar» vuelve a abrir la anulada y la factura corregida solo puede hacerse libre, sin enlace al trabajo; las señales cobradas siguen apuntadas en la anulada. Propuesta: con la ordinaria anulada a cero, «Facturar» crea una nueva del trabajo y las señales pasan a ella. Hace falta que el propietario (y su gestor) lo confirmen, porque afecta a facturas y cobros.
- [ ] Datos fiscales del taller y logotipo: se cargan en Ajustes > Empresa y precios; falta que el propietario los meta.

## Formato de datos
- [x] Enlace entre el perfil de una serie y la referencia de la tarifa: resuelto con la tabla `referencias` (`docs/FORMATO_SERIES.md`, apartado 5).
- [x] Formato de `vidrio` en las series: `docs/FORMATO_SERIES.md`, apartado 6.
- [ ] Pletinas de falleba: fórmula y unidad (¿se venden por metro o por unidad cortada?).
- [ ] Límites de medida por serie (`limites`): mínimos y máximos reales de cada tipología. **Desde la 0.17.0 sirven también para el corte:** con el mínimo, el programa sabrá cuál es la pieza más corta de cada perfil y marcará como desecho lo que quede por debajo (ADR-0026).

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
- [x] Al dar de alta a mano son obligatorios el nombre y un teléfono (propietario, 2026-11-22; antes, ningún dato). Pendiente con el gestor: qué casos admiten factura simplificada o sin NIF.
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
