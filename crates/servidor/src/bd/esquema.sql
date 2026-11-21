-- Esquema único de AlumFrame. Hasta el primer despliegue real se amplía aquí directamente,
-- sin migraciones (ADR-0011). Cambiarlo obliga a borrar las bases de desarrollo.

CREATE TABLE esquema (
    huella TEXT PRIMARY KEY CHECK (length(huella) = 64),
    creado_en TEXT NOT NULL
) STRICT;

CREATE TABLE usuario (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(nombre)) BETWEEN 1 AND 60),
    rol TEXT NOT NULL CHECK (rol IN ('propietario', 'operario')),
    clave_hash TEXT NOT NULL,
    activo INTEGER NOT NULL DEFAULT 1 CHECK (activo IN (0, 1)),
    intentos_fallidos INTEGER NOT NULL DEFAULT 0 CHECK (intentos_fallidos >= 0),
    bloqueado_hasta TEXT,
    creado_en TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
) STRICT;

CREATE TABLE sesion (
    id TEXT PRIMARY KEY,
    usuario_id INTEGER NOT NULL REFERENCES usuario (id) ON DELETE RESTRICT,
    creada_en TEXT NOT NULL,
    ultima_actividad_en TEXT NOT NULL,
    caduca_en TEXT NOT NULL,
    token_csrf TEXT NOT NULL,
    hasta_cerrar_navegador INTEGER NOT NULL DEFAULT 0 CHECK (hasta_cerrar_navegador IN (0, 1))
) STRICT;

CREATE INDEX sesion_por_usuario ON sesion (usuario_id);

CREATE TABLE bloqueo_ip (
    ip TEXT PRIMARY KEY,
    intentos_fallidos INTEGER NOT NULL DEFAULT 0 CHECK (intentos_fallidos >= 0),
    bloqueado_hasta TEXT
) STRICT;

CREATE TABLE auditoria (
    id INTEGER PRIMARY KEY,
    cuando_en TEXT NOT NULL,
    usuario_id INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    accion TEXT NOT NULL,
    entidad TEXT NOT NULL,
    entidad_id INTEGER,
    detalle_json TEXT
) STRICT;

CREATE TRIGGER auditoria_no_se_modifica BEFORE UPDATE ON auditoria
BEGIN
    SELECT RAISE(ABORT, 'La auditoría solo admite añadir');
END;

CREATE TRIGGER auditoria_no_se_borra BEFORE DELETE ON auditoria
BEGIN
    SELECT RAISE(ABORT, 'La auditoría solo admite añadir');
END;

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

CREATE TABLE configuracion (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL,
    actualizado_en TEXT NOT NULL,
    usuario_id INTEGER REFERENCES usuario (id) ON DELETE RESTRICT
) STRICT;
