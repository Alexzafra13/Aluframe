const path = require("node:path");
const { test, expect } = require("@playwright/test");
const { entrarComoPropietario, pulsarYEsperar } = require("./comun");

test.describe.configure({ mode: "serial" });

// Datos ficticios (fixtures/demo): nunca se cargan en el uso real.
const TARIFA_DEMO = path.join(__dirname, "..", "..", "fixtures", "demo", "tarifa-demo.csv");
const CON_ERRORES = "referencia;descripcion;unidad;precio_tarifa;longitud_barra\nX-1;Perfil;barra;0;640\nX-2;Tornillo;saco;3;\n";

async function irA(page, pestana) {
  await pulsarYEsperar(page, page.getByRole("link", { name: "Catálogo" }));
  const secciones = page.getByRole("navigation", { name: "Secciones del catálogo" });
  await pulsarYEsperar(page, secciones.getByRole("link", { name: pestana }));
}

test("da de alta un proveedor", async ({ page }) => {
  await entrarComoPropietario(page);
  await irA(page, "Proveedores");
  await page.getByLabel("Nombre", { exact: true }).fill("Proveedor demo");
  await page.getByLabel("Descuento habitual (%)").fill("10");
  await page.getByRole("button", { name: "Guardar proveedor" }).click();
  await expect(page.getByRole("status")).toHaveText("Proveedor guardado.");
  await expect(page.getByRole("cell", { name: "Proveedor demo" })).toBeVisible();
});

test("una tarifa con errores avisa y no guarda nada", async ({ page }) => {
  await entrarComoPropietario(page);
  await irA(page, "Tarifas");
  await page.getByRole("link", { name: "Importar tarifa" }).click();
  await page.getByLabel("Contenido").fill(CON_ERRORES);
  await page.getByRole("button", { name: "Revisar" }).click();
  await expect(page.getByText("2 errores: corrige el archivo")).toBeVisible();
  await expect(page.getByText("línea 2: el precio «0» debe ser mayor que cero")).toBeVisible();
  await expect(page.getByText("línea 3: unidad desconocida «saco»")).toBeVisible();
  await expect(page.getByRole("button", { name: /confirmar/i })).toHaveCount(0);
  await irA(page, "Artículos");
  await expect(page.getByText("Aún no hay artículos")).toBeVisible();
});

test("importa la tarifa, edita un precio y deshacer lo restaura", async ({ page }) => {
  await entrarComoPropietario(page);
  await irA(page, "Tarifas");
  await page.getByRole("link", { name: "Importar tarifa" }).click();
  await page.locator("#archivo").setInputFiles(TARIFA_DEMO);
  await expect(page.locator("#texto")).toHaveValue(/DEMO_MARCO_LATERAL/);
  await page.getByRole("button", { name: "Revisar" }).click();
  await expect(page.locator(".resumen-importacion")).toContainText("8 artículos nuevos");
  await page.getByRole("button", { name: "Paso 3: confirmar e importar" }).click();
  await expect(page.getByRole("status")).toContainText("Tarifa importada: 8 precios");

  await irA(page, "Artículos");
  await page.getByLabel("Proveedor").selectOption({ label: "Proveedor demo" });
  await page.getByRole("button", { name: "Filtrar" }).click();
  const precio = page.getByLabel("Precio de DEMO_KIT");
  await expect(precio).toHaveValue("9,00");
  await precio.fill("12,50");
  await page.getByRole("button", { name: "Guardar precios" }).click();
  await expect(page.getByRole("status")).toContainText("Precios guardados");
  await expect(page.getByLabel("Precio de DEMO_KIT")).toHaveValue("12,50");

  await irA(page, "Tarifas");
  await page.getByRole("button", { name: "Deshacer" }).click();
  await expect(page.getByRole("status")).toContainText("Versión deshecha");
  await irA(page, "Artículos");
  await page.getByLabel("Proveedor").selectOption({ label: "Proveedor demo" });
  await page.getByRole("button", { name: "Filtrar" }).click();
  await expect(page.getByLabel("Precio de DEMO_KIT")).toHaveValue("9,00");
});
