// Envía un archivo tal cual (la copia de datos, que puede pesar varios MB, o el logotipo) con el
// token CSRF en la cabecera; un formulario normal tendría que codificarlo y no cabría (ADR-0016).
document.addEventListener("submit", function (evento) {
  const formulario = evento.target;
  if (!formulario.matches("form[data-subir-copia], form[data-subir-archivo]")) return;
  const que = formulario.matches("[data-subir-copia]") ? "la copia" : "el archivo";
  evento.preventDefault();
  const archivo = formulario.querySelector("input[type=file]").files[0];
  const aviso = formulario.querySelector("[data-aviso]");
  const boton = formulario.querySelector("button[type=submit]");
  if (!archivo) return;
  const mostrar = function (texto) {
    const esPagina = texto.trim().startsWith("<");
    aviso.textContent = esPagina ? "No se pudo guardar " + que + ". Vuelve a probar." : texto;
    aviso.hidden = false;
    boton.disabled = false;
  };
  aviso.hidden = true;
  boton.disabled = true;
  fetch(formulario.action, {
    method: "POST",
    body: archivo,
    headers: { "Content-Type": "application/octet-stream", "X-CSRF-Token": formulario.dataset.csrf },
  })
    .then(function (respuesta) {
      if (respuesta.ok) {
        window.location.href = respuesta.url;
        return;
      }
      return respuesta.text().then(mostrar);
    })
    .catch(function () {
      mostrar("No se pudo enviar " + que + ". Comprueba la conexión y vuelve a probar.");
    });
});
