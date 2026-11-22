// Rellena provincia, país y localidad al escribir el código postal, y la provincia al escribir
// la localidad, cuyos códigos postales ofrece en la lista del campo (ADR-0020). Solo toca campos vacíos o rellenados por él;
// con varias localidades no elige: las ofrece en la lista del campo.
(function () {
  function campo(id) {
    return id ? document.getElementById(id) : null;
  }

  function sePuedeRellenar(entrada) {
    return entrada && (entrada.value.trim() === "" || entrada.dataset.autorrellenado === "si");
  }

  function poner(entrada, valor) {
    if (!valor || !sePuedeRellenar(entrada)) return;
    entrada.value = valor;
    entrada.dataset.autorrellenado = "si";
  }

  function ofrecer(entrada, opciones) {
    if (!entrada) return;
    let lista = document.getElementById(entrada.id + "-opciones");
    if (!lista) {
      lista = document.createElement("datalist");
      lista.id = entrada.id + "-opciones";
      entrada.after(lista);
      entrada.setAttribute("list", lista.id);
    }
    lista.replaceChildren(...opciones.map(function (texto) {
      const opcion = document.createElement("option");
      opcion.value = texto;
      return opcion;
    }));
  }

  function consultar(parametros) {
    return fetch("/poblaciones?" + new URLSearchParams(parametros), { headers: { Accept: "application/json" } })
      .then(function (r) { return r.ok ? r.json() : null; })
      .catch(function () { return null; });
  }

  function trasCodigoPostal(entrada) {
    const cifras = entrada.value.replace(/\s/g, "");
    if (!/^\d{5}$/.test(cifras)) return;
    const d = entrada.dataset;
    consultar({ cp: cifras }).then(function (p) {
      if (!p || !p.provincia) return;
      poner(campo(d.provincia), p.provincia);
      poner(campo(d.pais), p.pais);
      const localidad = campo(d.localidad);
      ofrecer(localidad, p.localidades);
      if (p.localidades.length === 1) {
        poner(localidad, p.localidades[0]);
      } else if (p.localidades.length > 1 && sePuedeRellenar(localidad)) {
        localidad.value = "";
        delete localidad.dataset.autorrellenado;
        localidad.placeholder = "Elige en la lista: " + p.localidades.slice(0, 2).join(", ") + "…";
      }
    });
  }

  function trasLocalidad(entrada) {
    const d = entrada.dataset;
    const cp = campo(d.cp);
    if (!entrada.value.trim() || (cp && cp.value.trim() !== "")) return;
    consultar({ localidad: entrada.value }).then(function (p) {
      if (!p) return;
      poner(campo(d.provincia), p.provincia);
      poner(campo(d.pais), p.pais);
      // El listado puede no tener todos los códigos de una localidad: se ofrecen, no se escriben.
      ofrecer(cp, p.codigos_postales);
    });
  }

  document.addEventListener("input", function (evento) {
    const entrada = evento.target;
    // Lo que escribe la persona deja de ser «rellenado»: ya no se sustituye.
    delete entrada.dataset.autorrellenado;
    if (entrada.matches("[data-cp-rellena]")) trasCodigoPostal(entrada);
  });
  document.addEventListener("change", function (evento) {
    if (evento.target.matches("[data-localidad-rellena]")) trasLocalidad(evento.target);
  });
})();
