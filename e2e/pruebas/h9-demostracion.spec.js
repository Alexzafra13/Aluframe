// La demostración (ADR-0028): se entra desde la pantalla de entrada sin contraseña, se ven datos
// inventados, se cambia al operario sin PIN y se sale a los de verdad. Va después de las demás:
// la base de verdad ya tiene usuarios.
const { test, expect } = require("@playwright/test");

test("se prueba la demostración sin contraseña y se vuelve a los datos de verdad", async ({ page }) => {
  await page.goto("/entrar");
  await page.getByRole("button", { name: "Probar la demostración" }).click();
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

  await page.getByRole("button", { name: "Cambiar de usuario" }).click();
  await expect(page.getByText("Elige con quién entrar, sin contraseña")).toBeVisible();
  await page.getByRole("button", { name: "Juan (demo)" }).click();
  await expect(page).toHaveURL(/\/mi-dia$/);
  await expect(page.locator(".usuario-actual")).toHaveText("Juan (demo)");

  await page.getByRole("button", { name: "Salir de la demostración" }).click();
  await expect(page.getByRole("button", { name: "Probar la demostración" })).toBeVisible();
  await expect(page.getByText("Dueño (demo)")).toHaveCount(0);
});
