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
    var texto = boton.querySelector(".menu-texto");
    if (texto) texto.textContent = plegado ? "Desplegar menú" : "Plegar menú";
  }

  document.addEventListener("DOMContentLoaded", function () {
    document.querySelectorAll("[data-plegar-menu]").forEach(rotular);
  });
  document.addEventListener("click", function (evento) {
    var boton = evento.target.closest("[data-plegar-menu]");
    if (!boton) return;
    var plegado = raiz.classList.toggle("menu-plegado");
    try {
      localStorage.setItem(CLAVE, plegado ? "si" : "no");
    } catch (e) {}
    rotular(boton);
  });
})();
