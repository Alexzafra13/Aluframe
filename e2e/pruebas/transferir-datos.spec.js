const fs = require("node:fs");
const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

// Va la última (orden alfabético): importar sustituye la base que usan las demás pruebas.
test("exporta los datos y los vuelve a importar tras revisar la copia", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/ajustes/datos");
  const descarga = page.waitForEvent("download");
  await page.getByRole("link", { name: "Exportar datos" }).click();
  const copia = await descarga;
  expect(copia.suggestedFilename()).toMatch(/^alumframe-\d{4}-\d{2}-\d{2}\.alumframe$/);
  const ruta = await copia.path();
  expect(fs.statSync(ruta).size).toBeGreaterThan(1000);

  await page.getByLabel("Copia (.alumframe)").setInputFiles({
    name: "no-es-copia.alumframe",
    mimeType: "application/octet-stream",
    buffer: Buffer.from("hola"),
  });
  await page.getByRole("button", { name: "Revisar la copia" }).click();
  await expect(page.getByRole("alert")).toContainText("no es una copia de Aluframe");

  await page.getByLabel("Copia (.alumframe)").setInputFiles(ruta);
  await page.getByRole("button", { name: "Revisar la copia" }).click();
  await expect(page.getByRole("heading", { name: "Importar datos" })).toBeVisible();
  await expect(page.getByRole("row", { name: /Clientes/ })).toBeVisible();
  await page.getByRole("button", { name: "Sustituir los datos por la copia" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "Datos importados. Lo que había se guardó antes en la carpeta de copias.",
  );
});
