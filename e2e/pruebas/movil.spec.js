const { test, expect, devices } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

// Pantallas del propietario que ya existen; cada hito añade las suyas.
const PANTALLAS = [
  "/", "/clientes", "/clientes/nuevo", "/clientes/importar", "/presupuestos",
  "/catalogo/articulos", "/catalogo/proveedores", "/catalogo/acabados", "/catalogo/tarifas",
  "/catalogo/series", "/catalogo/series/importar", "/catalogo/tarifas/importar",
  "/ajustes/usuarios", "/ajustes/taller", "/ajustes/presupuestos", "/ajustes/datos",
];

// El tamaño y el tacto de cada aparato, con el navegador de la configuración (Chromium).
function aparato(nombre) {
  const { defaultBrowserType, ...resto } = devices[nombre];
  return resto;
}

for (const nombre of ["iPhone 13", "iPad (gen 7)"]) {
  test.describe(`en ${nombre}`, () => {
    test.use(aparato(nombre));

    test("ninguna pantalla se sale por los lados", async ({ page }) => {
      await entrarComoPropietario(page);
      for (const ruta of PANTALLAS) {
        await page.goto(ruta);
        const sobra = await page.evaluate(
          () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
        );
        expect(sobra, `${ruta} se sale ${sobra} px por la derecha`).toBeLessThanOrEqual(0);
      }
    });
  });
}

test.describe("en el móvil", () => {
  test.use(aparato("iPhone 13"));

  test("se navega con la barra de pestañas y la hoja «Más»", async ({ page }) => {
    await entrarComoPropietario(page);
    const barra = page.getByRole("navigation", { name: "Menú principal" }).filter({ visible: true });
    await expect(barra).toHaveCount(1);
    await barra.getByRole("link", { name: "Presupuestos" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "Presupuestos" })).toBeVisible();
    await expect(barra.getByRole("link", { name: "Presupuestos" })).toHaveAttribute("aria-current", "page");

    await barra.getByRole("button", { name: "Más" }).click();
    const mas = page.getByRole("dialog", { name: "Más" });
    await expect(mas).toBeVisible();
    await mas.getByRole("link", { name: "Catálogo" }).click();
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(page).toHaveURL(/\/catalogo/);
    await expect(barra.getByRole("button", { name: "Más" })).toHaveAttribute("aria-current", "page");
  });
});

test("en el ordenador se usa el menú lateral y no la barra de pestañas", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await entrarComoPropietario(page);
  const visibles = page.getByRole("navigation", { name: "Menú principal" }).filter({ visible: true });
  await expect(visibles).toHaveCount(1);
  await expect(visibles.getByRole("link", { name: "Stock" })).toBeVisible();
});
