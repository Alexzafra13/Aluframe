-- Apuntes: notas y tareas sueltas para lo que no es una ventana (ADR-0025). Solo añade una
-- tabla; los datos existentes no cambian.

CREATE TABLE apunte (
    id INTEGER PRIMARY KEY,
    autor_id INTEGER NOT NULL REFERENCES usuario (id) ON DELETE RESTRICT,
    -- Sin destinatario es para el dueño; con él, una tarea para ese operario.
    para_id INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    -- Si se borra el cliente (protección de datos), se borran sus apuntes.
    cliente_id INTEGER REFERENCES cliente (id) ON DELETE CASCADE,
    direccion_id INTEGER REFERENCES direccion_montaje (id) ON DELETE SET NULL,
    contacto TEXT CHECK (contacto IS NULL OR length(contacto) <= 200),
    texto TEXT NOT NULL CHECK (length(trim(texto)) BETWEEN 1 AND 2000),
    medidas TEXT CHECK (medidas IS NULL OR length(medidas) <= 2000),
    estado TEXT NOT NULL DEFAULT 'pendiente'
        CHECK (estado IN ('pendiente', 'hecho', 'presupuestado')),
    presupuesto_id INTEGER REFERENCES presupuesto (id) ON DELETE SET NULL,
    creado_en TEXT NOT NULL,
    cerrado_en TEXT,
    cerrado_por INTEGER REFERENCES usuario (id) ON DELETE RESTRICT,
    version INTEGER NOT NULL DEFAULT 1
) STRICT;

CREATE INDEX apunte_por_estado ON apunte (estado, para_id);
CREATE INDEX apunte_por_cliente ON apunte (cliente_id) WHERE cliente_id IS NOT NULL;
