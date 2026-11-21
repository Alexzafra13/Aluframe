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

module.exports = { PROPIETARIO, entrarComoPropietario };
