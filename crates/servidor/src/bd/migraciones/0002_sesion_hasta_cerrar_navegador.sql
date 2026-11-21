ALTER TABLE sesion ADD COLUMN hasta_cerrar_navegador INTEGER NOT NULL DEFAULT 0
    CHECK (hasta_cerrar_navegador IN (0, 1));
