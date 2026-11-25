const path = require("node:path");
const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

test.describe.configure({ mode: "serial" });

// Serie ficticia «validada» de las pruebas: las de demostración nunca dan precio final y las
// reales necesitan una tarifa real. No es una serie del taller.
const COMPLETA = path.join(__dirname, "..", "..", "crates", "servidor", "tests", "datos", "serie-prueba-completa.yaml");
const OPERARIO = { nombre: "Luis", pin: "246802" };

test("aceptar un presupuesto crea el trabajo y se asigna a un operario", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/catalogo/series/importar");
  await page.locator("#archivo").setInputFiles(COMPLETA);
  await expect(page.locator("#texto")).toHaveValue(/PRUEBA_COMPLETA/);
  await page.getByRole("button", { name: "Revisar" }).click();
  await page.getByRole("button", { name: "Confirmar e importar" }).click();
  await expect(page).toHaveURL(/importada=/);
  await page.goto("/ajustes/usuarios");
  await page.getByLabel("Nombre").fill(OPERARIO.nombre);
  await page.getByLabel("PIN de 6 cifras").fill(OPERARIO.pin);
  await page.getByRole("button", { name: "Dar de alta" }).click();

  await page.getByRole("link", { name: "Clientes" }).click();
  await page.getByRole("link", { name: "Construcciones Peña" }).click();
  await page.getByRole("button", { name: "Nuevo presupuesto" }).click();
  await page.getByLabel("Serie y tipología").selectOption({ label: "Prueba completa (Demo) · DEMO corredera 2 hojas (ficticia)" });
  await page.getByLabel("Ancho (cm)").fill("150");
  await page.getByLabel("Alto (cm)").fill("120");
  await expect(page.locator("#dibujo .texto-cota").first()).toHaveText("150");
  await page.locator("#form-linea").getByRole("button", { name: "Guardar", exact: true }).click();
  await page.getByRole("button", { name: "Aceptar presupuesto" }).click();
  await page.getByRole("dialog").getByRole("button", { name: "Aceptar presupuesto" }).click();
  await page.getByRole("link", { name: "Ver el trabajo en el taller" }).click();
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Construcciones Peña");
  await page.getByLabel("Operario del trabajo").selectOption({ label: OPERARIO.nombre });
  await page.getByRole("button", { name: "Guardar", exact: true }).click();
  await expect(page.getByRole("status")).toHaveText("Trabajo guardado.");

  await page.getByRole("link", { name: "Orden de taller" }).click();
  await expect(page.getByText("ventana 1 de 1", { exact: false })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Plan de corte del trabajo" })).toBeVisible();
  await page.goto("/");
  await expect(page.locator(".columna").first()).toContainText("Construcciones Peña");
});

test("el operario ve la ventana en su día y marca un paso", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("link", { name: OPERARIO.nombre }).click();
  for (const cifra of OPERARIO.pin) {
    await page.locator(`.teclado button[data-cifra="${cifra}"]`).click();
  }
  await expect(page).toHaveURL(/\/mi-dia$/);
  await page.locator(".tarjeta-ventana").first().click();
  await expect(page.getByRole("heading", { level: 1 })).toContainText("DEMO corredera 2 hojas");
  await expect(page.locator("body")).not.toContainText("€");
  const cortar = page.locator(".paso-operario").first();
  await cortar.getByRole("button", { name: "Terminar" }).click();
  await expect(page.locator(".paso-operario").first()).toHaveClass(/paso-hecho/);
  await page.getByRole("button", { name: "Falta cristal" }).click();
  await expect(page.getByText("Avisado: Falta cristal")).toBeVisible();
});
