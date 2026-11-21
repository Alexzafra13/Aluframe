document.addEventListener("click", function (evento) {
  if (evento.target.closest("[data-imprimir]")) window.print();
});
