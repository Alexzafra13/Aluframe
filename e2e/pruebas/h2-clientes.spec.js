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
  await page.locator("#fiscal_localidad").focus();
  await expect(page.locator("#fiscal_localidad-lista [role=option]")).toHaveCount(2);
  await page.locator("#fiscal_localidad-lista [role=option]", { hasText: "El Palmer" }).click();
  await expect(page.locator("#fiscal_localidad")).toHaveValue("El Palmer");
  // Lo escrito a mano no se sustituye.
  await page.locator("#d0_provincia").fill("Mi provincia");
  await page.locator("#d0_cp").pressSequentially("28001");
  await expect(page.locator("#d0_localidad")).toHaveValue("Madrid");
  await expect(page.locator("#d0_provincia")).toHaveValue("Mi provincia");
});

test("la localidad sugiere mientras se escribe y al elegirla pone su código postal", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/nuevo");
  await page.locator("#fiscal_localidad").pressSequentially("zaf");
  const zafra = page.locator("#fiscal_localidad-lista [role=option]").first();
  await expect(zafra).toContainText("Zafra");
  await expect(zafra).toContainText("06300 · Badajoz");
  await zafra.click();
  await expect(page.locator("#fiscal_localidad")).toHaveValue("Zafra");
  await expect(page.locator("#fiscal_cp")).toHaveValue("06300");
  await expect(page.locator("#fiscal_provincia")).toHaveValue("Badajoz");
  await expect(page.locator("#fiscal_pais")).toHaveValue("España");
  // Salir del campo no cambia lo elegido (había una aldea Zafra en Ourense).
  await page.locator("#fiscal_provincia").click();
  await expect(page.locator("#fiscal_cp")).toHaveValue("06300");
});

test("una ciudad con varios códigos no se adivina: deja elegir el código", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/nuevo");
  await page.locator("#d0_localidad").pressSequentially("madrid");
  await page.locator("#d0_localidad-lista [role=option]", { hasText: "Madrid" }).first().click();
  await expect(page.locator("#d0_provincia")).toHaveValue("Madrid");
  await expect(page.locator("#d0_cp")).toHaveValue("");
  await page.locator("#d0_cp").focus();
  const codigos = page.locator("#d0_cp-lista [role=option]");
  expect(await codigos.count()).toBeGreaterThan(10);
  await page.locator("#d0_cp").pressSequentially("2801");
  await page.locator("#d0_cp-lista [role=option]", { hasText: "28015" }).click();
  await expect(page.locator("#d0_cp")).toHaveValue("28015");
});

test("la calle sugiere las de la localidad y deja el cursor para el número", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/nuevo");
  await page.locator("#fiscal_cp").pressSequentially("06300");
  await expect(page.locator("#fiscal_localidad")).toHaveValue("Zafra");
  const calle = page.locator("#fiscal_calle");
  await calle.pressSequentially("duques");
  const opcion = page.locator("#fiscal_calle-lista [role=option]").first();
  await expect(opcion).toContainText("Avenida Duques de Feria");
  await expect(opcion).toContainText("Zafra · 06300");
  await opcion.click();
  await expect(calle).toHaveValue("Avenida Duques de Feria, ");
  await calle.pressSequentially("12");
  await expect(calle).toHaveValue("Avenida Duques de Feria, 12");
  await expect(page.locator("#fiscal_calle-lista")).toHaveCount(0);
});

test("con la calle escrita primero se rellenan la localidad y el código", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/nuevo");
  const calle = page.locator("#d0_calle");
  await calle.pressSequentially("avda duques de feria");
  await page.locator("#d0_calle-lista [role=option]", { hasText: "Zafra" }).first().click();
  await expect(page.locator("#d0_localidad")).toHaveValue("Zafra");
  await expect(page.locator("#d0_cp")).toHaveValue("06300");
  await expect(page.locator("#d0_provincia")).toHaveValue("Badajoz");
  // Una localidad de otra provincia no recibe calles de Badajoz.
  await page.locator("#fiscal_localidad").fill("Bilbao");
  await page.locator("#fiscal_calle").pressSequentially("calle");
  await page.waitForTimeout(500);
  await expect(page.locator("#fiscal_calle-lista")).toHaveCount(0);
});

test("la lista de localidades se maneja con el teclado sin enviar el formulario", async ({ page }) => {
  await entrarComoPropietario(page);
  await page.goto("/clientes/nuevo");
  const localidad = page.locator("#fiscal_localidad");
  await localidad.pressSequentially("almendralej");
  await expect(page.locator("#fiscal_localidad-lista [role=option]").first()).toContainText("Almendralejo");
  await localidad.press("ArrowDown");
  await localidad.press("Enter");
  await expect(localidad).toHaveValue("Almendralejo");
  await expect(page.locator("#fiscal_cp")).toHaveValue("06200");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Nuevo cliente");
  await localidad.fill("");
  await localidad.pressSequentially("zaf");
  await expect(page.locator("#fiscal_localidad-lista")).toBeVisible();
  await localidad.press("Escape");
  await expect(page.locator("#fiscal_localidad-lista")).toHaveCount(0);
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
