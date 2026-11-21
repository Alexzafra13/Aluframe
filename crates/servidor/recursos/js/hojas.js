// Hojas (paneles) al estilo iOS con <dialog> nativo: el navegador gestiona el foco, Esc y la
// accesibilidad; aquí solo se abren y se cierran con su animación de salida.
(function () {
  function cerrar(hoja) {
    if (!hoja.open || hoja.classList.contains("cerrando")) return;
    var reducido = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reducido) { hoja.close(); return; }
    hoja.classList.add("cerrando");
    hoja.addEventListener("animationend", function fin() {
      hoja.removeEventListener("animationend", fin);
      hoja.classList.remove("cerrando");
      hoja.close();
    });
  }

  document.addEventListener("click", function (evento) {
    var abrir = evento.target.closest("[data-abrir-hoja]");
    if (abrir) {
      var hoja = document.getElementById(abrir.dataset.abrirHoja);
      if (hoja) hoja.showModal();
      return;
    }
    var boton = evento.target.closest("[data-cerrar-hoja]");
    if (boton) { cerrar(boton.closest("dialog")); return; }
    if (evento.target.matches("dialog.hoja")) cerrar(evento.target);
  });

  document.addEventListener("cancel", function (evento) {
    if (evento.target.matches("dialog.hoja")) {
      evento.preventDefault();
      cerrar(evento.target);
    }
  }, true);
})();
