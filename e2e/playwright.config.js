const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { defineConfig } = require("@playwright/test");

const PUERTO = process.env.ALUMFRAME_PUERTO_E2E || "9191";
const EJECUTABLE =
  process.env.ALUMFRAME_BIN || path.join(__dirname, "..", "target", "release", "alumframe");
const DATOS = fs.mkdtempSync(path.join(os.tmpdir(), "alumframe-e2e-"));

module.exports = defineConfig({
  testDir: "pruebas",
  workers: 1,
  fullyParallel: false,
  reporter: "list",
  use: {
    baseURL: `http://127.0.0.1:${PUERTO}`,
    locale: "es-ES",
  },
  webServer: {
    command: `"${EJECUTABLE}"`,
    url: `http://127.0.0.1:${PUERTO}/health`,
    reuseExistingServer: false,
    timeout: 30_000,
    env: {
      ALUMFRAME_MODO: "local",
      ALUMFRAME_PUERTO: PUERTO,
      ALUMFRAME_CARPETA_DATOS: DATOS,
      ALUMFRAME_ABRIR_NAVEGADOR: "no",
    },
  },
});
