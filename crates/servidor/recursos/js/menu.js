// Menú lateral plegable a solo iconos. Se carga sin «defer» para aplicar la preferencia antes de
// pintar la página y que no se vea el menú abrirse y cerrarse al cambiar de pantalla.
(function () {
  var CLAVE = "alumframe.menu-plegado";
  var raiz = document.documentElement;
  try {
    if (localStorage.getItem(CLAVE) === "si") raiz.classList.add("menu-plegado");
  } catch (e) {}

  function rotular(boton) {
    var plegado = raiz.classList.contains("menu-plegado");
    boton.setAttribute("aria-expanded", String(!plegado));
    var texto = plegado ? "Desplegar menú" : "Plegar menú";
    boton.setAttribute("aria-label", texto);
    boton.setAttribute("title", texto);
  }

  function rotularTodos() {
    document.querySelectorAll("[data-plegar-menu]").forEach(rotular);
  }
  document.addEventListener("DOMContentLoaded", rotularTodos);
  // Con la navegación sin recarga el menú llega nuevo en cada pantalla.
  document.addEventListener("htmx:afterSettle", rotularTodos);
  document.addEventListener("click", function (evento) {
    var boton = evento.target.closest("[data-plegar-menu]");
    if (!boton) return;
    var plegado = raiz.classList.toggle("menu-plegado");
    try {
      localStorage.setItem(CLAVE, plegado ? "si" : "no");
    } catch (e) {}
    rotular(boton);
  });

  // La campana se cierra al pulsar fuera o con Esc, como cualquier menú desplegable.
  function cerrarCampana(excepto) {
    document.querySelectorAll("[data-campana][open]").forEach(function (c) {
      if (c !== excepto) c.removeAttribute("open");
    });
  }
  document.addEventListener("click", function (evento) {
    cerrarCampana(evento.target.closest("[data-campana]"));
  });
  document.addEventListener("keydown", function (evento) {
    if (evento.key === "Escape") cerrarCampana(null);
  });
})();
