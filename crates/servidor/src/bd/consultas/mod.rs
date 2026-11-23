pub mod acabados;
pub mod articulos;
pub mod articulos_con_precio;
pub mod auditoria;
pub mod bloqueo_ip;
pub mod clientes;
pub mod cobros;
pub mod configuracion;
pub mod direcciones;
pub mod facturas;
pub mod importaciones;
pub mod lineas_presupuesto;
pub mod pasos;
pub mod permisos;
pub mod presupuestos;
pub mod proveedores;
pub mod repetidas;
pub mod series;
pub mod sesiones;
pub mod stock;
pub mod tarifas;
pub mod trabajos;
pub mod usuarios;
pub mod ventanas_trabajo;
pub mod versiones_presupuesto;

/// Escapa los comodines de LIKE para que «%» o «_» se busquen como texto.
pub fn escapar_like(texto: &str) -> String {
    texto
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
