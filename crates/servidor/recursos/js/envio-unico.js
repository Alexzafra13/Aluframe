// Un formulario se envía una sola vez: en la tablet un doble toque en «Registrar señal» o
// «Crear la factura» lo mandaba dos veces (ADR-0036). El servidor también lo frena.
(function () {
  function botones(formulario) {
    return formulario.querySelectorAll("button[type=submit], button:not([type]), input[type=submit]");
  }

  function liberar(formulario) {
    delete formulario.dataset.enviando;
    formulario.removeAttribute("aria-busy");
    botones(formulario).forEach(function (boton) { boton.disabled = false; });
  }

  document.addEventListener("submit", function (evento) {
    var formulario = evento.target;
    if (formulario.method !== "post" || formulario.matches("[data-subir-copia], [data-subir-archivo]")) return;
    if (formulario.dataset.enviando) {
      evento.preventDefault();
      evento.stopImmediatePropagation();
      return;
    }
    formulario.dataset.enviando = "si";
    formulario.setAttribute("aria-busy", "true");
    // Después de que el navegador recoja los datos: un botón desactivado no enviaría su valor.
    setTimeout(function () {
      botones(formulario).forEach(function (boton) { boton.disabled = true; });
    }, 0);
  }, true);

  // Con la navegación sin recarga la página no cambia si la petición falla: se puede reintentar.
  document.addEventListener("htmx:afterRequest", function (evento) {
    var formulario = evento.detail.elt.closest("form");
    if (formulario && formulario.dataset.enviando) liberar(formulario);
  });
  window.addEventListener("pageshow", function () {
    document.querySelectorAll("form[data-enviando]").forEach(liberar);
  });
})();
