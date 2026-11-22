const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

const PENDRIVE = process.env.ALUMFRAME_E2E_PENDRIVE;

test("se prepara un disco, se copia y una copia se puede revisar para restaurarla", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/ajustes/copias");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Copias de seguridad");
  const disco = page.locator("form[action='/ajustes/copias/preparar']").filter({ hasText: PENDRIVE });
  await disco.getByRole("button", { name: "Usar este disco para las copias" }).click();
  await expect(page.getByRole("status")).toContainText("Disco preparado");
  await page.getByRole("button", { name: "Copiar ahora" }).click();
  await expect(page.getByRole("status")).toHaveText("Copia hecha y comprobada.");
  const enDisco = page.getByRole("region", { name: /En el disco/ });
  await expect(enDisco.locator("tbody tr")).toHaveCount(1);
  await page.goto("/");
  await expect(page.getByText("Ver las copias")).toHaveCount(0);
  await page.goto("/ajustes/copias");
  await enDisco.getByRole("button", { name: /Revisar la copia/ }).click();
  await expect(page).toHaveURL(/\/ajustes\/datos\/importar\/[0-9a-f]{64}/);
});
