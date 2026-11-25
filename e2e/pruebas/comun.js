const { expect } = require("@playwright/test");

const PROPIETARIO = { nombre: "Dueño del taller", contrasena: "contrasena-segura" };

async function entrarComoPropietario(page) {
  await page.goto("/");
  if (page.url().endsWith("/primer-arranque")) {
    await page.getByLabel("Tu nombre").fill(PROPIETARIO.nombre);
    await page.getByLabel("Contraseña (al menos 10 caracteres)").fill(PROPIETARIO.contrasena);
    await page.getByLabel("Repite la contraseña").fill(PROPIETARIO.contrasena);
    await page.getByRole("button", { name: "Empezar" }).click();
  } else {
    await page.getByRole("link", { name: PROPIETARIO.nombre }).click();
    await page.getByLabel("Contraseña").fill(PROPIETARIO.contrasena);
    await page.getByRole("button", { name: "Entrar" }).click();
  }
  await expect(page.getByRole("heading", { name: "Trabajos" })).toBeVisible();
}

// Con la navegación sin recarga (ADR-0036) un clic no espera a la pantalla nueva: esto sí.
// Espera a que esté puesta la pantalla del enlace pulsado (no otra que acabe a la vez, como la
// campana de avisos o el final de un envío anterior).
async function pulsarYEsperar(page, enlace) {
  const destino = await enlace.getAttribute("href");
  const puesta = page.evaluate(
    (ruta) =>
      new Promise((listo) => {
        document.addEventListener("htmx:afterSettle", function puesta(evento) {
          if (!evento.detail.boosted || evento.detail.pathInfo.requestPath.split("?")[0] !== ruta) return;
          document.removeEventListener("htmx:afterSettle", puesta);
          listo();
        });
      }),
    destino,
  );
  await enlace.click();
  await puesta;
}

module.exports = { PROPIETARIO, entrarComoPropietario, pulsarYEsperar };
