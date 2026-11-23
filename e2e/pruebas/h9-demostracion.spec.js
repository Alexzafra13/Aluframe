// La demostración (ADR-0028): se entra desde la pantalla de entrada, se ven datos inventados y
// se sale a los de verdad. Va después de las demás: la base de verdad ya tiene usuarios.
const { test, expect } = require("@playwright/test");

test("se prueba la demostración desde la entrada y se vuelve a los datos de verdad", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("button", { name: "Probar la demostración" }).click();
  await expect(page.getByText("Demostración con datos inventados")).toBeVisible();
  await page.getByRole("link", { name: "Dueño (demo)" }).click();
  await page.getByLabel("Contraseña").fill("demostracion");
  await page.getByRole("button", { name: "Entrar" }).click();
  await expect(page.getByRole("button", { name: "Salir de la demostración" })).toBeVisible();
  // Los estilos cargan también dentro de la demostración (antes daban error 500).
  await expect(page.locator(".franja-demostracion")).toHaveCSS("border-left-style", "solid");

  await page.goto("/clientes");
  await expect(page.getByRole("link", { name: /Lucía Demo Pérez/ }).first()).toBeVisible();
  await page.goto("/presupuestos?estado=caducado");
  await page.getByRole("link", { name: /Carmen Demo López/ }).click();
  await expect(page.getByRole("button", { name: "Renovar" })).toBeVisible();
  await page.goto("/facturas?pendientes=si");
  await expect(page.locator(".etiqueta", { hasText: "Vencida" })).toBeVisible();

  await page.getByRole("button", { name: "Salir de la demostración" }).click();
  await expect(page.getByRole("button", { name: "Probar la demostración" })).toBeVisible();
  await expect(page.getByText("Dueño (demo)")).toHaveCount(0);
});
