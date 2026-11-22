const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

test.describe.configure({ mode: "serial" });

// Sigue al trabajo de h4-taller y h5-stock: señal, factura emitida, cobro parcial y albarán.
test("señal por Bizum, factura emitida con número, cobro del resto y albarán", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/trabajos");
  await page.locator(".tarjeta-trabajo").first().click();
  const senal = page.locator("form[action$='/senal']");
  await senal.getByLabel("Importe (€)").fill("50");
  await senal.getByLabel("Bizum").check();
  await senal.getByRole("button", { name: "Registrar señal" }).click();
  await expect(page.getByRole("status")).toContainText("Señal registrada");

  await page.getByRole("button", { name: "Crear la factura" }).click();
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Borrador");
  const confirmar = page.getByLabel(/no tiene NIF/);
  if (await confirmar.count()) await confirmar.check();
  await page.getByRole("button", { name: "Emitir factura" }).click();
  await expect(page.getByRole("status")).toContainText("Factura emitida");
  await expect(page.getByRole("heading", { level: 1 })).toContainText(/F-\d{4}-0001/);
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Parcial");

  const cobro = page.locator("form[action$='/cobros']");
  await cobro.getByLabel("Tarjeta").check();
  await cobro.getByRole("button", { name: "Registrar cobro" }).click();
  await expect(page.getByRole("status")).toHaveText("Cobro registrado.");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Pagada");

  await page.getByRole("link", { name: /Imprimir/ }).first().click();
  await expect(page.getByText("Base imponible")).toBeVisible();
  await page.goto("/facturas");
  await expect(page.locator("table")).toContainText(/F-\d{4}-0001/);

  await page.goto("/trabajos");
  await page.locator(".tarjeta-trabajo").first().click();
  await page.getByRole("button", { name: "Crear albarán de entrega" }).click();
  await expect(page).toHaveURL(/\/albaranes\/\d+\/imprimir/);
  await expect(page.getByText(/A-\d{4}-0001/).first()).toBeVisible();
});
