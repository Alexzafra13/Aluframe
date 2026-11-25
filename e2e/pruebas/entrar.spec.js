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
  await expect(page).toHaveURL(/\/mi-dia$/);
  await expect(page.getByText("No tienes ventanas asignadas")).toBeVisible();
  await expect(page.getByRole("link", { name: "Facturas" })).toHaveCount(0);
  const respuesta = await page.goto("/ajustes/usuarios");
  expect(respuesta.status()).toBe(404);
  await page.goto("/mi-dia");
  await salir(page);
});

test("el dueño da permisos al operario y este ve solo lo permitido", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("link", { name: PROPIETARIO.nombre }).click();
  await page.getByLabel("Contraseña").fill(PROPIETARIO.contrasena);
  await page.getByRole("button", { name: "Entrar" }).click();
  // Sin recarga (ADR-0036) el clic no espera a la respuesta: se espera a haber entrado.
  await expect(page).not.toHaveURL(/\/entrar/);
  await page.goto("/ajustes/usuarios");
  const fila = page.getByRole("row", { name: new RegExp(OPERARIO.nombre) });
  await fila.getByText("Permisos (0 de 4)").click();
  await fila.getByLabel("Ver facturas y cobros").check();
  await fila.getByRole("button", { name: "Guardar permisos" }).click();
  await expect(page.getByRole("status")).toContainText("Permisos guardados");
  await expect(page.getByRole("row", { name: new RegExp(OPERARIO.nombre) }).getByText("Permisos (1 de 4)")).toBeVisible();
  await salir(page);

  await page.goto("/entrar");
  await page.getByRole("link", { name: OPERARIO.nombre }).click();
  for (const cifra of OPERARIO.pin) {
    await page.locator(`.teclado button[data-cifra="${cifra}"]`).click();
  }
  await expect(page).toHaveURL(/\/mi-dia$/);
  await expect(page.getByRole("link", { name: "Clientes" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Facturas" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Presupuestos" })).toHaveCount(0);
  await page.getByRole("link", { name: "Clientes" }).click();
  await expect(page.getByRole("heading", { name: "Clientes" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Nuevo cliente" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Importar clientes" })).toHaveCount(0);
  expect((await page.goto("/presupuestos")).status()).toBe(404);
  expect((await page.goto("/facturas")).status()).toBe(200);
  await page.goto("/mi-dia");
  await salir(page);
});

test("un PIN incorrecto no deja entrar", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("link", { name: OPERARIO.nombre }).click();
  await expect(page.locator(".puntos-pin span.lleno")).toHaveCount(0);
  for (const cifra of "000000") {
    await page.locator(`.teclado button[data-cifra="${cifra}"]`).click();
  }
  await expect(page.getByRole("alert")).toHaveText("El PIN o la contraseña no son correctos.");
  await expect(page.locator(".puntos-pin")).toHaveClass(/agitar/);
});
