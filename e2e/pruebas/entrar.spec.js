const { test, expect } = require("@playwright/test");
const { PROPIETARIO } = require("./comun");

test.describe.configure({ mode: "serial" });

const OPERARIO = { nombre: "Juan", pin: "123456" };

async function salir(page) {
  await page.getByRole("button", { name: "Salir" }).click();
  await expect(page).toHaveURL(/\/entrar$/);
}

test("el primer arranque crea el propietario y muestra su inicio", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveURL(/\/primer-arranque$/);
  await page.getByLabel("Tu nombre").fill(PROPIETARIO.nombre);
  await page.getByLabel("Contraseña (al menos 10 caracteres)").fill(PROPIETARIO.contrasena);
  await page.getByLabel("Repite la contraseña").fill(PROPIETARIO.contrasena);
  await page.getByRole("button", { name: "Empezar" }).click();
  await expect(page.getByRole("heading", { name: "Trabajos" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Facturas" })).toBeVisible();
});

test("el propietario da de alta un operario con PIN", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("link", { name: PROPIETARIO.nombre }).click();
  await page.getByLabel("Contraseña").fill(PROPIETARIO.contrasena);
  await expect(page.getByLabel("Mantener la sesión abierta hasta cerrar el navegador")).not.toBeChecked();
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("link", { name: "Ajustes" }).click();
  await page.getByLabel("Nombre").fill(OPERARIO.nombre);
  await page.getByLabel("PIN de 6 cifras").fill(OPERARIO.pin);
  await page.getByRole("button", { name: "Dar de alta" }).click();
  await expect(page.getByRole("status")).toHaveText("Operario dado de alta.");
  await expect(page.getByRole("cell", { name: OPERARIO.nombre })).toBeVisible();
  await salir(page);
});

test("el operario entra con el teclado de PIN y solo ve su día", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("link", { name: OPERARIO.nombre }).click();
  for (const cifra of OPERARIO.pin) {
    await page.locator(`.teclado button[data-cifra="${cifra}"]`).click();
  }
  await page.getByRole("button", { name: "Entrar" }).click();
  await expect(page).toHaveURL(/\/mi-dia$/);
  await expect(page.getByText("No tienes ventanas asignadas")).toBeVisible();
  await expect(page.getByRole("link", { name: "Facturas" })).toHaveCount(0);
  const respuesta = await page.goto("/ajustes/usuarios");
  expect(respuesta.status()).toBe(404);
  await page.goto("/mi-dia");
  await salir(page);
});

test("un PIN incorrecto no deja entrar", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("link", { name: OPERARIO.nombre }).click();
  await page.getByLabel("PIN").fill("000000");
  await page.getByRole("button", { name: "Entrar" }).click();
  await expect(page.getByRole("alert")).toHaveText("El PIN o la contraseña no son correctos.");
});
