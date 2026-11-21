// Vuelca el archivo elegido en el campo de texto. Excel en España guarda los CSV en
// Windows-1252: si el archivo no es UTF-8 válido se lee con esa codificación (ADR-0012).
document.addEventListener("change", function (evento) {
  const entrada = evento.target;
  if (!entrada.matches("input[type=file][data-volcar-en]") || !entrada.files.length) return;
  const destino = document.getElementById(entrada.dataset.volcarEn);
  entrada.files[0].arrayBuffer().then(function (bytes) {
    let texto;
    try {
      texto = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    } catch (error) {
      texto = new TextDecoder("windows-1252").decode(bytes);
    }
    destino.value = texto;
  });
});
