document.addEventListener("change", function (evento) {
  const entrada = evento.target;
  if (!entrada.matches("input[type=file][data-volcar-en]") || !entrada.files.length) return;
  const destino = document.getElementById(entrada.dataset.volcarEn);
  entrada.files[0].text().then(function (texto) {
    destino.value = texto;
  });
});
