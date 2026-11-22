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
    -- Color del acabado (ADR-0019): para el dibujo y la hoja; no interviene en el precio.
    ral TEXT CHECK (ral IS NULL OR length(ral) BETWEEN 1 AND 10),
    color_dibujo TEXT NOT NULL DEFAULT '#f2f2ed' CHECK (color_dibujo GLOB '#[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]'),
    madera INTEGER NOT NULL DEFAULT 0 CHECK (madera IN (0, 1)),
    superficie TEXT NOT NULL DEFAULT 'brillo' CHECK (superficie IN ('brillo', 'mate', 'texturado', 'anodizado')),
    cierres TEXT NOT NULL DEFAULT 'negro' CHECK (cierres IN ('blanco', 'negro')),
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

-- H3: presupuestos (DATOS 7.6). Numeración sin huecos solo para facturas (BACKEND 5.6).

CREATE TABLE secuencia_documentos (
    serie TEXT NOT NULL CHECK (serie IN ('P', 'T', 'F', 'R', 'A')),
    anio INTEGER NOT NULL CHECK (anio BETWEEN 2000 AND 9999),
    ultimo INTEGER NOT NULL CHECK (ultimo >= 0),
    PRIMARY KEY (serie, anio)
) STRICT;

CREATE TABLE presupuesto (
    id INTEGER PRIMARY KEY,
    anio INTEGER NOT NULL,
    numero INTEGER NOT NULL CHECK (numero >= 1),
    cliente_id INTEGER NOT NULL REFERENCES cliente (id) ON DELETE RESTRICT,
    direccion_montaje_id INTEGER REFERENCES direccion_montaje (id) ON DELETE RESTRICT,
    estado TEXT NOT NULL DEFAULT 'borrador'
        CHECK (estado IN ('borrador', 'enviado', 'aceptado', 'rechazado', 'caducado')),
    validez_hasta TEXT,
    condiciones TEXT,
    descuento_bp INTEGER NOT NULL DEFAULT 0 CHECK (descuento_bp BETWEEN 0 AND 10000),
    enviado_en TEXT,
    aceptado_en TEXT,
    copia_json TEXT,
    copia_sha256 TEXT CHECK (copia_sha256 IS NULL OR length(copia_sha256) = 64),
    creado_por INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    creado_en TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE (anio, numero),
    CHECK ((estado = 'aceptado') = (copia_json IS NOT NULL))
) STRICT;

CREATE INDEX presupuesto_por_cliente ON presupuesto (cliente_id);

-- Lo aceptado no se toca: la copia congelada es la que vale (DATOS, regla 3).
CREATE TRIGGER presupuesto_aceptado_no_se_modifica BEFORE UPDATE ON presupuesto
WHEN OLD.estado = 'aceptado'
BEGIN
    SELECT RAISE(ABORT, 'Un presupuesto aceptado no se modifica');
END;

CREATE TABLE version_presupuesto (
    id INTEGER PRIMARY KEY,
    presupuesto_id INTEGER NOT NULL REFERENCES presupuesto (id) ON DELETE RESTRICT,
    numero_version INTEGER NOT NULL CHECK (numero_version >= 1),
    motivo TEXT,
    vigente INTEGER NOT NULL CHECK (vigente IN (0, 1)),
    base_cent INTEGER NOT NULL DEFAULT 0,
    iva_cent INTEGER NOT NULL DEFAULT 0,
    total_cent INTEGER NOT NULL DEFAULT 0,
    completo INTEGER NOT NULL DEFAULT 0 CHECK (completo IN (0, 1)),
    creada_en TEXT NOT NULL,
    UNIQUE (presupuesto_id, numero_version)
) STRICT;

CREATE UNIQUE INDEX version_vigente_unica ON version_presupuesto (presupuesto_id) WHERE vigente = 1;

CREATE TABLE linea_presupuesto (
    id INTEGER PRIMARY KEY,
    version_id INTEGER NOT NULL REFERENCES version_presupuesto (id) ON DELETE RESTRICT,
    orden INTEGER NOT NULL CHECK (orden >= 1),
    tipo TEXT NOT NULL CHECK (tipo IN ('ventana', 'libre')),
    descripcion TEXT NOT NULL,
    cantidad_mil INTEGER NOT NULL CHECK (cantidad_mil > 0),
    serie_version_id INTEGER REFERENCES serie_version (id) ON DELETE RESTRICT,
    tipologia TEXT,
    ancho_dmm INTEGER CHECK (ancho_dmm IS NULL OR ancho_dmm > 0),
    alto_dmm INTEGER CHECK (alto_dmm IS NULL OR alto_dmm > 0),
    acabado_id INTEGER REFERENCES acabado (id) ON DELETE RESTRICT,
    vidrio_articulo_id INTEGER REFERENCES articulo (id) ON DELETE RESTRICT,
    opciones_json TEXT NOT NULL DEFAULT '{}',
    resultado_json TEXT,
    faltan_json TEXT NOT NULL DEFAULT '[]',
    incompleta INTEGER NOT NULL CHECK (incompleta IN (0, 1)),
    revisar_limites INTEGER NOT NULL DEFAULT 0 CHECK (revisar_limites IN (0, 1)),
    tarifas_json TEXT NOT NULL DEFAULT '[]',
    coste_cent INTEGER NOT NULL DEFAULT 0,
    mano_de_obra_cent INTEGER NOT NULL DEFAULT 0,
    descuento_bp INTEGER NOT NULL DEFAULT 0 CHECK (descuento_bp BETWEEN 0 AND 10000),
    tipo_iva_bp INTEGER NOT NULL CHECK (tipo_iva_bp BETWEEN 0 AND 10000),
    precio_unitario_pu INTEGER,
    precio_cent INTEGER NOT NULL DEFAULT 0,
    UNIQUE (version_id, orden),
    CHECK ((tipo = 'ventana') = (serie_version_id IS NOT NULL))
) STRICT;

-- Tarifas que usa un presupuesto aceptado: esas ya no se pueden deshacer (ADR-0012).
CREATE TABLE presupuesto_tarifa (
    presupuesto_id INTEGER NOT NULL REFERENCES presupuesto (id) ON DELETE RESTRICT,
    tarifa_id INTEGER NOT NULL REFERENCES tarifa (id) ON DELETE RESTRICT,
    PRIMARY KEY (presupuesto_id, tarifa_id)
) STRICT;

-- H4: trabajos y orden de taller (DATOS 7.7). Nacen al aceptar un presupuesto, con una copia
-- congelada de cada ventana: cambiar después una serie o una tarifa no los altera.
CREATE TABLE trabajo (
    id INTEGER PRIMARY KEY,
    anio INTEGER NOT NULL,
    numero INTEGER NOT NULL CHECK (numero >= 1),
    presupuesto_id INTEGER NOT NULL UNIQUE REFERENCES presupuesto (id) ON DELETE RESTRICT,
    cliente_id INTEGER NOT NULL REFERENCES cliente (id) ON DELETE RESTRICT,
    cliente_nombre TEXT NOT NULL,
    direccion_montaje TEXT,
    estado TEXT NOT NULL DEFAULT 'pendiente'
        CHECK (estado IN ('pendiente', 'en_curso', 'terminado', 'entregado', 'anulado')),
    asignado_a INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    fecha_prevista TEXT,
    importe_cent INTEGER NOT NULL DEFAULT 0,
    creado_en TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE (anio, numero)
) STRICT;

CREATE INDEX trabajo_por_cliente ON trabajo (cliente_id);

-- Una ventana por unidad: una línea con cantidad 3 son 3 ventanas, cada una con sus pasos.
CREATE TABLE ventana_trabajo (
    id INTEGER PRIMARY KEY,
    trabajo_id INTEGER NOT NULL REFERENCES trabajo (id) ON DELETE RESTRICT,
    orden INTEGER NOT NULL CHECK (orden >= 1),
    linea_orden INTEGER NOT NULL CHECK (linea_orden >= 1),
    descripcion TEXT NOT NULL,
    asignado_a INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    cristal_recibido INTEGER NOT NULL DEFAULT 0 CHECK (cristal_recibido IN (0, 1)),
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision >= 1),
    revisada_en TEXT,
    medida_cambiada INTEGER NOT NULL DEFAULT 0 CHECK (medida_cambiada IN (0, 1)),
    falta_algo TEXT,
    congelado_json TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE (trabajo_id, orden)
) STRICT;

CREATE TABLE paso_ventana (
    id INTEGER PRIMARY KEY,
    ventana_id INTEGER NOT NULL REFERENCES ventana_trabajo (id) ON DELETE RESTRICT,
    orden INTEGER NOT NULL CHECK (orden >= 1),
    nombre TEXT NOT NULL,
    estado TEXT NOT NULL DEFAULT 'pendiente' CHECK (estado IN ('pendiente', 'en_curso', 'hecho')),
    usuario_id INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    cambiado_en TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE (ventana_id, orden)
) STRICT;

-- Quién cambió cada paso y cuándo; solo se añade.
CREATE TABLE paso_historial (
    id INTEGER PRIMARY KEY,
    paso_id INTEGER NOT NULL REFERENCES paso_ventana (id) ON DELETE RESTRICT,
    estado_anterior TEXT NOT NULL,
    estado_nuevo TEXT NOT NULL,
    nota TEXT,
    usuario_id INTEGER NOT NULL REFERENCES usuario (id) ON DELETE RESTRICT,
    cuando_en TEXT NOT NULL
) STRICT;

CREATE TRIGGER paso_historial_solo_anadir_u BEFORE UPDATE ON paso_historial
BEGIN SELECT RAISE(ABORT, 'El historial de pasos solo admite añadir'); END;
CREATE TRIGGER paso_historial_solo_anadir_d BEFORE DELETE ON paso_historial
BEGIN SELECT RAISE(ABORT, 'El historial de pasos solo admite añadir'); END;

-- Cada «Modificar medidas» del propietario: revisión con motivo; solo se añade.
CREATE TABLE cambio_ventana_trabajo (
    id INTEGER PRIMARY KEY,
    ventana_id INTEGER NOT NULL REFERENCES ventana_trabajo (id) ON DELETE RESTRICT,
    revision INTEGER NOT NULL CHECK (revision >= 2),
    congelado_anterior_json TEXT NOT NULL,
    congelado_nuevo_json TEXT NOT NULL,
    motivo TEXT NOT NULL CHECK (length(trim(motivo)) >= 1),
    pasos_reiniciados INTEGER NOT NULL CHECK (pasos_reiniciados IN (0, 1)),
    usuario_id INTEGER NOT NULL REFERENCES usuario (id) ON DELETE RESTRICT,
    cuando_en TEXT NOT NULL,
    UNIQUE (ventana_id, revision)
) STRICT;

CREATE TRIGGER cambio_ventana_solo_anadir_u BEFORE UPDATE ON cambio_ventana_trabajo
BEGIN SELECT RAISE(ABORT, 'Las revisiones de una ventana solo admiten añadir'); END;
CREATE TRIGGER cambio_ventana_solo_anadir_d BEFORE DELETE ON cambio_ventana_trabajo
BEGIN SELECT RAISE(ABORT, 'Las revisiones de una ventana solo admiten añadir'); END;

-- La orden se termina una vez: entonces se congela el plan de corte y se descuenta el stock (H5).
CREATE TABLE orden_taller (
    id INTEGER PRIMARY KEY,
    trabajo_id INTEGER NOT NULL UNIQUE REFERENCES trabajo (id) ON DELETE RESTRICT,
    estado TEXT NOT NULL DEFAULT 'abierta' CHECK (estado IN ('abierta', 'terminada')),
    plan_de_corte_json TEXT,
    terminada_en TEXT,
    terminada_por INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    CHECK ((estado = 'terminada') = (plan_de_corte_json IS NOT NULL))
) STRICT;

CREATE TRIGGER orden_terminada_no_se_modifica BEFORE UPDATE ON orden_taller
WHEN OLD.estado = 'terminada'
BEGIN SELECT RAISE(ABORT, 'Una orden terminada no se modifica'); END;

-- H5: libro de stock (DATOS 7.8). El saldo es la suma de los movimientos; no hay saldo guardado.
CREATE TABLE movimiento_stock (
    id INTEGER PRIMARY KEY,
    articulo_id INTEGER NOT NULL REFERENCES articulo (id) ON DELETE RESTRICT,
    acabado_id INTEGER REFERENCES acabado (id) ON DELETE RESTRICT,
    longitud_dmm INTEGER CHECK (longitud_dmm IS NULL OR longitud_dmm > 0),
    tipo TEXT NOT NULL CHECK (tipo IN ('entrada', 'salida', 'ajuste')),
    cantidad_mil INTEGER NOT NULL CHECK (cantidad_mil <> 0),
    motivo TEXT NOT NULL CHECK (length(trim(motivo)) >= 1),
    orden_taller_id INTEGER REFERENCES orden_taller (id) ON DELETE RESTRICT,
    usuario_id INTEGER NOT NULL REFERENCES usuario (id) ON DELETE RESTRICT,
    cuando_en TEXT NOT NULL,
    CHECK (tipo <> 'entrada' OR cantidad_mil > 0),
    CHECK (tipo <> 'salida' OR cantidad_mil < 0)
) STRICT;

CREATE INDEX movimiento_por_articulo ON movimiento_stock (articulo_id, acabado_id, longitud_dmm);
CREATE UNIQUE INDEX movimiento_una_salida_por_orden ON movimiento_stock
    (orden_taller_id, articulo_id, ifnull(acabado_id, 0), ifnull(longitud_dmm, 0))
    WHERE orden_taller_id IS NOT NULL;

CREATE TRIGGER movimiento_solo_anadir_u BEFORE UPDATE ON movimiento_stock
BEGIN SELECT RAISE(ABORT, 'El stock no se edita: se corrige con un ajuste'); END;
CREATE TRIGGER movimiento_solo_anadir_d BEFORE DELETE ON movimiento_stock
BEGIN SELECT RAISE(ABORT, 'El stock no se edita: se corrige con un ajuste'); END;

-- H6: albaranes, facturas y cobros (DATOS 7.9). Lo emitido no se toca.
CREATE TABLE albaran (
    id INTEGER PRIMARY KEY,
    anio INTEGER NOT NULL,
    numero INTEGER NOT NULL CHECK (numero >= 1),
    trabajo_id INTEGER NOT NULL REFERENCES trabajo (id) ON DELETE RESTRICT,
    fecha TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,
    snapshot_sha256 TEXT NOT NULL CHECK (length(snapshot_sha256) = 64),
    UNIQUE (anio, numero)
) STRICT;

CREATE TRIGGER albaran_no_se_modifica_u BEFORE UPDATE ON albaran
BEGIN SELECT RAISE(ABORT, 'Un albarán emitido no se modifica'); END;
CREATE TRIGGER albaran_no_se_modifica_d BEFORE DELETE ON albaran
BEGIN SELECT RAISE(ABORT, 'Un albarán emitido no se modifica'); END;

CREATE TABLE factura (
    id INTEGER PRIMARY KEY,
    serie TEXT NOT NULL CHECK (serie IN ('F', 'R')),
    anio INTEGER NOT NULL,
    numero INTEGER CHECK (numero IS NULL OR numero >= 1),
    tipo TEXT NOT NULL DEFAULT 'ordinaria' CHECK (tipo IN ('ordinaria', 'rectificativa')),
    rectifica_factura_id INTEGER REFERENCES factura (id) ON DELETE RESTRICT,
    motivo_rectificacion TEXT,
    cliente_id INTEGER NOT NULL REFERENCES cliente (id) ON DELETE RESTRICT,
    trabajo_id INTEGER REFERENCES trabajo (id) ON DELETE RESTRICT,
    estado TEXT NOT NULL DEFAULT 'borrador' CHECK (estado IN ('borrador', 'emitida')),
    fecha_emision TEXT,
    fecha_operacion TEXT,
    vencimiento TEXT,
    notas TEXT,
    descuento_bp INTEGER NOT NULL DEFAULT 0 CHECK (descuento_bp BETWEEN 0 AND 10000),
    base_cent INTEGER NOT NULL DEFAULT 0,
    iva_cent INTEGER NOT NULL DEFAULT 0,
    total_cent INTEGER NOT NULL DEFAULT 0,
    snapshot_json TEXT,
    snapshot_sha256 TEXT CHECK (snapshot_sha256 IS NULL OR length(snapshot_sha256) = 64),
    -- Reservados para Verifactu y la factura electrónica (SPEC, H6): vacíos por ahora.
    huella_anterior TEXT,
    huella TEXT,
    estado_envio TEXT,
    creado_en TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE (serie, anio, numero),
    CHECK ((estado = 'emitida') = (numero IS NOT NULL AND snapshot_json IS NOT NULL)),
    CHECK ((tipo = 'rectificativa') = (rectifica_factura_id IS NOT NULL))
) STRICT;

CREATE INDEX factura_por_cliente ON factura (cliente_id);
CREATE INDEX factura_por_trabajo ON factura (trabajo_id) WHERE trabajo_id IS NOT NULL;

CREATE TRIGGER factura_emitida_no_se_modifica BEFORE UPDATE ON factura
WHEN OLD.estado = 'emitida'
BEGIN SELECT RAISE(ABORT, 'Una factura emitida no se modifica: se hace una rectificativa'); END;
CREATE TRIGGER factura_emitida_no_se_borra BEFORE DELETE ON factura
WHEN OLD.estado = 'emitida'
BEGIN SELECT RAISE(ABORT, 'Una factura emitida no se borra: se hace una rectificativa'); END;

CREATE TABLE linea_factura (
    id INTEGER PRIMARY KEY,
    factura_id INTEGER NOT NULL REFERENCES factura (id) ON DELETE CASCADE,
    orden INTEGER NOT NULL CHECK (orden >= 1),
    descripcion TEXT NOT NULL CHECK (length(trim(descripcion)) >= 1),
    cantidad_mil INTEGER NOT NULL CHECK (cantidad_mil <> 0),
    precio_unitario_pu INTEGER NOT NULL,
    descuento_bp INTEGER NOT NULL DEFAULT 0 CHECK (descuento_bp BETWEEN 0 AND 10000),
    tipo_iva_bp INTEGER NOT NULL CHECK (tipo_iva_bp BETWEEN 0 AND 10000),
    base_cent INTEGER NOT NULL,
    UNIQUE (factura_id, orden)
) STRICT;

CREATE TRIGGER linea_de_factura_emitida_u BEFORE UPDATE ON linea_factura
WHEN (SELECT estado FROM factura WHERE id = OLD.factura_id) = 'emitida'
BEGIN SELECT RAISE(ABORT, 'Una factura emitida no se modifica'); END;
CREATE TRIGGER linea_de_factura_emitida_d BEFORE DELETE ON linea_factura
WHEN (SELECT estado FROM factura WHERE id = OLD.factura_id) = 'emitida'
BEGIN SELECT RAISE(ABORT, 'Una factura emitida no se modifica'); END;
CREATE TRIGGER linea_de_factura_emitida_i BEFORE INSERT ON linea_factura
WHEN (SELECT estado FROM factura WHERE id = NEW.factura_id) = 'emitida'
BEGIN SELECT RAISE(ABORT, 'Una factura emitida no se modifica'); END;

-- Una señal se cobra en el trabajo antes de la factura; al emitirla cuenta como pagado a cuenta.
CREATE TABLE cobro (
    id INTEGER PRIMARY KEY,
    factura_id INTEGER REFERENCES factura (id) ON DELETE RESTRICT,
    trabajo_id INTEGER REFERENCES trabajo (id) ON DELETE RESTRICT,
    fecha TEXT NOT NULL,
    importe_cent INTEGER NOT NULL CHECK (importe_cent > 0),
    medio TEXT NOT NULL CHECK (medio IN ('efectivo', 'bizum', 'tarjeta', 'transferencia', 'otro')),
    nota TEXT,
    anulado INTEGER NOT NULL DEFAULT 0 CHECK (anulado IN (0, 1)),
    motivo_anulacion TEXT,
    usuario_id INTEGER NOT NULL REFERENCES usuario (id) ON DELETE RESTRICT,
    creado_en TEXT NOT NULL,
    CHECK (factura_id IS NOT NULL OR trabajo_id IS NOT NULL),
    CHECK (anulado = 0 OR motivo_anulacion IS NOT NULL)
) STRICT;

CREATE INDEX cobro_por_factura ON cobro (factura_id) WHERE factura_id IS NOT NULL;
CREATE INDEX cobro_por_trabajo ON cobro (trabajo_id) WHERE trabajo_id IS NOT NULL;
