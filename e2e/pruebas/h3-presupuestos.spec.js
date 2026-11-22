const path = require("node:path");
const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

test.describe.configure({ mode: "serial" });

// Datos ficticios (fixtures/demo): nunca se cargan en el uso real.
const DEMO = path.join(__dirname, "..", "..", "fixtures", "demo");

test("prepara la serie y la tarifa ficticias y las reglas de precio", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/catalogo/series/importar");
  await page.locator("#archivo").setInputFiles(path.join(DEMO, "serie-demo.yaml"));
  await expect(page.locator("#texto")).toHaveValue(/DEMO_FICTICIA/);
  await page.getByRole("button", { name: "Revisar" }).click();
  await page.getByRole("button", { name: "Confirmar e importar" }).click();
  await page.goto("/catalogo/tarifas/importar");
  await page.getByLabel("Proveedor").selectOption({ label: "Proveedor demo" });
  await page.locator("#archivo").setInputFiles(path.join(DEMO, "tarifa-demo.csv"));
  await expect(page.locator("#texto")).toHaveValue(/DEMO_KIT/);
  await page.getByRole("button", { name: "Revisar" }).click();
  await expect(page.getByText("Paso 2 de 3")).toBeVisible();
  // h2-tarifas ya pudo dejarla vigente: entonces la vista previa dice que no hay cambios.
  const confirmar = page.getByRole("button", { name: "Paso 3: confirmar e importar" });
  if (await confirmar.count()) await confirmar.click();
  await page.goto("/ajustes/presupuestos");
  await page.getByLabel("Sobre el coste (coste × 1,30)").check();
  await page.getByLabel("Margen (%)").fill("30");
  await page.getByLabel("Barras enteras que gasta la ventana").check();
  await page.getByRole("button", { name: "Guardar ajustes" }).click();
  await expect(page.getByRole("status")).toHaveText("Ajustes guardados.");
});

// Las series reales vienen incluidas en el programa (ADR-0014): se elige la ficticia a propósito.
// Guardar vuelve al presupuesto, desde donde se añade la siguiente con «+ Ventana».
async function ventana(page, ancho, alto) {
  await page.getByRole("link", { name: "+ Ventana" }).click();
  await page.getByLabel("Serie y tipología").selectOption({ label: "DEMO_FICTICIA · DEMO corredera 2 hojas (ficticia)" });
  await page.getByLabel("Ancho (cm)").fill(ancho);
  await page.getByLabel("Alto (cm)").fill(alto);
  await expect(page.locator("#dibujo svg.dibujo-ventana")).toBeVisible();
  await expect(page.locator("#dibujo .texto-cota").first()).toHaveText(ancho);
  await page.locator("#form-linea").getByRole("button", { name: "Guardar ventana" }).click();
  await expect(page.getByRole("status")).toHaveText("Guardado en el presupuesto.");
}

/// Criterio de aceptación de H3: 3 ventanas distintas y otro concepto, con dibujo e imprimible.
test("crea un presupuesto de 3 ventanas y otro concepto y lo imprime", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.getByRole("link", { name: "Clientes" }).click();
  await page.getByRole("link", { name: "Construcciones Peña" }).click();
  await page.getByRole("button", { name: "Nuevo presupuesto" }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Construcciones Peña");
  await page.getByRole("link", { name: "Volver sin guardar" }).click();
  await ventana(page, "150", "120");
  await ventana(page, "90", "110");
  await ventana(page, "200", "140");
  await page.getByRole("button", { name: "+ Otro concepto" }).click();
  const hoja = page.getByRole("dialog", { name: "Otro concepto" });
  await expect(hoja).toBeVisible();
  await hoja.getByLabel("Descripción").fill("Instalación");
  await hoja.getByLabel("Precio unitario (€)").fill("80");
  await hoja.getByRole("button", { name: "Añadir al presupuesto" }).click();
  await expect(page.locator(".tarjeta-linea")).toHaveCount(4);
  await expect(page.locator(".tarjeta-linea svg.dibujo-ventana")).toHaveCount(3);
  const total = (await page.locator(".pie-total-importe").textContent()).trim();
  await page.getByRole("link", { name: "Imprimir o guardar PDF" }).click();
  await expect(page.locator(".a4-total dd")).toHaveText(total);
  await expect(page.locator(".a4-linea")).toHaveCount(4);
  const pdf = await page.pdf({ format: "A4" });
  expect(pdf.length).toBeGreaterThan(1000);
});

test("las hojas se cierran con Esc y devuelven el foco", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/presupuestos");
  const boton = page.getByRole("button", { name: "Nuevo presupuesto" });
  await boton.click();
  await expect(page.getByRole("dialog", { name: "Nuevo presupuesto" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog", { name: "Nuevo presupuesto" })).toBeHidden();
  await expect(boton).toBeFocused();
});
