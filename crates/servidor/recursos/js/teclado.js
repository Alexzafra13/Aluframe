// Teclado del PIN. Se prepara al cargar y tras cada cambio de pantalla sin recarga.
function prepararTeclado() {
  const teclado = document.querySelector(".teclado[data-destino]:not([data-listo])");
  if (!teclado) return;
  teclado.dataset.listo = "si";
  const campo = document.getElementById(teclado.dataset.destino);
  const formulario = campo.form;
  const puntos = formulario.querySelectorAll(".puntos-pin span");
  formulario.classList.add("con-teclado");
  campo.inputMode = "none";

  function pintar() {
    puntos.forEach(function (punto, indice) {
      punto.classList.toggle("lleno", indice < campo.value.length);
    });
    if (campo.value.length === 6) {
      setTimeout(function () { formulario.requestSubmit(); }, 120);
    }
  }

  teclado.addEventListener("click", function (evento) {
    const tecla = evento.target.closest("button");
    if (!tecla) return;
    if (tecla.hasAttribute("data-borrar")) {
      campo.value = campo.value.slice(0, -1);
    } else if (campo.value.length < 6) {
      campo.value += tecla.dataset.cifra;
    }
    pintar();
  });
  campo.addEventListener("input", function () {
    campo.value = campo.value.replace(/\D/g, "").slice(0, 6);
    pintar();
  });
  pintar();
}
document.addEventListener("DOMContentLoaded", prepararTeclado);
document.addEventListener("htmx:afterSettle", prepararTeclado);
