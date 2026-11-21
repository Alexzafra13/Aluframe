const path = require("node:path");
const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

const SERIE = path.join(__dirname, "..", "..", "data", "series", "aluprom-21.yaml");

test("el propietario importa una serie real y prueba una ventana", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.getByRole("link", { name: "Catálogo" }).click();
  await page.getByRole("link", { name: "Series", exact: true }).click();
  await page.getByRole("link", { name: "Importar serie" }).click();
  await page.locator("#archivo").setInputFiles(SERIE);
  await expect(page.locator("#texto")).toHaveValue(/ALUPROM_21/);
  await page.getByRole("button", { name: "Revisar" }).click();
  await expect(page.getByText("Se guardará como")).toContainText("versión 1");
  await page.getByRole("button", { name: "Confirmar e importar" }).click();
  await expect(page.getByRole("status")).toHaveText("Serie ALUPROM_21 importada como versión 1.");

  await page.getByRole("row", { name: /ALUPROM_21/ }).getByRole("link", { name: "Probar una ventana" }).click();
  await page.getByLabel("Ancho (cm)").fill("150");
  await page.getByLabel("Alto (cm)").fill("120");
  const resultado = page.locator("#resultado");
  await expect(resultado.getByRole("cell", { name: "145,9" }).first()).toBeVisible();
  await expect(resultado.getByRole("cell", { name: "73", exact: true })).toBeVisible();
  await expect(resultado).toContainText("Incompleta: no admite precio final");
  await expect(resultado).toContainText("HOJA_RODAMIENTO: 1 barra");
});
