-- Permisos que el dueño da a cada operario además de lo básico (ADR-0024). Solo añade una
-- tabla: los datos existentes no cambian y los operarios empiezan sin permisos extra.

CREATE TABLE usuario_permiso (
    usuario_id INTEGER NOT NULL REFERENCES usuario (id) ON DELETE CASCADE,
    permiso TEXT NOT NULL CHECK (
        permiso IN ('editar_clientes', 'ver_presupuestos', 'ver_facturas', 'ver_todos_los_trabajos')
    ),
    PRIMARY KEY (usuario_id, permiso)
) STRICT, WITHOUT ROWID;
