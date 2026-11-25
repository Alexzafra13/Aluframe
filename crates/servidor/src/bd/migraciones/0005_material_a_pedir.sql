-- Material que se encarga a medida para cada ventana del trabajo (propietario, 2026-11-25;
-- ADR-0035): el cristal y lo que la serie marca «se_pide» (la persiana). Lo normal es
-- pendiente → pedido; «recibido» es opcional. Lo marcan el dueño o el operario de la ventana.
-- Solo añade una tabla: no cambia ni borra nada de lo que había.
CREATE TABLE material_a_pedir (
    id INTEGER PRIMARY KEY,
    ventana_id INTEGER NOT NULL REFERENCES ventana_trabajo (id) ON DELETE RESTRICT,
    orden INTEGER NOT NULL CHECK (orden >= 1),
    clase TEXT NOT NULL CHECK (clase IN ('cristal', 'articulo')),
    -- El código de la serie (CRISTAL, PERSIANA); el nombre, «Arriba» o «Abajo» de un cristal partido.
    codigo TEXT NOT NULL,
    nombre TEXT,
    -- La descripción del artículo del catálogo al aceptar: el modelo que se pide.
    modelo TEXT,
    cantidad INTEGER NOT NULL CHECK (cantidad >= 1),
    ancho_dmm INTEGER CHECK (ancho_dmm IS NULL OR ancho_dmm > 0),
    alto_dmm INTEGER CHECK (alto_dmm IS NULL OR alto_dmm > 0),
    estado TEXT NOT NULL DEFAULT 'pendiente' CHECK (estado IN ('pendiente', 'pedido', 'recibido')),
    nota TEXT,
    usuario_id INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    cambiado_en TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    UNIQUE (ventana_id, orden)
) STRICT;

CREATE INDEX material_a_pedir_por_estado ON material_a_pedir (estado);

-- Los trabajos en marcha pasan sus cristales a la lista; lo marcado como recibido (la casilla
-- anterior, «cristal_recibido», que se deja de usar) queda como recibido. Los terminados,
-- entregados o anulados no, para no llenar de avisos lo que ya está hecho.
INSERT INTO material_a_pedir (ventana_id, orden, clase, codigo, nombre, cantidad, ancho_dmm, alto_dmm, estado)
SELECT v.id,
       CAST(c.key AS INTEGER) + 1,
       'cristal',
       json_extract(c.value, '$.tipo'),
       json_extract(c.value, '$.nombre'),
       json_extract(c.value, '$.cantidad'),
       json_extract(c.value, '$.ancho_dmm'),
       json_extract(c.value, '$.alto_dmm'),
       CASE v.cristal_recibido WHEN 1 THEN 'recibido' ELSE 'pendiente' END
FROM ventana_trabajo AS v
JOIN trabajo AS t ON t.id = v.trabajo_id,
     json_each(CASE WHEN json_valid(v.congelado_json) THEN v.congelado_json ELSE '{}' END,
               '$.resumen.vidrios') AS c
WHERE t.estado IN ('pendiente', 'en_curso')
  AND json_extract(c.value, '$.cantidad') >= 1;
