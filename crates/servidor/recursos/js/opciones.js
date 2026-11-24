// Muestra cada opción de la ventana solo si se cumplen sus condiciones (data-solo-si,
// ADR-0030): «Abajo» y «Medida de abajo» aparecen al elegir «Parteluces: Sí». Se hace en el
// navegador para no volver a pintar el campo mientras se escribe la medida.
(function () {
  "use strict";

  function cumple(raiz, condicion) {
    return condicion.split(";").every(function (par) {
      var partes = par.split("=");
      var control = raiz.querySelector('[name="opcion.' + partes[0] + '"]');
      return control !== null && control.value === partes[1] && !control.closest("[hidden]");
    });
  }

  function actualizar(raiz) {
    // Dos pasadas: una condición puede depender de otra opción que acaba de ocultarse.
    for (var vuelta = 0; vuelta < 2; vuelta += 1) {
      raiz.querySelectorAll("[data-solo-si]").forEach(function (campo) {
        campo.hidden = !cumple(raiz, campo.dataset.soloSi);
      });
    }
  }

  document.addEventListener("change", function (evento) {
    var raiz = evento.target.closest("#opciones, .rejilla-campos");
    if (raiz) {
      actualizar(raiz);
    }
  });
  document.addEventListener("htmx:afterSettle", function () {
    document.querySelectorAll("#opciones").forEach(actualizar);
  });
})();
