const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

test.describe.configure({ mode: "serial" });

// Sigue al trabajo de h4-taller (Construcciones Peña, serie ficticia «validada»).
test("un retal del stock entra en el plan y se coge al terminar la orden", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.getByRole("link", { name: "Stock" }).first().click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Stock");
  const entrada = page.locator("form[action='/stock/entrada']");
  await entrada.getByLabel("Artículo").selectOption({ label: "DEMO_HOJA_HORIZONTAL · DEMO perfil hoja horizontal (ficticio)" });
  await entrada.getByLabel(/^Cantidad/).fill("1");
  await entrada.getByLabel(/Largo del retal/).fill("300");
  await entrada.getByLabel(/^Motivo/).fill("Inventario inicial");
  await entrada.getByRole("button", { name: "Anotar entrada" }).click();
  await expect(page.getByRole("status")).toHaveText("Entrada anotada.");
  await expect(page.locator("table")).toContainText("1 × 300 cm");

  await page.goto("/trabajos");
  await page.locator(".tarjeta-trabajo").first().click();
  await page.locator(".repetir-pieza summary").first().click();
  await page.locator(".repetir-pieza").first().getByRole("button", { name: "Anotar" }).click();
  await expect(page.getByRole("status")).toContainText("Pieza repetida anotada");
  await page.getByRole("link", { name: "Orden de taller" }).click();
  await expect(page.getByText(/Coger el retal de 3000 del stock/)).toBeVisible();
  await page.getByRole("link", { name: "Volver al trabajo" }).click();
  await page.getByRole("link", { name: "Terminar orden y descontar stock" }).click();
  await expect(page.getByRole("heading", { name: "Sale del stock" })).toBeVisible();
  await page.getByRole("button", { name: "Terminar orden y descontar" }).click();
  await expect(page.getByRole("status")).toContainText("Orden terminada");
  await page.goto("/stock");
  await expect(page.locator("table")).not.toContainText("1 × 300 cm");
});
