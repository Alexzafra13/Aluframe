const { test, expect } = require("@playwright/test");
const { entrarComoPropietario } = require("./comun");

test.describe.configure({ mode: "serial" });

test("exige nombre y teléfono y guarda el móvil sin prefijo", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.getByRole("link", { name: "Clientes" }).click();
  await expect(page.getByText("Aún no hay clientes")).toBeVisible();
  await page.getByRole("link", { name: "Nuevo cliente" }).click();
  await expect(page.getByLabel("País")).toHaveValue("España");
  await page.getByRole("button", { name: "Guardar", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Nuevo cliente");
  await page.getByLabel(/Nombre o razón social/).fill("Jose Pérez");
  await page.getByLabel(/Teléfono móvil/).fill("612345678");
  await page.getByRole("button", { name: "Guardar", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Jose Pérez");
  await expect(page.getByText("612 345 678")).toBeVisible();
  await expect(page.getByRole("status")).toHaveText("Cliente guardado.");
});

test("el código postal rellena la localidad y la provincia sin adivinar", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/nuevo");
  await page.locator("#fiscal_cp").pressSequentially("06300");
  await expect(page.locator("#fiscal_localidad")).toHaveValue("Zafra");
  await expect(page.locator("#fiscal_provincia")).toHaveValue("Badajoz");
  await expect(page.locator("#fiscal_pais")).toHaveValue("España");
  // Con varias localidades no elige: las ofrece en la lista.
  await page.locator("#fiscal_cp").fill("");
  await page.locator("#fiscal_cp").pressSequentially("04002");
  await expect(page.locator("#fiscal_provincia")).toHaveValue("Almería");
  await expect(page.locator("#fiscal_localidad")).toHaveValue("");
  await expect(page.locator("#fiscal_localidad-opciones option")).toHaveCount(2);
  // Lo escrito a mano no se sustituye.
  await page.locator("#d0_provincia").fill("Mi provincia");
  await page.locator("#d0_cp").pressSequentially("28001");
  await expect(page.locator("#d0_localidad")).toHaveValue("Madrid");
  await expect(page.locator("#d0_provincia")).toHaveValue("Mi provincia");
});

test("comprueba el NIF al escribirlo y guarda varias direcciones", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/nuevo");
  await page.getByLabel("Empresa").check();
  await page.getByLabel(/Nombre o razón social/).fill("Construcciones Peña");
  await page.getByLabel(/Teléfono móvil/).fill("924 55 55 55");
  await page.getByLabel("NIF o CIF").fill("B12345675");
  await expect(page.locator("#nif-mensaje")).toContainText("no coincide");
  await page.getByLabel("NIF o CIF").fill("B12345674");
  await expect(page.locator("#nif-mensaje")).toContainText("Correcto: B12345674");
  await page.locator("#d0_etiqueta").fill("Nave");
  await page.locator("#d0_calle").fill("Polígono Los Caños 4");
  await page.getByRole("button", { name: "+ Añadir otra dirección" }).click();
  await page.locator("#d1_etiqueta").fill("Obra Calle Mayor");
  await page.locator("#d1_contacto_nombre").fill("Luis");
  await page.getByRole("button", { name: "Guardar", exact: true }).click();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Construcciones Peña");
  await expect(page.getByText("Nave · Polígono Los Caños 4")).toBeVisible();
  await expect(page.getByText("Obra Calle Mayor", { exact: true })).toBeVisible();
});

test("busca sin tildes mientras se escribe", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.getByRole("link", { name: "Clientes" }).click();
  await page.getByLabel(/Buscar por nombre/).fill("pena");
  await expect(page.locator("#lista-clientes")).toContainText("Construcciones Peña");
  await expect(page.locator("#lista-clientes")).not.toContainText("Jose Pérez");
});

test("importa clientes de un CSV guardado por Excel en Windows-1252", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/importar");
  const csv = Buffer.from("Nombre;Localidad;Teléfono\nJosé Núñez;Mérida;924 11 22 33\n", "latin1");
  await page.locator("#archivo").setInputFiles({ name: "clientes.csv", mimeType: "text/csv", buffer: csv });
  await expect(page.locator("#texto")).toHaveValue(/José Núñez;Mérida/);
  await page.getByRole("button", { name: "Revisar" }).click();
  await expect(page.getByRole("cell", { name: "José Núñez" })).toBeVisible();
  await page.getByRole("button", { name: /confirmar e importar 1 clientes/ }).click();
  await expect(page.getByRole("status")).toContainText("Importados 1 clientes.");
  await page.getByRole("button", { name: "Deshacer esta importación" }).click();
  await expect(page.getByRole("status")).toContainText("Importación deshecha: 1 clientes quitados.");
});
