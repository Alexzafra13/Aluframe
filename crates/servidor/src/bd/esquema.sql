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

-- H2: importaciones, clientes y catálogo (DATOS 7.3 y 7.4, ADR-0012).

CREATE TABLE importacion (
    id INTEGER PRIMARY KEY,
    tipo TEXT NOT NULL CHECK (tipo IN ('tarifa', 'clientes')),
    sha256 TEXT NOT NULL CHECK (length(sha256) = 64),
    filas INTEGER NOT NULL CHECK (filas >= 0),
    estado TEXT NOT NULL DEFAULT 'confirmada' CHECK (estado IN ('confirmada', 'deshecha')),
    usuario_id INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    importada_en TEXT NOT NULL,
    deshecha_en TEXT
) STRICT;

CREATE TABLE cliente (
    id INTEGER PRIMARY KEY,
    tipo TEXT NOT NULL DEFAULT 'particular' CHECK (tipo IN ('particular', 'empresa')),
    nombre_o_razon_social TEXT,
    nif TEXT,
    email TEXT,
    telefono TEXT,
    fiscal_calle TEXT,
    fiscal_cp TEXT,
    fiscal_localidad TEXT,
    fiscal_provincia TEXT,
    fiscal_pais TEXT,
    forma_de_pago TEXT,
    notas TEXT,
    busqueda TEXT NOT NULL DEFAULT '',
    importacion_id INTEGER REFERENCES importacion (id) ON DELETE RESTRICT,
    archivado INTEGER NOT NULL DEFAULT 0 CHECK (archivado IN (0, 1)),
    creado_en TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
) STRICT;

CREATE INDEX cliente_por_nif ON cliente (nif) WHERE nif IS NOT NULL;
CREATE INDEX cliente_por_importacion ON cliente (importacion_id) WHERE importacion_id IS NOT NULL;

CREATE TABLE direccion_montaje (
    id INTEGER PRIMARY KEY,
    cliente_id INTEGER NOT NULL REFERENCES cliente (id) ON DELETE RESTRICT,
    etiqueta TEXT,
    calle TEXT,
    cp TEXT,
    localidad TEXT,
    provincia TEXT,
    contacto_nombre TEXT,
    contacto_telefono TEXT,
    notas TEXT,
    archivado INTEGER NOT NULL DEFAULT 0 CHECK (archivado IN (0, 1))
) STRICT;

CREATE INDEX direccion_por_cliente ON direccion_montaje (cliente_id);

CREATE TABLE proveedor (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(nombre)) BETWEEN 1 AND 120),
    nif TEXT,
    contacto TEXT,
    condiciones_pago TEXT,
    descuento_habitual_bp INTEGER NOT NULL DEFAULT 0
        CHECK (descuento_habitual_bp BETWEEN 0 AND 10000),
    notas TEXT,
    archivado INTEGER NOT NULL DEFAULT 0 CHECK (archivado IN (0, 1)),
    version INTEGER NOT NULL DEFAULT 1
) STRICT;

-- El blanco es el acabado base (precio de tarifa) y no lleva recargo.
CREATE TABLE acabado (
    id INTEGER PRIMARY KEY,
    nombre TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (length(trim(nombre)) BETWEEN 1 AND 60),
    es_base INTEGER NOT NULL DEFAULT 0 CHECK (es_base IN (0, 1)),
    recargo_bp INTEGER NOT NULL DEFAULT 0 CHECK (recargo_bp BETWEEN 0 AND 100000),
    archivado INTEGER NOT NULL DEFAULT 0 CHECK (archivado IN (0, 1)),
    version INTEGER NOT NULL DEFAULT 1,
    CHECK (es_base = 0 OR recargo_bp = 0)
) STRICT;

CREATE UNIQUE INDEX acabado_un_solo_base ON acabado (es_base) WHERE es_base = 1;

CREATE TABLE articulo (
    id INTEGER PRIMARY KEY,
    proveedor_id INTEGER NOT NULL REFERENCES proveedor (id) ON DELETE RESTRICT,
    referencia TEXT NOT NULL CHECK (length(trim(referencia)) BETWEEN 1 AND 60),
    descripcion TEXT NOT NULL CHECK (length(trim(descripcion)) BETWEEN 1 AND 200),
    tipo TEXT NOT NULL DEFAULT 'otro'
        CHECK (tipo IN ('perfil', 'vidrio', 'herraje', 'accesorio', 'kit', 'otro')),
    unidad TEXT NOT NULL CHECK (unidad IN ('barra', 'metro', 'm2', 'unidad', 'caja')),
    longitud_barra_dmm INTEGER CHECK (longitud_barra_dmm IS NULL OR longitud_barra_dmm > 0),
    unidades_por_caja INTEGER CHECK (unidades_por_caja IS NULL OR unidades_por_caja > 0),
    con_acabado INTEGER NOT NULL DEFAULT 0 CHECK (con_acabado IN (0, 1)),
    stock_minimo_mil INTEGER NOT NULL DEFAULT 0 CHECK (stock_minimo_mil >= 0),
    importacion_id INTEGER REFERENCES importacion (id) ON DELETE RESTRICT,
    archivado INTEGER NOT NULL DEFAULT 0 CHECK (archivado IN (0, 1)),
    creado_en TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE (proveedor_id, referencia),
    CHECK ((unidad = 'barra') = (longitud_barra_dmm IS NOT NULL))
) STRICT;

-- Cada importación o ajuste crea una versión completa; solo una está vigente por proveedor.
CREATE TABLE tarifa (
    id INTEGER PRIMARY KEY,
    proveedor_id INTEGER NOT NULL REFERENCES proveedor (id) ON DELETE RESTRICT,
    nombre TEXT NOT NULL CHECK (length(trim(nombre)) BETWEEN 1 AND 120),
    fecha_tarifa TEXT,
    vigente_desde TEXT NOT NULL,
    descuento_compra_bp INTEGER NOT NULL DEFAULT 0 CHECK (descuento_compra_bp BETWEEN 0 AND 10000),
    formato_origen TEXT NOT NULL CHECK (formato_origen IN ('csv', 'ajuste', 'edicion')),
    archivo_origen TEXT,
    estado TEXT NOT NULL CHECK (estado IN ('vigente', 'sustituida')),
    importacion_id INTEGER REFERENCES importacion (id) ON DELETE RESTRICT,
    importada_en TEXT NOT NULL,
    usuario_id INTEGER REFERENCES usuario (id) ON DELETE RESTRICT
) STRICT;

CREATE UNIQUE INDEX tarifa_una_vigente ON tarifa (proveedor_id) WHERE estado = 'vigente';

CREATE TABLE tarifa_precio (
    tarifa_id INTEGER NOT NULL REFERENCES tarifa (id) ON DELETE RESTRICT,
    articulo_id INTEGER NOT NULL REFERENCES articulo (id) ON DELETE RESTRICT,
    precio_pu INTEGER NOT NULL CHECK (precio_pu > 0),
    minimo_pedido_mil INTEGER CHECK (minimo_pedido_mil IS NULL OR minimo_pedido_mil > 0),
    PRIMARY KEY (tarifa_id, articulo_id)
) STRICT;

CREATE INDEX tarifa_precio_por_articulo ON tarifa_precio (articulo_id);
