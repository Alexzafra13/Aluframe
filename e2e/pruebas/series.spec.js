const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

// Las series reales vienen incluidas en el programa y se cargan al arrancar (ADR-0014).
test("las series reales vienen incluidas y se prueba una ventana sin importar nada", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.getByRole("link", { name: "Catálogo" }).click();
  await page.getByRole("link", { name: "Series", exact: true }).click();
  await expect(page.getByRole("row", { name: /SOPENA_V94/ })).toBeVisible();

  await page.getByRole("row", { name: /ALUPROM_21/ }).getByRole("link", { name: "Probar una ventana" }).click();
  await page.getByLabel("Ancho (cm)").fill("150");
  await page.getByLabel("Alto (cm)").fill("120");
  const resultado = page.locator("#resultado");
  await expect(resultado.getByRole("cell", { name: "145,9" }).first()).toBeVisible();
  await expect(resultado.getByRole("cell", { name: "73", exact: true })).toBeVisible();
  // Desde 0.8.1 lleva cristal: dos de (150 / 2 − 9,8) × (120 − 13,1) cm; ya no sale incompleta.
  await expect(resultado.getByRole("row", { name: /CRISTAL/ })).toContainText("65,2");
  await expect(resultado).not.toContainText("Incompleta: no admite precio final");
  await expect(resultado).toContainText("HOJA_RODAMIENTO: 1 barra");
});
