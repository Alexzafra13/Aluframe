// Navegación sin recarga (hx-boost, ADR-0036): al cambiar de pantalla solo se sustituye el
// cuerpo de la página; estilos, scripts y letras ya están cargados. Aquí se remata lo que la
// recarga hacía sola.
(function () {
  function cuerpoDe(html) {
    var etiqueta = /<body\b[^>]*>/i.exec(html);
    return etiqueta ? etiqueta[0] : "";
  }

  // Como en el navegador, manda la última pantalla pedida: si se pulsa otra cosa antes de que
  // llegue la respuesta de un envío, esa respuesta no tapa la pantalla nueva. La petición
  // anterior no se corta: el servidor termina lo que empezó (un alta, un cobro).
  var ultimaNavegacion = null;
  document.addEventListener("htmx:beforeRequest", function (evento) {
    if (evento.detail.boosted) ultimaNavegacion = evento.detail.xhr;
  });

  document.addEventListener("htmx:beforeSwap", function (evento) {
    var detalle = evento.detail;
    if (!detalle.boosted) return;
    if (detalle.xhr !== ultimaNavegacion) {
      detalle.shouldSwap = false;
      return;
    }
    var cuerpo = cuerpoDe(detalle.xhr.responseText);
    // Una página que no se deja mostrar así (hojas de imprimir, con su propio CSS; conexión
    // del escritorio) se abre con una carga completa.
    if (cuerpo && cuerpo.indexOf('hx-boost="true"') === -1) {
      detalle.shouldSwap = false;
      window.location.assign(detalle.xhr.responseURL || detalle.pathInfo.finalRequestPath);
      return;
    }
    // Los errores de validación (422) y demás también son páginas que hay que enseñar.
    if (detalle.xhr.status >= 400) {
      detalle.shouldSwap = true;
      detalle.isError = false;
    }
    // La transición animada, solo al cambiar de pantalla: la campana o el editor, que se
    // refrescan solos, cortarían la animación a medias.
    detalle.swapOverride = "innerHTML transition:true";
  });

  // Dos cambios de pantalla muy seguidos: el navegador salta la primera animación y lo avisa
  // como error; no lo es.
  window.addEventListener("unhandledrejection", function (evento) {
    var motivo = evento.reason;
    if (motivo && motivo.name === "AbortError" && /Transition was skipped/.test(motivo.message)) {
      evento.preventDefault();
    }
  });

  // Barra de «cargando»: solo si tarda (por Tailscale o con datos móviles); en el taller todo
  // responde en milisegundos y no llega a verse.
  var pendientes = 0;
  var espera = null;
  document.addEventListener("htmx:beforeRequest", function () {
    pendientes += 1;
    if (espera === null) {
      espera = setTimeout(function () { document.documentElement.classList.add("cargando"); }, 200);
    }
  });
  document.addEventListener("htmx:afterRequest", function () {
    pendientes = Math.max(0, pendientes - 1);
    if (pendientes > 0) return;
    clearTimeout(espera);
    espera = null;
    document.documentElement.classList.remove("cargando");
  });

  // La clase del <body> cambia de una pantalla a otra (con menú, entrada, «Mi día»).
  document.addEventListener("htmx:afterSwap", function (evento) {
    if (!evento.detail.boosted) return;
    var clase = /\bclass="([^"]*)"/.exec(cuerpoDe(evento.detail.xhr.responseText));
    document.body.className = clase ? clase[1] : "";
  });
})();
