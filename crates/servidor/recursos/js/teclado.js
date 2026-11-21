document.addEventListener("click", function (evento) {
  const tecla = evento.target.closest(".teclado button[type=button]");
  if (!tecla) return;
  const campo = document.getElementById(tecla.closest(".teclado").dataset.destino);
  if (tecla.hasAttribute("data-borrar")) {
    campo.value = campo.value.slice(0, -1);
  } else if (campo.value.length < 6) {
    campo.value += tecla.dataset.cifra;
  }
  campo.focus();
});
