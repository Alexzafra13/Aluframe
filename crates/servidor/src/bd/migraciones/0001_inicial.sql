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
    token_csrf TEXT NOT NULL
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
