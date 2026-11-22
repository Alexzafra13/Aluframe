// Aplicación de escritorio (ADR-0015): mientras la ventana esté abierta, avisa al programa de
// este equipo para que no se apague. En el servidor del taller (otra dirección) no hace nada.
(function () {
  if (location.hostname !== "127.0.0.1" && location.hostname !== "localhost") return;
  function latir() {
    fetch("/escritorio/latido", { cache: "no-store" }).catch(function () {});
  }
  latir();
  setInterval(latir, 20000);
})();
