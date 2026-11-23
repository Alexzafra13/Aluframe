const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

test.describe.configure({ mode: "serial" });

const OPERARIO = { nombre: "Pepe", pin: "975310" };

async function salir(page) {
  await page.getByRole("button", { name: "Salir" }).first().click();
  await expect(page).toHaveURL(/\/entrar$/);
}

async function entrarComoOperario(page) {
  await page.goto("/entrar");
  await page.getByRole("link", { name: OPERARIO.nombre }).click();
  for (const cifra of OPERARIO.pin) {
    await page.locator(`.teclado button[data-cifra="${cifra}"]`).click();
  }
  await expect(page).toHaveURL(/\/mi-dia$/);
}

test("el dueño manda una tarea y el operario la ve en Mi día y la marca hecha", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/ajustes/usuarios");
  await page.getByLabel("Nombre").fill(OPERARIO.nombre);
  await page.getByLabel("PIN de 6 cifras").fill(OPERARIO.pin);
  await page.getByRole("button", { name: "Dar de alta" }).click();
  await page.goto("/clientes/nuevo");
  await page.getByLabel(/Nombre o razón social/).fill("Rosa Gil");
  await page.getByLabel(/Teléfono móvil/).fill("611222333");
  await page.getByRole("button", { name: "Guardar", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Rosa Gil");

  await page.getByRole("link", { name: "Apuntes" }).first().click();
  await page.getByRole("link", { name: "Nuevo apunte" }).click();
  await page.getByLabel(/Qué hay que hacer/).fill("Cambiar la cerradura del portal");
  await page.getByLabel("Para").selectOption({ label: `Tarea para ${OPERARIO.nombre}` });
  await page.getByRole("button", { name: "Guardar apunte" }).click();
  await expect(page.getByRole("status")).toHaveText("Apunte guardado.");
  await expect(page.getByText(`Tú → tarea para ${OPERARIO.nombre}`)).toBeVisible();
  await salir(page);

  await entrarComoOperario(page);
  await expect(page.getByRole("heading", { name: "Tareas" })).toBeVisible();
  const tarea = page.locator(".apunte", { hasText: "Cambiar la cerradura del portal" });
  await tarea.getByRole("button", { name: "Hecho" }).click();
  await expect(page).toHaveURL(/\/mi-dia$/);
  await expect(page.getByText("Cambiar la cerradura del portal")).toHaveCount(0);
});

test("el operario apunta un encargo desde la ficha y el dueño lo pasa a presupuesto", async ({ page }) => {
  await entrarComoOperario(page);
  await page.getByRole("link", { name: "Clientes" }).click();
  await page.getByRole("link", { name: "Rosa Gil" }).click();
  await page.getByRole("link", { name: "Nuevo apunte" }).click();
  await expect(page.getByText("Cliente:")).toBeVisible();
  await page.getByLabel(/Qué hay que hacer/).fill("Quiere presupuesto de 2 mosquiteras");
  await page.getByLabel("Medidas tomadas").fill("Cocina: 120 × 100\nBaño: 60 × 80");
  await expect(page.getByLabel("Para")).toHaveCount(0);
  await page.getByRole("button", { name: "Guardar apunte" }).click();
  const encargo = page.locator(".apunte", { hasText: "2 mosquiteras" });
  await expect(encargo).toContainText("Cocina: 120 × 100");
  await expect(encargo.getByRole("button", { name: "Hacer presupuesto" })).toHaveCount(0);
  await expect(encargo.getByRole("button", { name: "Hecho" })).toHaveCount(0);
  await salir(page);

  await entrarComoPropietario(page);
  await expect(page.getByRole("link", { name: /Pepe: Quiere presupuesto de 2 mosquiteras \(Rosa Gil\)/ })).toBeVisible();
  await page.getByRole("link", { name: /Pepe: Quiere presupuesto/ }).click();
  const tarjeta = page.locator(".apunte", { hasText: "2 mosquiteras" });
  await tarjeta.getByRole("button", { name: "Hacer presupuesto" }).click();
  await expect(page).toHaveURL(/\/presupuestos\/\d+\/lineas\/nueva$/);
  await page.goto("/apuntes?cerrados=si");
  await expect(page.locator(".apunte", { hasText: "2 mosquiteras" })).toContainText("Presupuestado");
});
