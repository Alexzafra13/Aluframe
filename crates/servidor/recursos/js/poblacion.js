// Calle, código postal, localidad, provincia y país se rellenan entre sí (ADR-0020 y ADR-0023):
// - Al escribir la calle salen las del callejero (en la localidad escrita, o en toda la provincia);
//   al elegir una se ponen su localidad, su provincia y su código postal si tiene uno solo.
// - Al escribir el código postal se ponen la provincia, el país y la localidad si solo hay una;
//   con varias, se ofrecen en la lista de la localidad.
// - Al escribir la localidad salen las más parecidas; al elegir una se ponen su provincia, su país
//   y su código postal si solo tiene uno. Con varios, el código queda vacío y los ofrece.
// Solo se tocan campos vacíos o rellenados por este script; lo escrito a mano no se sustituye.
(function () {
  const RETARDO_MS = 150;
  let abierta = null;
  let consultaEnCurso = 0;
  let temporizador = null;

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

  function sinTildes(texto) {
    return texto.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().trim();
  }

  function consultar(ruta, parametros) {
    return fetch(ruta + "?" + new URLSearchParams(parametros), { headers: { Accept: "application/json" } })
      .then(function (r) { return r.ok ? r.json() : null; })
      .catch(function () { return null; });
  }

  // --- Lista de opciones bajo el campo -------------------------------------------------------

  function cerrarLista() {
    if (!abierta) return;
    abierta.lista.remove();
    abierta.entrada.setAttribute("aria-expanded", "false");
    abierta.entrada.removeAttribute("aria-activedescendant");
    abierta = null;
  }

  function crearOpcion(entrada, opcion, i) {
    const li = document.createElement("li");
    li.id = entrada.id + "-opcion-" + i;
    li.className = "sugerencia";
    li.setAttribute("role", "option");
    const titulo = document.createElement("span");
    titulo.className = "sugerencia-titulo";
    titulo.textContent = opcion.titulo;
    li.append(titulo);
    if (opcion.detalle) {
      const detalle = document.createElement("span");
      detalle.className = "sugerencia-detalle";
      detalle.textContent = opcion.detalle;
      li.append(detalle);
    }
    // Sin esto, el campo perdería el foco antes del clic y la lista se cerraría.
    li.addEventListener("mousedown", function (e) { e.preventDefault(); });
    li.addEventListener("click", function () { elegir(i); });
    return li;
  }

  function mostrarLista(entrada, opciones) {
    cerrarLista();
    if (!opciones.length || document.activeElement !== entrada) return;
    const lista = document.createElement("ul");
    lista.id = entrada.id + "-lista";
    lista.className = "lista-sugerencias";
    lista.setAttribute("role", "listbox");
    lista.append(...opciones.map(function (o, i) { return crearOpcion(entrada, o, i); }));
    entrada.after(lista);
    entrada.setAttribute("role", "combobox");
    entrada.setAttribute("aria-autocomplete", "list");
    entrada.setAttribute("aria-controls", lista.id);
    entrada.setAttribute("aria-expanded", "true");
    abierta = { entrada: entrada, lista: lista, opciones: opciones, activa: -1 };
    // Con el campo al pie de la pantalla, la lista quedaría oculta: se desplaza lo justo.
    lista.scrollIntoView({ block: "nearest" });
  }

  function marcar(indice) {
    const total = abierta.opciones.length;
    abierta.activa = (indice + total) % total;
    abierta.lista.querySelectorAll(".sugerencia").forEach(function (li, i) {
      li.setAttribute("aria-selected", i === abierta.activa ? "true" : "false");
      if (i === abierta.activa) {
        li.scrollIntoView({ block: "nearest" });
        abierta.entrada.setAttribute("aria-activedescendant", li.id);
      }
    });
  }

  function elegir(indice) {
    if (!abierta) return;
    const opcion = abierta.opciones[indice];
    cerrarLista();
    if (opcion) opcion.elegir();
  }

  // Opciones guardadas en el campo (los códigos de una ciudad, las localidades de un código):
  // salen al entrar en él y se filtran con lo que se escribe.
  function ofrecer(entrada, opciones) {
    if (!entrada) return;
    entrada.ofrecidas = opciones;
    if (document.activeElement === entrada) mostrarOfrecidas(entrada);
  }

  function coincidentes(entrada) {
    const escrito = sinTildes(entrada.value);
    return (entrada.ofrecidas || []).filter(function (o) { return sinTildes(o.titulo).includes(escrito); });
  }

  function mostrarOfrecidas(entrada) {
    mostrarLista(entrada, coincidentes(entrada));
  }

  // --- Código postal --------------------------------------------------------------------------

  function opcionDeCodigo(cp, codigo) {
    return { titulo: codigo, elegir: function () { cp.value = codigo; delete cp.dataset.autorrellenado; } };
  }

  function trasCodigoPostal(entrada) {
    const cifras = entrada.value.replace(/\s/g, "");
    if (!/^\d{5}$/.test(cifras)) return;
    const d = entrada.dataset;
    consultar("/poblaciones", { cp: cifras }).then(function (p) {
      if (!p || !p.provincia) return;
      poner(campo(d.provincia), p.provincia);
      poner(campo(d.pais), p.pais);
      const localidad = campo(d.localidad);
      ofrecer(localidad, p.localidades.length < 2 ? [] : p.localidades.map(function (nombre) {
        return { titulo: nombre, elegir: function () { localidad.value = nombre; localidad.dataset.elegida = "si"; } };
      }));
      if (p.localidades.length === 1) {
        poner(localidad, p.localidades[0]);
      } else if (p.localidades.length > 1 && sePuedeRellenar(localidad)) {
        localidad.value = "";
        delete localidad.dataset.autorrellenado;
        localidad.placeholder = "Elige en la lista: " + p.localidades.slice(0, 2).join(", ") + "…";
      }
    });
  }

  // --- Localidad ------------------------------------------------------------------------------

  function elegirLocalidad(entrada, s) {
    const d = entrada.dataset;
    entrada.value = s.localidad;
    delete entrada.dataset.autorrellenado;
    entrada.dataset.elegida = "si";
    poner(campo(d.provincia), s.provincia);
    poner(campo(d.pais), s.pais);
    ponerCodigos(campo(d.cp), s.localidad, s.codigos_postales);
  }

  // Con un solo código se escribe; con varios no se adivina: el campo queda vacío y los ofrece.
  function ponerCodigos(cp, localidad, codigos) {
    if (!cp) return;
    ofrecer(cp, codigos.length < 2 ? [] : codigos.map(function (c) { return opcionDeCodigo(cp, c); }));
    if (codigos.length === 1) {
      poner(cp, codigos[0]);
    } else if (codigos.length > 1 && sePuedeRellenar(cp)) {
      cp.value = "";
      delete cp.dataset.autorrellenado;
      cp.placeholder = "Elige el código de " + localidad;
    }
  }

  function detalleDeSugerencia(s) {
    const codigos = s.codigos_postales;
    const codigo = codigos.length === 1 ? codigos[0] : codigos.length + " códigos";
    return codigo + " · " + s.provincia;
  }

  function sugerirLocalidades(entrada) {
    const escrito = entrada.value;
    const numero = ++consultaEnCurso;
    consultar("/poblaciones/sugerencias", { texto: escrito }).then(function (lista) {
      // Una respuesta lenta de algo ya borrado no debe pisar la lista de lo escrito ahora.
      if (!lista || numero !== consultaEnCurso || entrada.value !== escrito) return;
      mostrarLista(entrada, lista.map(function (s) {
        return { titulo: s.localidad, detalle: detalleDeSugerencia(s), elegir: function () { elegirLocalidad(entrada, s); } };
      }));
    });
  }

  function alEscribirLocalidad(entrada) {
    clearTimeout(temporizador);
    // Si el código postal ya dejó elegir entre unas pocas, se filtran esas; si no casa ninguna, se busca en todas.
    if (coincidentes(entrada).length) return mostrarOfrecidas(entrada);
    if (sinTildes(entrada.value).length < 2) return cerrarLista();
    temporizador = setTimeout(function () { sugerirLocalidades(entrada); }, RETARDO_MS);
  }

  // Si se sale del campo sin elegir en la lista, se busca el nombre tal cual está escrito.
  function trasLocalidad(entrada) {
    const d = entrada.dataset;
    const cp = campo(d.cp);
    if (d.elegida === "si" || !entrada.value.trim() || (cp && cp.value.trim() !== "")) return;
    consultar("/poblaciones", { localidad: entrada.value }).then(function (p) {
      if (!p) return;
      poner(campo(d.provincia), p.provincia);
      poner(campo(d.pais), p.pais);
      if (p.provincia) ponerCodigos(cp, entrada.value.trim(), p.codigos_postales);
    });
  }

  // --- Calle (ADR-0023) ----------------------------------------------------------------------

  function elegirCalle(entrada, s) {
    const d = entrada.dataset;
    // El número lo escribe la persona: se deja la coma y el cursor detrás.
    entrada.value = s.calle + ", ";
    entrada.setSelectionRange(entrada.value.length, entrada.value.length);
    poner(campo(d.localidad), s.localidad);
    poner(campo(d.provincia), s.provincia);
    poner(campo(d.pais), s.pais);
    ponerCodigos(campo(d.cp), s.calle, s.codigos_postales);
  }

  function detalleDeCalle(s) {
    const codigos = s.codigos_postales;
    if (!codigos.length) return s.localidad;
    return s.localidad + " · " + (codigos.length === 1 ? codigos[0] : codigos.length + " códigos");
  }

  function sugerirCalles(entrada) {
    const escrito = entrada.value;
    const d = entrada.dataset;
    const valor = function (id) { const c = campo(id); return c ? c.value : ""; };
    const numero = ++consultaEnCurso;
    consultar("/poblaciones/calles", { texto: escrito, localidad: valor(d.localidad), cp: valor(d.cp) }).then(function (lista) {
      if (!lista || numero !== consultaEnCurso || entrada.value !== escrito) return;
      mostrarLista(entrada, lista.map(function (s) {
        return { titulo: s.calle, detalle: detalleDeCalle(s), elegir: function () { elegirCalle(entrada, s); } };
      }));
    });
  }

  // Tras la coma va el número: ahí ya no hay nada que sugerir.
  function alEscribirCalle(entrada) {
    clearTimeout(temporizador);
    if (entrada.value.includes(",") || sinTildes(entrada.value).length < 2) return cerrarLista();
    temporizador = setTimeout(function () { sugerirCalles(entrada); }, RETARDO_MS);
  }

  // --- Eventos --------------------------------------------------------------------------------

  document.addEventListener("input", function (evento) {
    const entrada = evento.target;
    // Lo que escribe la persona deja de ser «rellenado» o «elegido»: ya no se sustituye.
    delete entrada.dataset.autorrellenado;
    delete entrada.dataset.elegida;
    if (entrada.matches("[data-cp-rellena]")) {
      if (entrada.ofrecidas) mostrarOfrecidas(entrada);
      trasCodigoPostal(entrada);
    }
    if (entrada.matches("[data-localidad-rellena]")) alEscribirLocalidad(entrada);
    if (entrada.matches("[data-calle-rellena]")) alEscribirCalle(entrada);
  });
  document.addEventListener("focusin", function (evento) {
    const entrada = evento.target;
    if (entrada.ofrecidas && entrada.matches("[data-cp-rellena], [data-localidad-rellena]")) mostrarOfrecidas(entrada);
  });
  document.addEventListener("focusout", function (evento) {
    if (abierta && evento.target === abierta.entrada) cerrarLista();
  });
  document.addEventListener("change", function (evento) {
    if (evento.target.matches("[data-localidad-rellena]")) trasLocalidad(evento.target);
  });
  document.addEventListener("keydown", function (evento) {
    if (!abierta || evento.target !== abierta.entrada) return;
    if (evento.key === "ArrowDown") marcar(abierta.activa + 1);
    else if (evento.key === "ArrowUp") marcar(abierta.activa - 1);
    else if (evento.key === "Enter" && abierta.activa >= 0) elegir(abierta.activa);
    else if (evento.key === "Escape") cerrarLista();
    else return;
    // Intro no envía el formulario y las flechas no mueven el cursor mientras la lista está abierta.
    evento.preventDefault();
  });
})();
