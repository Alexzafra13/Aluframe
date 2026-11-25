# ADR-0036: navegación sin recarga y envío único de formularios

**Estado:** aceptado (2026-11-25, el propietario pide «sensación de SPA» y evitar el doble toque).

## Contexto
- Cada cambio de pantalla recargaba la página entera: un pequeño parpadeo y el menú rehecho. El propietario quiere la sensación de una aplicación (SPA) sin dejar la arquitectura de HTML desde el servidor (ADR-0002).
- Nada impedía enviar dos veces un formulario: en la tablet, un doble toque en «Registrar señal» o «Crear la factura» lo mandaba dos veces y el servidor lo atendía dos veces (reproducido con una prueba: dos clientes iguales).

## Decisión
1. **`hx-boost` en `<body>`** (HTMX ya estaba en el programa): enlaces y formularios piden la página por HTMX y se sustituye el cuerpo; estilos, scripts y letras no se vuelven a cargar. Remates (`recursos/js/navegacion.js`):
   - Una página cuyo `<body>` no lleva `hx-boost="true"` (hojas de imprimir con su CSS, conexión del escritorio) se abre con carga completa. Enlaces de imprimir, descargas y los formularios que suben archivos llevan además `hx-boost="false"`.
   - Las respuestas de error (422 de validación) también se pintan.
   - La clase de `<body>` se copia de la respuesta (entrada, menú, «Mi día»).
   - El destino es siempre `<body>`: un enlace dentro de un trozo con `hx-target` propio (el formulario del editor apunta al dibujo) lo heredaría y metería la página dentro de ese trozo.
   - Manda la última pantalla pedida: la respuesta de una navegación anterior que llegue tarde no se pinta (no se corta, para que el servidor termine lo que empezó).
   - La transición animada solo al cambiar de pantalla, no en los trozos que se refrescan solos.
   - Sin copias de páginas en el navegador para «atrás» (`historyCacheSize: 0`): volverían a enseñar pantallas del dueño tras salir; «atrás» pide la página otra vez.
   - `defaultSettleDelay: 0`: sin el hueco de 20 ms en que la pantalla nueva ya está pero sus elementos HTMX aún no responden.
   - Todos los scripts se cargan una vez en `base.html` y escuchan en `document`; los que preparan algo al cargar lo hacen también tras cada cambio de pantalla (`htmx:afterSettle`).
   - El servidor distingue un trozo de página (`HX-Request`) de la navegación (`HX-Boosted`).
2. **Envío único:**
   - En el navegador (`envio-unico.js`) un formulario `post` no se envía otra vez hasta tener respuesta, y el botón se atenúa.
   - En el servidor (`http/envio_unico.rs`), como respaldo, cada formulario lleva `_envio` (aleatorio en cada página). Si llega repetido, se espera a la primera respuesta y se devuelve la misma redirección sin repetir la acción. Tras un error se puede corregir y volver a enviar.
   - Los trozos de HTMX (el editor recalcula con el mismo formulario) no cuentan.
   - Se recuerda 10 minutos, en memoria.
3. **Barra de «cargando»** fina arriba si una petición pasa de 200 ms.

## Alternativas descartadas
- **React u otra SPA:** duplicaría la lógica en el navegador, añadiría npm y una API (ADR-0002).
- **Extensión `head-support` de HTMX** para mezclar el `<head>` de cada página: otra dependencia para un caso que resuelve la carga completa de las pocas páginas con CSS propio.
- **Precargar al pasar el ratón** (reglas de especulación): solo Chrome y Edge, y con respuestas de 2–6 ms no se nota en el taller.
- **Testigo de un solo uso guardado en la base:** el doble envío llega en milisegundos; en memoria basta y no toca el esquema.

## Consecuencias
- En las E2E, un clic que cambia de pantalla ya no espera a la pantalla nueva. Tras un envío hay que esperar su resultado (la URL o un aviso) antes de `page.goto`, y para navegar está `pulsarYEsperar` en `e2e/pruebas/comun.js`.
- Una pantalla nueva con CSS o scripts propios en `{% block scripts %}` debe quitar la navegación sin recarga (`{% block navegacion %}{% endblock %}`), o cargar sus scripts en `base.html`.
- Un formulario nuevo de acción lleva `_csrf` y `_envio` (se copian juntos).
