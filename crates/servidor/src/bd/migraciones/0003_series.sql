CREATE TABLE serie (
    id INTEGER PRIMARY KEY,
    codigo TEXT NOT NULL UNIQUE CHECK (length(codigo) BETWEEN 1 AND 40),
    fabricante TEXT NOT NULL,
    creada_en TEXT NOT NULL
) STRICT;

CREATE TABLE serie_version (
    id INTEGER PRIMARY KEY,
    serie_id INTEGER NOT NULL REFERENCES serie (id) ON DELETE RESTRICT,
    numero INTEGER NOT NULL CHECK (numero >= 1),
    estado TEXT NOT NULL CHECK (estado IN ('borrador', 'validada', 'demo')),
    origen_yaml TEXT NOT NULL,
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64),
    avisos INTEGER NOT NULL CHECK (avisos >= 0),
    importada_por INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    creada_en TEXT NOT NULL,
    UNIQUE (serie_id, numero)
) STRICT;

CREATE TABLE serie_referencia (
    serie_version_id INTEGER NOT NULL REFERENCES serie_version (id) ON DELETE RESTRICT,
    codigo TEXT NOT NULL,
    referencia_texto TEXT NOT NULL,
    proveedor_texto TEXT,
    PRIMARY KEY (serie_version_id, codigo)
) STRICT;

CREATE TRIGGER serie_version_no_se_modifica BEFORE UPDATE ON serie_version
BEGIN
    SELECT RAISE(ABORT, 'Una versión de serie no se modifica: se importa una nueva');
END;

CREATE TRIGGER serie_version_no_se_borra BEFORE DELETE ON serie_version
BEGIN
    SELECT RAISE(ABORT, 'Una versión de serie no se borra');
END;
